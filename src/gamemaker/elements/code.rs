use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::elements::variables::GMVariable;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::serialize::DataBuilder;
use crate::utility::num_enum_from;

#[derive(Debug, Clone)]
pub struct GMCodes {
    pub codes: Vec<GMCode>,
    pub exists: bool,
}
impl GMChunkElement for GMCodes {
    fn empty() -> Self {
        Self { codes: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMCodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        if reader.get_chunk_length() == 0 {
            return Ok(Self { codes: vec![], exists: false })
        }
        
        let pointers: Vec<usize> = reader.read_simple_list()?;
        reader.cur_pos = match pointers.first() {
            Some(ptr) => *ptr,
            None => return Ok(Self { codes: vec![], exists: true })
        };
        let count: usize = pointers.len();
        let mut codes: Vec<GMCode> = Vec::with_capacity(count);
        let mut instructions_ranges: Vec<(usize, usize)> = Vec::with_capacity(count);
        let mut codes_by_pos: HashMap<usize, GMRef<GMCode>> = HashMap::new();
        let mut last_pos: usize = reader.cur_pos;
        
        for pointer in pointers {
            reader.assert_pos(pointer, "Code")?;
            let name: GMRef<String> = reader.read_gm_string()?;
            let code_length: usize = reader.read_usize()?;

            let instructions_start_pos: usize;
            let instructions_end_pos: usize;
            let bytecode15_info: Option<GMCodeBytecode15>;

            if reader.general_info.bytecode_version <= 14 {
                instructions_start_pos = reader.cur_pos;    // instructions are placed immediately after code metadata; how convenient!
                reader.cur_pos += code_length;  // skip over them; they will get parsed in the next loop
                instructions_end_pos = reader.cur_pos;
                bytecode15_info = None;
            } else {
                let locals_count: u16 = reader.read_u16()?;
                let arguments_count_raw: u16 = reader.read_u16()?;
                let arguments_count: u16 = arguments_count_raw & 0x7FFF;
                let weird_local_flag: bool = arguments_count_raw & 0x8000 != 0;

                let instructions_start_offset: i32 = reader.read_i32()?;
                instructions_start_pos = (instructions_start_offset + reader.cur_pos as i32 - 4) as usize;

                let offset: usize = reader.read_usize()?;
                let b15_info = GMCodeBytecode15 { locals_count, arguments_count, weird_local_flag, offset, parent: None };
                instructions_end_pos = instructions_start_pos + code_length;
                bytecode15_info = Some(b15_info);
            };
            
            codes.push(GMCode { name, instructions: vec![], bytecode15_info });
            instructions_ranges.push((instructions_start_pos, instructions_end_pos));
            last_pos = reader.cur_pos;
        }
        
        for (i, (start, end)) in instructions_ranges.into_iter().enumerate() {
            let code: &mut GMCode = &mut codes[i];
            let length: usize = end - start;

            // if bytecode15+ and the instructions pointer is known, then it's a child code entry
            if length > 0 {
                if let Some(parent_code) = codes_by_pos.get(&start) {
                    if let Some(ref mut b15_info) = code.bytecode15_info {
                        b15_info.parent = Some(parent_code.clone());
                        continue;
                    }
                }
            }

            reader.cur_pos = start;
            code.instructions = Vec::with_capacity(length / 6);  // estimate; data is from deltarune 1.00

            while reader.cur_pos < end {
                let instruction = GMInstruction::deserialize(reader).map_err(|e| format!(
                    "{e}\n↳ for Instruction #{} (at absolute position {}) of Code entry \"{}\" with absolute start position {}",
                    code.instructions.len(), reader.cur_pos, reader.display_gm_str(code.name), start,
                ))?;
                code.instructions.push(instruction);
            }
            code.instructions.shrink_to_fit();

            if length > 0 {
                // Update information to mark this entry as the root (if we have at least 1 instruction)
                codes_by_pos.insert(start, GMRef::new(i as u32));
            }
        }
        
        reader.cur_pos = last_pos;  // has to be chunk end (since instructions are stored separately in b15+)
        Ok(GMCodes { codes, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }

        builder.write_usize(self.codes.len())?;
        let pointer_list_pos: usize = builder.len();
        for _ in 0..self.codes.len() {
            builder.write_u32(0xDEADC0DE);
        }
        
        // bytecode 14 my beloved
        if builder.bytecode_version() <= 14 {
            for (i, code) in self.codes.iter().enumerate() {
                builder.overwrite_usize(builder.len(), pointer_list_pos + 4*i)?;
                builder.write_gm_string(&code.name)?;
                let length_placeholder_pos: usize = builder.len();
                builder.write_u32(0xDEADC0DE);
                let start: usize = builder.len();

                // in bytecode 14, instructions are written immediately
                for (i, instruction) in code.instructions.iter().enumerate() {
                    instruction.serialize(builder).map_err(|e| format!(
                        "{e}\n↳ while building bytecode14 code #{i} with name \"{}\"",
                        builder.display_gm_str(&code.name),
                    ))?;
                }

                let code_length: usize = builder.len() - start;
                builder.overwrite_usize(code_length, length_placeholder_pos)?;
            }
            return Ok(())
        }
        
        // in bytecode 15, the codes' instructions are written before the codes metadata
        let mut instructions_ranges: Vec<(usize, usize)> = Vec::with_capacity(self.codes.len());

        for (i, code) in self.codes.iter().enumerate() {
            if code.bytecode15_info.as_ref().unwrap().parent.is_some() {
                // If this is a child code entry, don't write instructions; just repeat last pointer
                instructions_ranges.push(instructions_ranges.last().unwrap().clone());
                // ^ this unwrap will fail if the first entry is a child entry (which is invalid anyway)
                continue
            }
            
            let start: usize = builder.len();
            for instruction in &code.instructions {
                instruction.serialize(builder).map_err(|e| format!(
                    "{e}\n↳ while serializing bytecode15 code #{i} with name \"{}\"",
                    builder.display_gm_str(&code.name),
                ))?;
            }
            let end: usize = builder.len();
            instructions_ranges.push((start, end));
        }

        for (i, code) in self.codes.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_pos + 4*i)?;
            let (start, end) = instructions_ranges[i];
            let length: usize = end - start;
            let b15_info: &GMCodeBytecode15 = code.bytecode15_info.as_ref()
                .ok_or_else(|| format!("Code bytecode 15 data not set in Bytecode version {}", builder.bytecode_version()))?;
            
            builder.write_gm_string(&code.name)?;
            builder.write_usize(length)?;
            builder.write_u16(b15_info.locals_count);
            builder.write_u16(b15_info.arguments_count | if b15_info.weird_local_flag {0x8000} else {0});
            let instructions_start_offset: i32 = start as i32 - builder.len() as i32;
            builder.write_i32(instructions_start_offset);
            builder.write_usize(b15_info.offset)?;
        }
        
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCode {
    pub name: GMRef<String>,
    pub instructions: Vec<GMInstruction>,
    pub bytecode15_info: Option<GMCodeBytecode15>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeBytecode15 {
    pub locals_count: u16,
    pub arguments_count: u16,
    pub weird_local_flag: bool,
    pub offset: usize,
    pub parent: Option<GMRef<GMCode>>,
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMInstructionData {
    Empty,
    SingleType(GMSingleTypeInstruction),
    Duplicate(GMDuplicateInstruction),
    DuplicateSwap(GMDuplicateSwapInstruction),
    CallVariable(GMCallVariableInstruction),
    DoubleType(GMDoubleTypeInstruction),
    Comparison(GMComparisonInstruction),
    Goto(GMGotoInstruction),
    Pop(GMPopInstruction),
    PopSwap(GMPopSwapInstruction),
    Push(GMPushInstruction),
    Call(GMCallInstruction),
    Extended16(GMExtendedInstruction16),
    Extended32(GMExtendedInstruction32),
    ExtendedFunc(GMExtendedInstructionFunction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMInstruction {
    pub opcode: GMOpcode,
    pub kind: GMInstructionData,
}
impl GMElement for GMInstruction {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let b0: u8 = reader.read_u8()?;
        let b1: u8 = reader.read_u8()?;
        let b2: u8 = reader.read_u8()?;
        let opcode = GMOpcode::deserialize(reader)?;
        // log::debug!("{} // {:02X} {:02X} {:02X} {:02X} // {:?}", reader.cur_pos-4, b0, b1, b2, u8::from(opcode), opcode);
        
        let instruction_data: GMInstructionData = match opcode {
            GMOpcode::Exit => {
                GMInstructionData::Empty
            }

            GMOpcode::Duplicate => {
                let data_type: GMDataType = num_enum_from(b2 & 0xf)
                    .map_err(|e| format!("{e} while parsing Duplicate Instruction"))?;
                if b0 == 0 {  // TODO: b1????
                    GMInstructionData::Duplicate(GMDuplicateInstruction { data_type, size: b0 })
                } else {
                    GMInstructionData::DuplicateSwap(GMDuplicateSwapInstruction { data_type, size1: b0, size2: b1 })
                }
            }

            GMOpcode::CallVariable => {
                let data_type: GMDataType = num_enum_from(b2 & 0xf)
                    .map_err(|e| format!("{e} while parsing Call Variable Instruction"))?;
                GMInstructionData::CallVariable(GMCallVariableInstruction { data_type, argument_count: b1 })
            }

            GMOpcode::Negate |
            GMOpcode::Not |
            GMOpcode::Return |
            GMOpcode::PopDiscard => {
                let data_type: GMDataType = num_enum_from(b2 & 0xf)
                    .map_err(|e| format!("{e} while parsing Single Type Instruction"))?;

                // Ensure basic conditions hold
                if b0 != 0 {
                    return Err(format!("Invalid padding {:02X} while parsing Single Type Instruction", b0));
                }
                if b2 >> 4 != 0 {
                    return Err(format!("Second type should be zero but is {0} (0x{0:02X}) for Single Type Instruction", b2 >> 4))
                }

                GMInstructionData::SingleType(GMSingleTypeInstruction { data_type })
            }

            GMOpcode::Convert |
            GMOpcode::Multiply |
            GMOpcode::Divide |
            GMOpcode::Remainder |
            GMOpcode::Modulus |
            GMOpcode::Add |
            GMOpcode::Subtract |
            GMOpcode::And |
            GMOpcode::Or |
            GMOpcode::Xor |
            GMOpcode::ShiftLeft |
            GMOpcode::ShiftRight => {
                let type1: GMDataType = num_enum_from(b2 & 0xf)
                    .map_err(|e| format!("{e} for type1 while parsing Double Type Instruction"))?;
                let type2: GMDataType = num_enum_from(b2 >> 4).
                    map_err(|e| format!("{e} for type2 while parsing Double Type Instruction"))?;

                if b1 != 0 {    // might be incorrect; remove if issues
                    return Err(format!("b1 should be zero but is {b1} (0x{b1:02X}) for Double Type Instruction"))
                }

                GMInstructionData::DoubleType(GMDoubleTypeInstruction { type1, type2 })
            }

            GMOpcode::Compare => {
                // Parse instruction components from bytes
                let comparison_type_raw: u8 = if reader.general_info.bytecode_version <= 14 {
                    u8::from(opcode) - 0x10   // In bytecode 14, the comparison kind is encoded in the opcode itself
                } else {
                    b1
                };
                let comparison_type: GMComparisonType = num_enum_from(comparison_type_raw)
                    .map_err(|e| format!("{e} while parsing Comparison Instruction"))?;
                let type1: GMDataType = num_enum_from(b2 & 0xf)
                    .map_err(|e| format!("{e} for type1 while parsing Double Type Instruction"))?;
                let type2: GMDataType = num_enum_from(b2 >> 4).
                    map_err(|e| format!("{e} for type2 while parsing Double Type Instruction"))?;
                
                // short circuit stuff {~~}
                
                GMInstructionData::Comparison(GMComparisonInstruction { comparison_type, type1, type2 })
            }

            GMOpcode::Branch |
            GMOpcode::BranchIf |
            GMOpcode::BranchUnless |
            GMOpcode::PushWithContext |
            GMOpcode::PopWithContext => {
                if reader.general_info.bytecode_version <= 14 {
                    let jump_offset: i32 = b0 as i32 | ((b1 as u32) << 8) as i32 | ((b2 as i32) << 16);
                    let instr = if jump_offset == -1048576 {   // little endian [00 00 F0]
                        GMGotoInstruction { jump_offset: None }
                    } else {
                        GMGotoInstruction { jump_offset: Some(jump_offset) }
                    };
                    GMInstructionData::Goto(instr)
                } else {
                    let v: u32 = b0 as u32 | ((b1 as u32) << 8) | ((b2 as u32) << 16);      // i hate bitshifting
                    let popenv_exit_magic: bool = (v & 0x800000) != 0;
                    if popenv_exit_magic && v != 0xF00000 {
                        return Err("Popenv exit magic doesn't work while parsing Goto Instruction".to_string());
                    }
                    // "The rest is int23 signed value, so make sure" (<-- idk what this is supposed to mean)
                    let mut jump_offset: u32 = v & 0x003FFFFF;
                    if (v & 0x00C00000) != 0 {
                        jump_offset |= 0xFFC00000;
                    }
                    let instr = if popenv_exit_magic {
                        GMGotoInstruction { jump_offset: None }
                    } else {
                        GMGotoInstruction { jump_offset: Some(jump_offset as i32) }
                    };
                    GMInstructionData::Goto(instr)
                }
            }

            GMOpcode::Pop => {
                let type1: GMDataType = num_enum_from(b2 & 0xf)
                    .map_err(|e| format!("{e} for type1 while parsing Pop Type Instruction"))?;
                let type2: GMDataType = num_enum_from(b2 >> 4).
                    map_err(|e| format!("{e} for type2 while parsing Pop Type Instruction"))?;
                let raw_instance_type: i16 = b0 as i16 | ((b1 as i16) << 8);

                if type1 == GMDataType::Int16 {
                    // maybe b1 or b0 and b1 combined, idk
                    GMInstructionData::PopSwap(GMPopSwapInstruction { size: b0 })
                } else {
                    let destination: CodeVariable = read_variable(reader, raw_instance_type)?;
                    GMInstructionData::Pop(GMPopInstruction { type1, type2, destination })
                }
            }

            GMOpcode::Push |
            GMOpcode::PushLocal |
            GMOpcode::PushGlobal |
            GMOpcode::PushBuiltin |
            GMOpcode::PushImmediate => {
                let data_type: GMDataType = num_enum_from(b2).map_err(|e| format!("{e} while parsing Push Instruction"))?;
                let raw_instance_type: i16 = (b0 as i16) | ((b1 as i16) << 8);

                let value: GMCodeValue = if data_type == GMDataType::Variable {
                    let variable: CodeVariable = read_variable(reader, raw_instance_type)?;
                    GMCodeValue::Variable(variable)
                } else {
                    read_code_value(reader, data_type)?
                };

                GMInstructionData::Push(GMPushInstruction { value })
            },

            GMOpcode::Call => {
                let data_type: GMDataType = num_enum_from(b2).map_err(|e| format!("{e} while parsing Call Instruction"))?;
                let function: GMRef<GMFunction> = reader.function_occurrence_map.get(&reader.cur_pos).ok_or_else(|| format!(
                    "Could not find any function with absolute occurrence position {} in map with length {} while parsing Call Instruction",
                    reader.cur_pos, reader.function_occurrence_map.len(),
                ))?.clone();
                reader.cur_pos += 4;   // skip next occurrence offset
                
                GMInstructionData::Call(GMCallInstruction {
                    arguments_count: b0,
                    data_type,
                    function,
                })
            }

            GMOpcode::Extended => {
                let kind: i16 = b0 as i16 | ((b1 as i16) << 8);
                let data_type: GMDataType = num_enum_from(b2).map_err(|e| format!("{e} while parsing Break Instruction"))?;
                if data_type == GMDataType::Int32 {
                    if let Some(&function) = reader.function_occurrence_map.get(&reader.cur_pos) {
                        reader.cur_pos += 4;    // skip next occurrence offset
                        GMInstructionData::ExtendedFunc(GMExtendedInstructionFunction { kind, function })
                    } else {
                        let int_argument: i32 = reader.read_i32()?;
                        GMInstructionData::Extended32(GMExtendedInstruction32 { kind, int_argument })
                    }
                } else if data_type == GMDataType::Int16 {
                    GMInstructionData::Extended16(GMExtendedInstruction16 { kind })
                } else {
                    return Err(format!("Unexpected extended/break data type {data_type:?}"))
                }
            }
        };

        Ok(GMInstruction { opcode, kind: instruction_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let instr_abs_pos: usize = builder.len();
        let bytecode14: bool = builder.bytecode_version() <= 14;

        match &self.kind {
            GMInstructionData::Empty => {
                builder.write_i24(0);
                builder.write_u8(self.opcode.into());
            }

            GMInstructionData::SingleType(instr) => {
                let opcode_raw: u8 = if !bytecode14 { self.opcode.into() } else {
                    match self.opcode {
                        GMOpcode::Negate => 0x0D,
                        GMOpcode::Not => 0x0E,
                        GMOpcode::Duplicate => 0x82,
                        GMOpcode::Return => 0x9D,
                        GMOpcode::Exit => 0x9E,
                        GMOpcode::PopDiscard => 0x9F,
                        other => return Err(format!("Invalid Single Type Instruction opcode {other:?} while building instructions")),
                    }
                };
                builder.write_u16(0);
                builder.write_u8(instr.data_type.into());
                builder.write_u8(opcode_raw);
            }

            GMInstructionData::Duplicate(instr) => {
                builder.write_u8(instr.size);
                builder.write_u8(0);
                builder.write_u8(instr.data_type.into());
                builder.write_u8(self.opcode.into());
            }

            GMInstructionData::DuplicateSwap(instr) => {
                builder.write_u8(instr.size1);
                builder.write_u8(instr.size2);
                builder.write_u8(instr.data_type.into());
                builder.write_u8(self.opcode.into());
            }

            GMInstructionData::CallVariable(instr) => {
                builder.write_u8(0);
                builder.write_u8(instr.argument_count);
                builder.write_u8(instr.data_type.into());
                builder.write_u8(self.opcode.into());
            }

            GMInstructionData::DoubleType(instr) => {
                let opcode_raw: u8 = if !bytecode14 { self.opcode.into() } else {
                    match self.opcode {
                        GMOpcode::Convert => 0x03,
                        GMOpcode::Multiply => 0x04,
                        GMOpcode::Divide => 0x05,
                        GMOpcode::Remainder => 0x06,
                        GMOpcode::Modulus => 0x07,
                        GMOpcode::Add => 0x08,
                        GMOpcode::Subtract => 0x09,
                        GMOpcode::And => 0x0A,
                        GMOpcode::Or => 0x0B,
                        GMOpcode::Xor => 0x0C,
                        GMOpcode::ShiftLeft => 0x0F,
                        GMOpcode::ShiftRight => 0x10,
                        other => return Err(format!("Invalid Double Type Instruction opcode {other:?} while building instructions")),
                    }
                };
                let type1: u8 = instr.type1.into();
                let type2: u8 = instr.type2.into();

                builder.write_u8(0);
                builder.write_u8(0);
                builder.write_u8(type1 | type2 << 4);
                builder.write_u8(opcode_raw);
            }

            GMInstructionData::Comparison(instr) => {
                let opcode_raw: u8 = if bytecode14 {
                    u8::from(instr.comparison_type) + 0x10
                } else {
                    u8::from(self.opcode)     // always GMOpcode::Cmp
                };
                let type1: u8 = instr.type1.into();
                let type2: u8 = instr.type2.into();

                builder.write_u8(0);
                builder.write_u8(instr.comparison_type.into());
                builder.write_u8(type1 | type2 << 4);
                builder.write_u8(opcode_raw);
            }

            GMInstructionData::Goto(instr) => {
                let opcode_raw: u8 = if !bytecode14 { self.opcode.into() } else {
                    match self.opcode {
                        GMOpcode::Branch => 0xB7,
                        GMOpcode::BranchIf => 0xB8,
                        GMOpcode::BranchUnless => 0xB9,
                        GMOpcode::PushWithContext => 0xBB,
                        GMOpcode::PopWithContext => 0xBC,
                        other => return Err(format!("Invalid Goto Instruction opcode {other:?} while building instructions")),
                    }
                };

                if let Some(jump_offset) = instr.jump_offset {
                    builder.write_i24(jump_offset & 0x7fffff);
                } else {
                    // popenv exit magic
                    builder.write_i24(0xF00000);
                }

                builder.write_u8(opcode_raw);
            }

            GMInstructionData::Pop(instr) => {
                if instr.type1 == GMDataType::Int16 {
                    return Err("Int16 Data Type not yet supported while building Pop Instruction".to_string())
                }

                let opcode_raw: u8 = if !bytecode14 { self.opcode.into() } else {
                    match self.opcode {
                        GMOpcode::Pop => 0x41,
                        other => return Err(format!("Invalid Pop Instruction opcode {other:?} while building instructions")),
                    }
                };
                let type1: u8 = instr.type1.into();
                let type2: u8 = instr.type2.into();

                builder.write_i16(build_instance_type(&instr.destination.instance_type));
                builder.write_u8(type1 | type2 << 4);
                builder.write_u8(opcode_raw);

                let variable: &GMVariable = instr.destination.variable.resolve(&builder.gm_data.variables.variables)?;
                write_variable_occurrence(builder, instr.destination.variable.index, instr_abs_pos, variable.name.index, instr.destination.variable_type)?;
            }

            GMInstructionData::PopSwap(instr) => {
                builder.write_u8(instr.size);
                builder.write_u8(0);
                builder.write_u8(GMDataType::Int16.into());
                builder.write_u8(self.opcode.into());
            }

            GMInstructionData::Push(instr) => {
                // Write 16-bit integer, instance type, or empty data
                builder.write_i16(match &instr.value {
                    GMCodeValue::Int16(int16) => *int16,
                    GMCodeValue::Variable(variable) => build_instance_type(&variable.instance_type),
                    _ => 0
                });

                let data_type: GMDataType = get_data_type_from_value(&instr.value);
                builder.write_u8(data_type.into());
                
                if bytecode14 {
                    builder.write_u8(GMOpcode::Push.into());
                } else {
                    builder.write_u8(self.opcode.into());
                }

                match &instr.value {
                    GMCodeValue::Int16(_) => {}     // nothing because it was already written inside the instruction
                    GMCodeValue::Int32(int32) => builder.write_i32(*int32),
                    GMCodeValue::Int64(int64) => builder.write_i64(*int64),
                    GMCodeValue::Double(double) => builder.write_f64(*double),
                    GMCodeValue::Float(float) => builder.write_f32(*float),
                    GMCodeValue::Boolean(boolean) => builder.write_bool32(*boolean),
                    GMCodeValue::String(string_ref) => builder.write_u32(string_ref.index),
                    GMCodeValue::Variable(code_variable) => {
                        let variable: &GMVariable = code_variable.variable.resolve(&builder.gm_data.variables.variables)?;
                        write_variable_occurrence(builder, code_variable.variable.index, instr_abs_pos, variable.name.index, code_variable.variable_type)?;
                    }
                    GMCodeValue::Function(func_ref) => {
                        let function: &GMFunction = func_ref.resolve(&builder.gm_data.functions.functions)?;
                        write_function_occurrence(builder, func_ref.index, instr_abs_pos, function.name.index)?;
                    }
                }
            }

            GMInstructionData::Call(instr) => {
                builder.write_u8(instr.arguments_count);
                builder.write_u8(0);        // TODO check if writing zero is ok since b1 isn't checked or saved
                builder.write_u8(instr.data_type.into());
                builder.write_u8(self.opcode.into());

                let function: &GMFunction = instr.function.resolve(&builder.gm_data.functions.functions)?;
                write_function_occurrence(builder, instr.function.index, instr_abs_pos, function.name.index)?;
            }

            GMInstructionData::Extended16(instr) => {
                builder.write_i16(instr.kind);
                builder.write_u8(GMDataType::Int16.into());
                builder.write_u8(self.opcode.into());
            }

            GMInstructionData::Extended32(instr) => {
                builder.write_i16(instr.kind);
                builder.write_u8(GMDataType::Int32.into());
                builder.write_u8(self.opcode.into());
                builder.write_i32(instr.int_argument);
            }

            GMInstructionData::ExtendedFunc(instr) => {
                builder.write_i16(instr.kind);
                builder.write_u8(GMDataType::Int32.into());
                builder.write_u8(self.opcode.into());
                let function: &GMFunction = instr.function.resolve(&builder.gm_data.functions.functions)?;
                write_function_occurrence(builder, instr.function.index, instr_abs_pos, function.name.index)?;
            }
        }

        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMOpcode {
    /// Converts the top of the stack from one type to another.
    Convert = 0x07,
    
    /// Pops two values from the stack, multiplies them, and pushes the result.
    Multiply = 0x08,

    /// Pops two values from the stack, divides them, and pushes the result.
    /// The second popped value is divided by the first popped value.
    Divide = 0x09,
    
    /// Pops two values from the stack, performs a GML `div` operation (division with remainder), and pushes the result.
    /// The second popped value is divided (with remainder) by the first popped value.
    Remainder = 0x0A,
    
    /// Pops two values from the stack, performs a GML `mod` operation (`%`), and pushes the result.
    /// The second popped value is modulo'd against the first popped value.
    Modulus = 0x0B,
    
    /// Pops two values from the stack, adds them, and pushes the result.
    Add = 0x0C,

    /// Pops two values from the stack, **subtracts** them, and pushes the result.
    /// The second popped value is subtracted by the first popped value.
    Subtract = 0x0D,
    
    /// Pops two values from the stack, performs an **AND** operation, and pushes the result.
    /// This can be done bitwise or logically.
    And = 0x0E,

    /// Pops two values from the stack, performs an **OR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Or = 0x0F,

    /// Pops two values from the stack, performs an **XOR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Xor = 0x10,

    /// Negates the top value of the stack (as in, multiplies it with negative one).
    Negate = 0x11,
    
    /// Performs a boolean or bitwise NOT operation on the top value of the stack (modifying it).
    Not = 0x12,
    
    /// Pops two values from the stack, performs a bitwise left shift operation (`<<`), and pushes the result.
    /// The second popped value is shifted left by the first popped value.
    ShiftLeft = 0x13,

    /// Pops two values from the stack, performs a bitwise right shift operation (`>>`), and pushes the result.
    /// The second popped value is shifted right by the first popped value.
    ShiftRight = 0x14,
    
    /// Pops two values from the stack, compares them using a [`GMComparisonType`], and pushes a boolean result.
    Compare = 0x15,

    /// Pops a value from the stack, and generally stores it in a variable, array, or otherwise.
    /// Has an alternate mode that can swap values around on the stack.
    Pop = 0x45,
    
    /// Duplicates values on the stack, or swaps them around ("dup swap" mode).
    /// Behavior depends on instruction parameters, both in data sizes and mode.
    Duplicate = 0x86,

    /// Pops a value from the stack, and returns from the current function/script with that value as the return value.
    Return = 0x9C,

    /// Returns from the current function/script/event with no return value.
    Exit = 0x9D,

    /// Pops a value from the stack, and discards it.
    PopDiscard = 0x9E,
    
    /// Branches (jumps) to another instruction in the code entry.
    Branch = 0xB6,
    
    /// Pops a boolean/int32 value from the stack. If true/nonzero, branches (jumps) to another instruction in the code entry.
    BranchIf = 0xB7,

    /// Pops a boolean/int32 value from the stack. If false/zero, branches (jumps) to another instruction in the code entry.
    BranchUnless = 0xB8,

    /// Pushes a `with` context, used for GML `with` statements, to the VM environment/self instance stack.
    PushWithContext = 0xBA,

    /// Pops/ends a `with` context, used for GML `with` statements, from the VM environment/self instance stack.
    /// This instruction will branch to its encoded address until no longer iterating instances, where the context will finally be gone for good.
    /// If a flag is encoded in this instruction, then this will always terminate the loop, and branch to the encoded address.
    PopWithContext = 0xBB,
    
    /// Pushes a constant value onto the stack. Can vary in size depending on value type.
    Push = 0xC0,

    /// Pushes a value stored in a local variable onto the stack.
    PushLocal = 0xC1,

    /// Pushes a value stored in a global variable onto the stack.
    PushGlobal = 0xC2,

    /// Pushes a value stored in a GameMaker builtin variable onto the stack.
    PushBuiltin = 0xC3,

    /// Pushes an immediate signed 32-bit integer value onto the stack, encoded as a signed 16-bit integer.
    PushImmediate = 0x84,

    /// Calls a GML script/function, using its ID. Arguments are prepared prior to this instruction, in reverse order.
    /// Argument count is encoded in this instruction. Arguments are popped off of the stack.
    Call = 0xD9,

    /// Pops two values off of the stack, and then calls a GML script/function using those values, representing
    /// the "self" instance to be used when calling, as well as the reference to the function being called. 
    /// Arguments are dealt with identically to "call".
    CallVariable = 0x99,

    /// Performs extended operations that are not detailed anywhere.
    Extended = 0xFF,
}
impl GMOpcode {
    fn convert_bytecode14(raw_opcode: u8) -> u8 {
        match raw_opcode {
            0x03 => 0x07,
            0x04 => 0x08,
            0x05 => 0x09,
            0x06 => 0x0A,
            0x07 => 0x0B,
            0x08 => 0x0C,
            0x09 => 0x0D,
            0x0A => 0x0E,
            0x0B => 0x0F,
            0x0C => 0x10,
            0x0D => 0x11,
            0x0E => 0x12,
            0x0F => 0x13,
            0x10 => 0x14,
            0x11 | 0x12 | 0x13 | 0x14 | 0x16 => 0x15,
            0xDA => 0xD9,
            0x41 => 0x45,
            0x82 => 0x86,
            0xB7 => 0xB6,
            0xB8 => 0xB7,
            0xB9 => 0xB8,
            0x9D => 0x9C,
            0x9E => 0x9D,
            0x9F => 0x9E,
            0xBB => 0xBA,
            0xBC => 0xBB,
            _ => raw_opcode,
        }
    }
}
impl GMElement for GMOpcode {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let mut raw: u8 = reader.read_u8()?;
        if reader.general_info.bytecode_version <= 14 {
            raw = Self::convert_bytecode14(raw);
        }
        Self::try_from(raw).map_err(|_| format!("Invalid Opcode 0x{raw:02X}"))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u8(u8::from(*self));
        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMSingleTypeInstruction {
    pub data_type: GMDataType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDuplicateInstruction {
    pub data_type: GMDataType,
    pub size: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDuplicateSwapInstruction {
    pub data_type: GMDataType,
    pub size1: u8,
    pub size2: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMCallVariableInstruction {
    pub data_type: GMDataType,
    pub argument_count: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDoubleTypeInstruction {
    pub type1: GMDataType,
    pub type2: GMDataType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMComparisonInstruction {
    pub comparison_type: GMComparisonType,
    pub type1: GMDataType,
    pub type2: GMDataType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMGotoInstruction {
    /// Contains the offset of where to jump. 1 = 4 bytes. Can be negative.
    /// If [`None`], it is a "popenv exit magic" goto instruction.
    pub jump_offset: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMPopInstruction {
    pub type1: GMDataType,
    pub type2: GMDataType,
    pub destination: CodeVariable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMPopSwapInstruction {
    pub size: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMPushInstruction {
    pub value: GMCodeValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCallInstruction {
    pub arguments_count: u8,
    pub data_type: GMDataType,
    pub function: GMRef<GMFunction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMExtendedInstruction16 {
    pub kind: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMExtendedInstruction32 {
    pub kind: i16,
    pub int_argument: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMExtendedInstructionFunction {
    pub kind: i16,
    pub function: GMRef<GMFunction>,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMDataType {
    Double,
    Float,
    Int32,
    Int64,
    Boolean,
    Variable,
    String,
    Int16 = 0x0f,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMInstanceType {
    Undefined,
    
    /// Represents the current chunk instance.
    Self_(Option<GMRef<GMGameObject>>),
    
    /// Instance ID in the Room -100000; used when the Variable Type is [`GMVariableType::Instance`].
    /// This doesn't exist in UTMT.
    RoomInstance(i16),
    
    /// Represents the other context, which has multiple definitions based on the location used.
    Other,
    
    /// Represents all active object instances. Assignment operations can perform a loop.
    All,
    
    /// Represents no object/instance.
    None,
    
    /// Used for global variables.
    Global,
    
    /// Used for GML built-in variables.
    Builtin,
    
    /// Used for local variables; local to their code script.
    Local,
    
    /// Instance is stored in a Variable data type on the top of the stack.
    StackTop,
    
    /// Used for function argument variables in GMLv2 (GMS 2.3).
    Argument,
    
    /// Used for static variables.
    Static,
}

impl Display for GMInstanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GMInstanceType::Undefined => write!(f, "Undefined"),
            GMInstanceType::Self_(None) => write!(f, "Self"),
            GMInstanceType::Self_(Some(reference)) => write!(f, "Self<{}>", reference.index),
            GMInstanceType::RoomInstance(instance_id) => write!(f, "RoomInstanceID<{instance_id}>"),
            GMInstanceType::Other => write!(f, "Other"),
            GMInstanceType::All => write!(f, "All"),
            GMInstanceType::None => write!(f, "None"),
            GMInstanceType::Global => write!(f, "Global"),
            GMInstanceType::Builtin => write!(f, "Builtin"),
            GMInstanceType::Local => write!(f, "Local"),
            GMInstanceType::StackTop => write!(f, "StackTop"),
            GMInstanceType::Argument => write!(f, "Argument"),
            GMInstanceType::Static => write!(f, "Static"),
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMVariableType {
    /// Used for normal single-dimension array variables
    Array = 0x00,
    
    /// Used when referencing a variable on another variable, e.g. a chain referenc
    StackTop = 0x80,
    
    /// normal
    Normal = 0xA0,
    
    /// Used when referencing variables on room instance IDs, e.g. something like "inst_01ABCDEF.x" in GML
    Instance = 0xE0,
    
    /// GMS2.3+, multidimensional array with pushaf
    ArrayPushAF = 0x10,
    
    /// GMS2.3+, multidimensional array with pushaf or popaf
    ArrayPopAF = 0x90,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMComparisonType {
    /// "Less than" | `<`
    LessThan = 1,
    
    /// "Less than or equal to" | `<=`
    LessOrEqual = 2,
    
    /// "Equal to" | `==`
    Equal = 3,

    /// "Not equal to" | `!=`
    NotEqual = 4,

    /// "Greater than or equal to" | `>=`
    GreaterOrEqual = 5,

    /// "Greater than" | `>`
    GreaterThan = 6,
}


#[derive(Debug, Clone, PartialEq)]
pub struct CodeVariable {
    pub variable: GMRef<GMVariable>,
    pub variable_type: GMVariableType,
    pub instance_type: GMInstanceType,
    pub is_int32: bool,
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMCodeValue {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Double(f64),
    Float(f32),
    Boolean(bool),
    String(GMRef<String>),
    Variable(CodeVariable),
    /// Does not exist in UTMT. Added in order to support inline/anonymous functions.
    Function(GMRef<GMFunction>),
}


fn read_code_value(reader: &mut DataReader, data_type: GMDataType) -> Result<GMCodeValue, String> {
    match data_type {
        GMDataType::Double => reader.read_f64().map(GMCodeValue::Double),
        GMDataType::Float => reader.read_f32().map(GMCodeValue::Float),
        GMDataType::Int32 => {
            if let Some(&function) = reader.function_occurrence_map.get(&reader.cur_pos) {
                reader.cur_pos += 4;    // skip next occurrence offset
                return Ok(GMCodeValue::Function(function.clone()))
            }

            if let Some(&variable) = reader.variable_occurrence_map.get(&reader.cur_pos) {
                reader.cur_pos += 4;    // skip next occurrence offset
                return Ok(GMCodeValue::Variable(CodeVariable {
                    variable,
                    variable_type: GMVariableType::Normal,
                    instance_type: GMInstanceType::Undefined,
                    is_int32: true,
                }))
            }

            reader.read_i32().map(GMCodeValue::Int32)
        }
        GMDataType::Int64 => reader.read_i64().map(GMCodeValue::Int64),
        GMDataType::Boolean => reader.read_bool32().map(GMCodeValue::Boolean),
        GMDataType::String => reader.read_resource_by_id().map(GMCodeValue::String),
        GMDataType::Int16 => {
            // int16 in embedded in the instruction itself
            reader.cur_pos -= 4;
            let number: i16 = reader.read_i16()?;
            reader.cur_pos += 2;
            Ok(GMCodeValue::Int16(number))
        }
        other => Err(format!("Trying to read unsupported data type {other:?} while reading value in code at absolute position {}", reader.cur_pos)),
    }
}


fn read_variable(reader: &mut DataReader, raw_instance_type: i16) -> Result<CodeVariable, String> {
    let occurrence_position: usize = reader.cur_pos;
    let raw_value: i32 = reader.read_i32()?;

    let variable_type: i32 = (raw_value >> 24) & 0xF8;
    let variable_type: GMVariableType = num_enum_from(variable_type as u8)
        .map_err(|e| format!("{e} while parsing variable reference chain"))?;

    let instance_type: GMInstanceType = parse_instance_type(raw_instance_type, variable_type)?;

    let variable: GMRef<GMVariable> = reader.variable_occurrence_map.get(&occurrence_position)
        .ok_or_else(|| format!(
            "Could not find {} Variable with occurrence position {} in hashmap with length {} while parsing code value",
            instance_type, occurrence_position, reader.variable_occurrence_map.len(),
        ))?.clone();

    Ok(CodeVariable { variable, variable_type, instance_type, is_int32: false })
}


pub fn parse_instance_type(raw_value: i16, variable_type: GMVariableType) -> Result<GMInstanceType, String> {
    // If > 0; then game object id (or room instance id). If < 0, then variable instance type.
    if raw_value > 0 {
        return Ok(if variable_type == GMVariableType::Instance {
            GMInstanceType::RoomInstance(raw_value)
        } else {
            GMInstanceType::Self_(Some(GMRef::new(raw_value as u32)))
        });
    }

    let instance_type: GMInstanceType = match raw_value {
        0 => GMInstanceType::Undefined,
        -1 => GMInstanceType::Self_(None),
        -2 => GMInstanceType::Other,
        -3 => GMInstanceType::All,
        -4 => GMInstanceType::None,
        -5 => GMInstanceType::Global,
        -6 => GMInstanceType::Builtin,
        -7 => GMInstanceType::Local,
        -9 => GMInstanceType::StackTop,
        -15 => GMInstanceType::Argument,
        -16 => GMInstanceType::Static,
        _ => return Err(format!("Invalid instance type {raw_value} (0x{raw_value:04X})"))
    };

    Ok(instance_type)
}

pub fn build_instance_type(instance_type: &GMInstanceType) -> i16 {
    // If > 0; then game object id. If < 0, then variable instance type.
    match instance_type {
        GMInstanceType::Undefined => 0,
        GMInstanceType::Self_(None) => -1,
        GMInstanceType::Self_(Some(game_object_ref)) => game_object_ref.index as i16,
        GMInstanceType::RoomInstance(instance_id) => *instance_id,
        GMInstanceType::Other => -2,
        GMInstanceType::All => -3,
        GMInstanceType::None => -4,
        GMInstanceType::Global => -5,
        GMInstanceType::Builtin => -6,
        GMInstanceType::Local => -7,
        GMInstanceType::StackTop => -9,
        GMInstanceType::Argument => -15,
        GMInstanceType::Static => -16,
    }
}


fn write_variable_occurrence(
    builder: &mut DataBuilder,
    gm_index: u32,
    occurrence_position: usize,
    name_string_id: u32,
    variable_type: GMVariableType,
) -> Result<(), String> {
    let occurrence_map_len: usize = builder.variable_occurrences.len();   // prevent double borrow on error message
    let occurrences: &mut Vec<(usize, GMVariableType)> = builder.variable_occurrences.get_mut(gm_index as usize).ok_or_else(|| format!(
        "Trying to get inner variable occurrences vec out of bounds while writing occurrence: {} >= {}",
        gm_index, occurrence_map_len,
    ))?;

    if let Some((last_occurrence_pos, old_variable_type)) = occurrences.last().cloned() {
        // replace last occurrence (which is name string id) with next occurrence offset
        let occurrence_offset: i32 = occurrence_position as i32 - last_occurrence_pos as i32;
        let occurrence_offset_full: i32 = occurrence_offset & 0x07FFFFFF | (((u8::from(old_variable_type) & 0xF8) as i32) << 24);
        builder.overwrite_i32(occurrence_offset_full, last_occurrence_pos+4)?;
    }

    // write name string id for this occurrence. this is correct if it is the last occurrence.
    // otherwise, it will be overwritten later by the code above.
    // hopefully, writing the name string id instead of -1 for unused variables will be fine.
    builder.write_u32(name_string_id & 0x07FFFFFF | (((u8::from(variable_type) & 0xF8) as u32) << 24));

    // fuckass borrow checker
    builder.variable_occurrences.get_mut(gm_index as usize).unwrap().push((occurrence_position, variable_type));
    Ok(())
}


fn write_function_occurrence(builder: &mut DataBuilder, gm_index: u32, occurrence_position: usize, name_string_id: u32) -> Result<(), String> {
    let occurrence_map_len: usize = builder.function_occurrences.len();   // prevent double borrow on error message
    let occurrences: &mut Vec<usize> = builder.function_occurrences.get_mut(gm_index as usize).ok_or_else(|| format!(
        "Trying to get inner function occurrences vec out of bounds while writing occurrence: {} >= {}",
        gm_index, occurrence_map_len,
    ))?;

    if let Some(last_occurrence_pos) = occurrences.last().cloned() {
        // replace last occurrence (which is name string id) with next occurrence offset
        let occurrence_offset: i32 = occurrence_position as i32 - last_occurrence_pos as i32;
        builder.overwrite_i32(occurrence_offset & 0x07FFFFFF, last_occurrence_pos+4)?;
    }

    // write name string id for this occurrence. this is correct if it is the last occurrence.
    // otherwise, it will be overwritten later by the code above.
    builder.write_u32(name_string_id & 0x07FFFFFF);

    builder.function_occurrences.get_mut(gm_index as usize).unwrap().push(occurrence_position);
    Ok(())
}


pub fn get_data_type_from_value(code_value: &GMCodeValue) -> GMDataType {
    match code_value {
        GMCodeValue::Int16(_) => GMDataType::Int16,
        GMCodeValue::Int32(_) => GMDataType::Int32,
        GMCodeValue::Function(_) => GMDataType::Int32,  // functions are not a "real" gm type; they're always int32
        GMCodeValue::Variable(var) if var.is_int32 => GMDataType::Int32,    // idk when this happens
        GMCodeValue::Int64(_) => GMDataType::Int64,
        GMCodeValue::Float(_) => GMDataType::Float,
        GMCodeValue::Double(_) => GMDataType::Double,
        GMCodeValue::Boolean(_) => GMDataType::Boolean,
        GMCodeValue::String(_) => GMDataType::String,
        GMCodeValue::Variable(_) => GMDataType::Variable,
    }
}


/// Check whether this data file was generated with YYC (YoYoGames Compiler).
/// Should that be the case, the CODE, VARI and FUNC chunks will be empty (or not exist?).
/// NOTE: YYC is untested. Issues may occur.
pub fn check_yyc(reader: &DataReader) -> bool {
    let Some(chunk_code) = reader.chunks.get("CODE") else {return true};
    let Some(chunk_vari) = reader.chunks.get("VARI") else {return true};
    let Some(chunk_func) = reader.chunks.get("FUNC") else {return true};
    chunk_code.end_pos <= chunk_code.start_pos &&
        chunk_vari.end_pos <= chunk_vari.start_pos &&
        chunk_func.end_pos <= chunk_func.start_pos
}

