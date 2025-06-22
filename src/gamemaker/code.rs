use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::variables::{GMVariable, GMVariableB15Data};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::functions::GMFunction;
use crate::gamemaker::game_objects::GMGameObject;
use crate::gm_serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMCodes {
    pub codes: Vec<GMCode>,
    pub exists: bool,
}
impl GMChunkElement for GMCodes {
    fn empty() -> Self {
        Self { codes: vec![], exists: false }
    }
}
impl GMElement for GMCodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let codes: Vec<GMCode> = reader.read_pointer_list()?;
        Ok(GMCodes { codes, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }

        if builder.bytecode_version() >= 15 {
            // in bytecode 15, the codes' instructions are written before the codes metadata
            let mut instructions_start_positions: Vec<usize> = Vec::with_capacity(self.codes.len());
            let mut instructions_end_positions: Vec<usize> = Vec::with_capacity(self.codes.len());

            for (i, code) in self.codes.iter().enumerate() {
                instructions_start_positions.push(builder.len());
                for instruction in &code.instructions {
                    instruction.serialize(builder).map_err(|e| format!(
                        "{e}\n↳ while serializing bytecode15 code #{i} with name \"{}\"",
                        builder.display_gm_str(&code.name),
                    ))?;
                }
                instructions_end_positions.push(builder.len());
            }

            for (i, code) in self.codes.iter().enumerate() {
                builder.resolve_pointer(self)?;
                let b15_info: &GMCodeBytecode15 = code.bytecode15_info.as_ref()
                    .ok_or_else(|| format!("Code bytecode 15 data not set in Bytecode version {}", builder.bytecode_version()))?;
                let length = instructions_end_positions[i] - instructions_start_positions[i];
                builder.write_usize(length)?;
                builder.write_u16(b15_info.locals_count);
                builder.write_u16(b15_info.arguments_count | if b15_info.weird_local_flag {0x8000} else {0});
                builder.write_usize(instructions_start_positions[i] - builder.len())?;  // bytecode relative address
                builder.write_usize(b15_info.offset)?;
            }
        } else {  // bytecode 14

        }
        builder.write_pointer_list(&self.codes)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCode {
    pub name: GMRef<String>,
    pub instructions: Vec<GMInstruction>,
    pub bytecode15_info: Option<GMCodeBytecode15>,
}
impl GMElement for GMCode {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let code_length: usize = reader.read_usize()?;

        let (end_pos, bytecode15_info): (usize, Option<GMCodeBytecode15>) = if reader.general_info.bytecode_version <= 14 {
            (reader.cur_pos + code_length, None)
        } else {
            let b15_info = GMCodeBytecode15::deserialize(reader)?;
            reader.cur_pos = b15_info.bytecode_start_address;
            (b15_info.bytecode_start_address + code_length, Some(b15_info))
        };

        let mut instructions: Vec<GMInstruction> = Vec::with_capacity(code_length * 5);  // estimate
        let start_pos: usize = reader.cur_pos;
        while reader.cur_pos < end_pos {
            let instruction = GMInstruction::deserialize(reader).map_err(|e| format!(
                "{e} for Instruction #{} (at absolute position {}) of Code entry \"{}\" with absolute start position {}",
                instructions.len(), reader.cur_pos, reader.display_gm_str(name), start_pos,
            ))?;
            instructions.push(instruction);
        }

        instructions.shrink_to_fit();
        Ok(GMCode { name, instructions, bytecode15_info })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if builder.bytecode_version() > 14 {
            return Err(format!(
                "[internal error] GMCode::serialize is only meant to be called for bytecode14; data has bytecode version {}",
                builder.bytecode_version(),
            ))
        }
        builder.resolve_pointer(self)?;
        builder.write_gm_string(&self.name)?;
        let length_placeholder_pos: usize = builder.len();
        builder.write_u32(0xDEADC0DE);
        let start: usize = builder.len();

        // in bytecode 14, instructions are written immediately
        for (i, instruction) in self.instructions.iter().enumerate() {
            instruction.serialize(builder).map_err(|e| format!(
                "{e}\n↳ while building bytecode14 code #{i} with name \"{}\"",
                builder.display_gm_str(&self.name),
            ))?;
        }

        let code_length: i32 = builder.len() as i32 - start as i32;
        builder.overwrite_i32(code_length, length_placeholder_pos)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeBytecode15 {
    pub locals_count: u16,
    pub arguments_count: u16,
    pub weird_local_flag: bool,
    pub offset: usize,
    /// only exists for deserializing code
    bytecode_start_address: usize,
}
impl GMElement for GMCodeBytecode15 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let locals_count: u16 = reader.read_u16()?;
        let arguments_count_raw: u16 = reader.read_u16()?;
        let arguments_count: u16 = arguments_count_raw & 0x7FFF;
        let weird_local_flag: bool = arguments_count_raw & 0x8000 != 0;

        let bytecode_relative_address: i32 = reader.read_i32()?;
        let bytecode_start_address: usize = (bytecode_relative_address + reader.cur_pos as i32 - 4) as usize;

        let offset: usize = reader.read_usize()?;
        // child check {~~}

        Ok(GMCodeBytecode15 { locals_count, arguments_count, weird_local_flag, offset, bytecode_start_address })
    }

    fn serialize(&self, _builder: &mut DataBuilder) -> Result<(), String> {
        unreachable!("[internal error] GMCodeBytecode15::serialize is not supported; use GMCodes::serialize instead")
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMInstructionData {
    SingleType(GMSingleTypeInstruction),
    DoubleType(GMDoubleTypeInstruction),
    Comparison(GMComparisonInstruction),
    Goto(GMGotoInstruction),
    Pop(GMPopInstruction),
    Push(GMPushInstruction),
    Call(GMCallInstruction),
    Break(GMBreakInstruction),
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
        let mut opcode = GMOpcode::deserialize(reader)?;

       let instruction_data: GMInstructionData = match opcode {
            GMOpcode::Neg |
            GMOpcode::Not |
            GMOpcode::Dup |
            GMOpcode::Ret |
            GMOpcode::Exit |
            GMOpcode::Popz |
            GMOpcode::CallV => {
                let data_type: u8 = b2 & 0xf;
                let data_type: GMDataType = data_type.try_into().map_err(|_| format!(
                    "Invalid Data Type {data_type:02X} while parsing Single Type Instruction"
                ))?;

                // Ensure basic conditions hold
                if b0 != 0 && !matches!(opcode, GMOpcode::Dup | GMOpcode::CallV) {
                    return Err(format!("Invalid padding {:02X} while parsing Single Type Instruction", b0));
                }
                if b2 >> 4 != 0 {
                    return Err(format!("Second type should be zero but is {0} (0x{0:02X}) for Single Type Instruction", b2 >> 4))
                }

                GMInstructionData::SingleType(GMSingleTypeInstruction { extra: b0, data_type })
            }

            GMOpcode::Conv |
            GMOpcode::Mul |
            GMOpcode::Div |
            GMOpcode::Rem |
            GMOpcode::Mod |
            GMOpcode::Add |
            GMOpcode::Sub |
            GMOpcode::And |
            GMOpcode::Or |
            GMOpcode::Xor |
            GMOpcode::Shl |
            GMOpcode::Shr => {
                let type1: u8 = b2 & 0xf;
                let type1: GMDataType = type1.try_into().map_err(|_| format!(
                    "Invalid Data Type {type1:02X} while parsing Double Type Instruction"
                ))?;

                let type2: u8 = b2 >> 4;
                let type2: GMDataType = type2.try_into().map_err(|_| format!(
                    "Invalid Data Type {type2:02X} while parsing Double Type Instruction"
                ))?;

                if b1 != 0 {    // might be incorrect; remove if issues
                    return Err(format!("b1 should be zero but is {b1} (0x{b1:02X}) for Double Type Instruction"))
                }

                GMInstructionData::DoubleType(GMDoubleTypeInstruction { type1, type2 })
            }

            GMOpcode::Cmp => {
                // Parse instruction components from bytes
                let mut comparison_type: GMComparisonType = b1.try_into().map_err(|_| format!(
                    "Invalid Comparison Type {b1:02X} while parsing Comparison Instruction"
                ))?;    // TODO probably doesn't work for bytecode14; needs to be checked before

                let type1: u8 = b2 & 0xf;
                let type1: GMDataType = type1.try_into().map_err(|_| format!(
                    "Invalid Data Type {type1:02X} while parsing Comparison Instruction"
                ))?;

                let type2: u8 = b2 >> 4;
                let type2: GMDataType = type2.try_into().map_err(|_| format!(
                    "Invalid Data Type {type2:02X} while parsing Comparison Instruction"
                ))?;

                if reader.general_info.bytecode_version <= 14 {
                    // in bytecode14, the comparison kind is encoded in the opcode
                    let comparison_type_raw: u8 = u8::from(opcode) - 0x10;
                    comparison_type = comparison_type_raw.try_into().map_err(|_| format!(
                        "Invalid Bytecode14 Comparison Type {:02X} (Opcode {:?} = 0x{:02X}) while parsing Comparison Instruction",
                        comparison_type_raw, opcode, u8::from(opcode),
                    ))?;
                }
                // short circuit stuff {~~}
                
                GMInstructionData::Comparison(GMComparisonInstruction { comparison_type, type1, type2 })
            }

            GMOpcode::B |
            GMOpcode::Bt |
            GMOpcode::Bf |
            GMOpcode::PushEnv |
            GMOpcode::PopEnv => {
                if reader.general_info.bytecode_version <= 14 {
                    let jump_offset: i32 = b0 as i32 | ((b1 as u32) << 8) as i32 | ((b2 as i32) << 16);     // yeah idk
                    let popenv_exit_magic: bool = jump_offset == -1048576;      // little endian [00 00 F0]
                    GMInstructionData::Goto(GMGotoInstruction {
                        jump_offset,
                        popenv_exit_magic,
                    })
                } else {
                    let v: u32 = b0 as u32 | ((b1 as u32) << 8) | ((b2 as u32) << 16);      // i hate bitshifting
                    let popenv_exit_magic: bool = (v & 0x800000) != 0;
                    if popenv_exit_magic && v != 0xF00000 {
                        return Err("\"Popenv magic doesn't work\" while parsing Goto Instruction".to_string());
                    }
                    // The rest is int23 signed value, so make sure (<-- idk what this is supposed to mean)
                    let mut jump_offset: u32 = v & 0x003FFFFF;
                    if (v & 0x00C00000) != 0 {
                        jump_offset |= 0xFFC00000;
                    }
                    let jump_offset: i32 = jump_offset as i32;
                    GMInstructionData::Goto(GMGotoInstruction { jump_offset, popenv_exit_magic })
                }
            }

            GMOpcode::Pop => {
                let type1: u8 = b2 & 0xf;
                let type2: u8 = b2 >> 4;
                let instance_type: i16 = b0 as i16 | ((b1 as i16) << 8);

                let type1: GMDataType = type1.try_into().map_err(|_| format!("Invalid Data Type 1 {type1:02X} while parsing Pop Instruction"))?;
                let type2: GMDataType = type2.try_into().map_err(|_| format!("Invalid Data Type 2 {type2:02X} while parsing Pop Instruction"))?;
                let instance_type: GMInstanceType = parse_instance_type(instance_type)?;

                if type1 == GMDataType::Int16 {
                    return Err(format!(
                        "[Internal Error] Unhandled \"Special swap instruction\" (UndertaleModTool/Issues/#129) \
                        occurred at absolute position {} while parsing Pop Instruction. \
                        Please report this error to https://github.com/BioTomateDE/LibGM/issues",
                        reader.cur_pos,
                    ));
                }

                let destination: GMCodeVariable = read_variable(reader, &instance_type)?;
                GMInstructionData::Pop(GMPopInstruction {
                    instance_type,
                    type1,
                    type2,
                    destination,
                })
            }

            GMOpcode::Push |
            GMOpcode::PushLoc |
            GMOpcode::PushGlb |
            GMOpcode::PushBltn |
            GMOpcode::PushI => {
                let data_type: u8 = b2;
                let data_type: GMDataType = data_type.try_into()
                    .map_err(|_| format!("Invalid Data Type {data_type:02X} while parsing Push Instruction"))?;

                let val: i16 = (b0 as i16) | ((b1 as i16) << 8);

                if reader.general_info.bytecode_version <= 14 {
                    match data_type {
                        GMDataType::Int16 => opcode = GMOpcode::PushI,
                        GMDataType::Variable => {
                            match val {
                                -5 => opcode = GMOpcode::PushGlb,
                                -6 => opcode = GMOpcode::PushBltn,
                                -7 => opcode = GMOpcode::PushLoc,
                                _ => ()
                            }
                        },
                        _ => ()
                    }
                }

                let value: GMValue = if data_type == GMDataType::Variable {
                    let instance_type: GMInstanceType = parse_instance_type(val)?;
                    let variable: GMCodeVariable = read_variable(reader, &instance_type)?;
                    GMValue::Variable(variable)
                } else {
                    read_code_value(reader, data_type)?
                };

                GMInstructionData::Push(GMPushInstruction {
                    data_type,
                    value,
                })
            }

            GMOpcode::Call => {
                let data_type: u8 = b2;
                let data_type: GMDataType = data_type.try_into().map_err(|_| format!("Invalid Data Type {data_type:02X} while parsing Call Instruction"))?;

                let function: GMRef<GMFunction> = reader.function_occurrence_map.get(&reader.cur_pos).ok_or_else(|| format!(
                    "Could not find any function with absolute occurrence position {} in map with length {} while parsing Call Instruction",
                    reader.cur_pos, reader.function_occurrence_map.len(),
                ))?.clone();
                reader.skip_bytes(4);   // skip next occurrence offset
                
                GMInstructionData::Call(GMCallInstruction {
                    arguments_count: b0,
                    data_type,
                    function,
                })
            }

            GMOpcode::Break => {
                let value: i16 = b0 as i16 | ((b1 as i16) << 8);
                let data_type: u8 = b2;
                let data_type: GMDataType = data_type.try_into().map_err(|_| format!("Invalid Data Type {data_type:02X} while parsing Break Instruction"))?;

                let int_argument: Option<i32> = if data_type == GMDataType::Int32 {
                    // reader.general_info.set_version_at_least(2023, 8, 0, 0, None)?;
                    Some(reader.read_i32()?)
                } else {
                    None
                };

                // other set gms version stuff {~~}

                GMInstructionData::Break(GMBreakInstruction {
                    value,
                    data_type,
                    int_argument,
                })
            } 
       };
       
       Ok(GMInstruction { opcode, kind: instruction_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let instr_abs_pos: usize = builder.len();
        let bytecode14: bool = builder.bytecode_version() <= 14;

        match &self.kind {
            GMInstructionData::SingleType(instr) => {
                let opcode_raw: u8 = if !bytecode14 { self.opcode.into() } else {
                    match self.opcode {
                        GMOpcode::Neg => 0x0D,
                        GMOpcode::Not => 0x0E,
                        GMOpcode::Dup => 0x82,
                        GMOpcode::Ret => 0x9D,
                        GMOpcode::Exit => 0x9E,
                        GMOpcode::Popz => 0x9F,
                        other => return Err(format!("Invalid Single Type Instruction opcode {other:?} while building instructions")),
                    }
                };
                builder.write_u8(instr.extra);
                builder.write_u8(0);
                builder.write_u8(instr.data_type.into());
                builder.write_u8(opcode_raw);
            }

            GMInstructionData::DoubleType(instr) => {
                let opcode_raw: u8 = if !bytecode14 { self.opcode.into() } else {
                    match self.opcode {
                        GMOpcode::Conv => 0x03,
                        GMOpcode::Mul => 0x04,
                        GMOpcode::Div => 0x05,
                        GMOpcode::Rem => 0x06,
                        GMOpcode::Mod => 0x07,
                        GMOpcode::Add => 0x08,
                        GMOpcode::Sub => 0x09,
                        GMOpcode::And => 0x0A,
                        GMOpcode::Or => 0x0B,
                        GMOpcode::Xor => 0x0C,
                        GMOpcode::Shl => 0x0F,
                        GMOpcode::Shr => 0x10,
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
                        GMOpcode::B => 0xB7,
                        GMOpcode::Bt => 0xB8,
                        GMOpcode::Bf => 0xB9,
                        GMOpcode::PushEnv => 0xBB,
                        GMOpcode::PopEnv => 0xBC,
                        other => return Err(format!("Invalid Goto Instruction opcode {other:?} while building instructions")),
                    }
                };

                if bytecode14 {
                    builder.write_i24(instr.jump_offset);
                } else if instr.popenv_exit_magic {
                    builder.write_i24(0xF00000);    // idek
                } else {
                    // If not using popenv exit magic, encode jump offset as 23-bit signed integer
                    builder.write_i24(instr.jump_offset & 0x7fffff);    // TODO verify that this works
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

                builder.write_i16(build_instance_type(&instr.instance_type));
                builder.write_u8(type1 | type2 << 4);
                builder.write_u8(opcode_raw);

                let variable: &GMVariable = instr.destination.variable.resolve(&builder.gm_data.variables.variables)?;
                write_variable_occurrence(builder, instr.destination.variable.index, instr_abs_pos, variable.name_string_id, instr.destination.variable_type)?;
            }

            GMInstructionData::Push(instr) => {
                // Write 16-bit integer, instance type, or empty data
                builder.write_i16(match &instr.value {
                    GMValue::Int16(int16) => *int16,
                    GMValue::Variable(variable) if !bytecode14 => {
                        let variable: &GMVariable = variable.variable.resolve(&builder.gm_data.variables.variables)?;
                        let b15_data: &GMVariableB15Data = variable.b15_data.as_ref().ok_or("Variable Bytecode 15 data not set while building Push Instruction")?;
                        build_instance_type(&b15_data.instance_type)
                    },
                    _ => 0
                });

                builder.write_u8(instr.data_type.into());
                // builder.write_u8(if bytecode14 { instr.opcode.into() } else { 0xC0 });
                builder.write_u8(self.opcode.into());

                match &instr.value {
                    GMValue::Double(double) => builder.write_f64(*double),
                    GMValue::Float(float) => builder.write_f32(*float),
                    GMValue::Int32(int32) => builder.write_i32(*int32),
                    GMValue::Int64(int64) => builder.write_i64(*int64),
                    GMValue::Boolean(boolean) => builder.write_u8(if *boolean {1} else {0}),
                    GMValue::String(string_ref) => builder.write_u32(string_ref.index),
                    GMValue::Int16(_) => {}     // nothing because it was already written inside the instruction
                    GMValue::Variable(code_variable) => {
                        let variable: &GMVariable = code_variable.variable.resolve(&builder.gm_data.variables.variables)?;
                        write_variable_occurrence(builder, code_variable.variable.index, instr_abs_pos, variable.name_string_id, code_variable.variable_type)?;
                    }
                }
            }

            GMInstructionData::Call(instr) => {
                builder.write_u8(instr.arguments_count);
                builder.write_u8(0);        // TODO check if writing zero is ok since b1 isn't checked or saved
                builder.write_u8(instr.data_type.into());
                builder.write_u8(self.opcode.into());  // v removing this (also for push instruction) might break bytecode14 but the line below is wrong too
                // builder.write_u8(if bytecode14 { instr.opcode.into() } else { 0xDA });

                let function: &GMFunction = instr.function.resolve(&builder.gm_data.functions.functions)?;
                write_function_occurrence(builder, instr.function.index, instr_abs_pos, function.name_string_id)?;
            }

            GMInstructionData::Break(instr) => {
                builder.write_i16(instr.value);
                builder.write_u8(instr.data_type.into());
                builder.write_u8(self.opcode.into());
                if instr.data_type == GMDataType::Int32 {
                    let int_argument: i32 = instr.int_argument.ok_or("Int argument not set but Data Type is Int32 while building Break Instruction")?;
                    builder.write_i32(int_argument);
                }
            }
        }

        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMOpcode {
    Conv = 0x07,     // Push((Types.Second)Pop) // DoubleTypeInstruction
    Mul = 0x08,      // Push(Pop() * Pop()) // DoubleTypeInstruction
    Div = 0x09,      // Push(Pop() / Pop()) // DoubleTypeInstruction
    Rem = 0x0A,      // Push(Remainder(Pop(), Pop())) // DoubleTypeInstruction
    Mod = 0x0B,      // Push(Pop() % Pop()) // DoubleTypeInstruction
    Add = 0x0C,      // Push(Pop() + Pop()) // DoubleTypeInstruction
    Sub = 0x0D,      // Push(Pop() - Pop()) // DoubleTypeInstruction
    And = 0x0E,      // Push(Pop() & Pop()) // DoubleTypeInstruction
    Or = 0x0F,       // Push(Pop() | Pop()) // DoubleTypeInstruction
    Xor = 0x10,      // Push(Pop() ^ Pop()) // DoubleTypeInstruction
    Neg = 0x11,      // Push(-Pop()) // SingleTypeInstruction
    Not = 0x12,      // Push(~Pop()) // SingleTypeInstruction
    Shl = 0x13,      // Push(Pop() << Pop()) // DoubleTypeInstruction
    Shr = 0x14,      // Push(Pop() >>= Pop()) // DoubleTypeInstruction
    Cmp = 0x15,      // Push(Pop() `cmp` Pop())// ComparisonInstruction
    Pop = 0x45,      // Instance.Destination = Pop();
    Dup = 0x86,      // Push(Peek()) // SingleTypeInstruction
    Ret = 0x9C,      // return Pop() // SingleTypeInstruction
    Exit = 0x9D,     // return; // SingleTypeInstruction
    Popz = 0x9E,     // Pop(); // SingleTypeInstruction
    B = 0xB6,        // goto Index + Offset*4; // GotoInstruction
    Bt = 0xB7,       // if (Pop()) goto Index + Offset*4; // GotoInstruction
    Bf = 0xB8,       // if (!Pop()) goto Index + Offset*4; // GotoInstruction
    PushEnv = 0xBA,  // GotoInstruction
    PopEnv = 0xBB,   // GotoInstruction
    Push = 0xC0,     // Push(Value) // push constant
    PushLoc = 0xC1,  // Push(Value) // push local
    PushGlb = 0xC2,  // Push(Value) // push global
    PushBltn = 0xC3, // Push(Value) // push builtin variable
    PushI = 0x84,    // Push(Value) // push int16
    Call = 0xD9,     // Function(arg0, arg1, ..., argn) where arg = Pop() and n = ArgumentsCount
    CallV = 0x99, // TODO: Unknown, maybe to do with calling using the stack? Generates with "show_message((function(){return 5;})());"
    Break = 0xFF, // TODO: Several sub-opcodes in GMS 2.3
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
    pub extra: u8,
    pub data_type: GMDataType,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDoubleTypeInstruction {
    pub type1: GMDataType,
    pub type2: GMDataType,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMComparisonInstruction {
    pub comparison_type: GMComparisonType,  // comparison kind
    pub type1: GMDataType,                  // datatype of element to compare
    pub type2: GMDataType,                  // datatype of element to compare
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMGotoInstruction {
    pub jump_offset: i32,
    pub popenv_exit_magic: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMPopInstruction {
    pub instance_type: GMInstanceType,
    pub type1: GMDataType,
    pub type2: GMDataType,
    pub destination: GMCodeVariable,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMPushInstruction {
    pub data_type: GMDataType,
    pub value: GMValue,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMCallInstruction {
    pub arguments_count: u8,
    pub data_type: GMDataType,
    pub function: GMRef<GMFunction>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMBreakInstruction {
    pub value: i16,
    pub data_type: GMDataType,
    pub int_argument: Option<i32>,
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
    Instance, // obsolete??
    Delete,   // these 3 types apparently exist
    Undefined,
    UnsignedInt,
    Int16 = 0x0f,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMInstanceType {
    Undefined,  // Idk
    Instance(Option<GMRef<GMGameObject>>),      // Represents the current chunk instance.
    Other,      // Represents the other context, which has multiple definitions based on the location used.
    All,        // Represents all active object instances. Assignment operations can perform a loop.
    None,       // Represents no object/instance.
    Global,     // Used for global variables.
    Builtin,    // Used for GML built-in variables.
    Local,      // Used for local variables; local to their code script.
    StackTop,   // Instance is stored in a Variable data type on the top of the stack.
    Argument,   // Used for function argument variables in GMLv2 (GMS 2.3).
    Static,     // Used for static variables.
}
impl Display for GMInstanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GMInstanceType::Undefined => write!(f, "Undefined"),
            GMInstanceType::Instance(None) => write!(f, "Self"),
            GMInstanceType::Instance(Some(reference)) => write!(f, "Self<{}>", reference.index),
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
    Array = 0x00,           // Used for normal single-dimension array variables
    StackTop = 0x80,        // Used when referencing a variable on another variable, e.g. a chain referenc
    Normal = 0xA0,          // normal
    Instance = 0xE0,        // used when referencing variables on room instance IDs, e.g. something like "inst_01ABCDEF.x" in GML
    MultiPush = 0x10,       // GMS2.3+, multidimensional array with pushaf
    MultiPushPop = 0x90,    // GMS2.3+, multidimensional array with pushaf or popaf
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMComparisonType {
    LT = 1,
    LTE = 2,
    EQ = 3,
    NEQ = 4,
    GTE = 5,
    GT = 6,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeVariable {
    pub variable: GMRef<GMVariable>,
    pub variable_type: GMVariableType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMValue {
    Double(f64),
    Float(f32),
    Int32(i32),
    Int64(i64),
    Boolean(bool),
    String(GMRef<String>),
    Variable(GMCodeVariable),
    Int16(i16),
}


fn read_code_value(reader: &mut DataReader, data_type: GMDataType) -> Result<GMValue, String> {
    match data_type {
        GMDataType::Double => reader.read_f64().map(|i| GMValue::Double(i)),
        GMDataType::Float => reader.read_f32().map(|i| GMValue::Float(i)),
        GMDataType::Int32 => reader.read_i32().map(|i| GMValue::Int32(i)),
        GMDataType::Int64 => reader.read_i64().map(|i| GMValue::Int64(i)),
        GMDataType::Boolean => reader.read_u8().map(|i| match i {
            0 => Ok(GMValue::Boolean(false)),
            1 => Ok(GMValue::Boolean(true)),
            other => Err(format!("Invalid boolean value {other} (0x{other:02X}) while reading value in code at absolute position {}", reader.cur_pos-1))
        })?,
        GMDataType::String => reader.read_gm_string().map(GMValue::String),
        GMDataType::Int16 => {
            reader.cur_pos -= 4;
            let number: i16 = reader.read_i16()?;
            reader.cur_pos += 2;
            Ok(GMValue::Int16(number))
        }
        other => Err(format!("Trying to read unsupported data type {other:?} while reading value in code at absolute position {}", reader.cur_pos)),
    }
}


fn read_variable(reader: &mut DataReader, instance_type: &GMInstanceType) -> Result<GMCodeVariable, String> {
    let occurrence_position: usize = reader.cur_pos;
    let raw_value: i32 = reader.read_i32()?;

    let variable_type: i32 = (raw_value >> 24) & 0xF8;
    let variable_type: u8 = variable_type as u8;
    let variable_type: GMVariableType = variable_type.try_into()
        .map_err(|_| format!("Invalid Variable Type {variable_type} (0x{variable_type:02X}) while parsing variable reference chain"))?;

    let variable: GMRef<GMVariable> = reader.variable_occurrence_map.get(&occurrence_position)
        .ok_or_else(|| format!(
            "Could not find {} Variable with occurrence position {} in hashmap with length {} while parsing code value",
            instance_type, occurrence_position, reader.variable_occurrence_map.len(),
        ))?.clone();

    Ok(GMCodeVariable { variable, variable_type })
}


pub fn parse_instance_type(raw_value: i16) -> Result<GMInstanceType, String> {
    // If > 0; then game object id. If < 0, then variable instance type.
    if raw_value > 0 {
        return Ok(GMInstanceType::Instance(Some(GMRef::new(raw_value as u32))))
    }

    let instance_type = match raw_value {
        0 => GMInstanceType::Undefined,         // this doesn't exit in UTMT anymore but enums in C# can hold any value so idk
        -1 => GMInstanceType::Instance(None),
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
        GMInstanceType::Instance(None) => -1,
        GMInstanceType::Instance(Some(game_object_ref)) => game_object_ref.index as i16,
        GMInstanceType::Other => -2,
        GMInstanceType::All => -3,
        GMInstanceType::None => -4,
        GMInstanceType::Global => -5,
        GMInstanceType::Builtin => -6,
        GMInstanceType::Local => -7,
        GMInstanceType::StackTop => -8,
        GMInstanceType::Argument => -15,
        GMInstanceType::Static => -16,
    }
}


fn write_variable_occurrence(
    builder: &mut DataBuilder,
    gm_index: u32,
    occurrence_position: usize,
    name_string_id: i32,
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
    builder.write_i32(name_string_id & 0x07FFFFFF | (((u8::from(variable_type) & 0xF8) as i32) << 24));

    // fuckass borrow checker
    builder.variable_occurrences.get_mut(gm_index as usize).unwrap().push((occurrence_position, variable_type));
    Ok(())
}


fn write_function_occurrence(builder: &mut DataBuilder, gm_index: u32, occurrence_position: usize, name_string_id: i32) -> Result<(), String> {
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
    builder.write_i32(name_string_id & 0x07FFFFFF);

    builder.function_occurrences.get_mut(gm_index as usize).unwrap().push(occurrence_position);
    Ok(())
}

