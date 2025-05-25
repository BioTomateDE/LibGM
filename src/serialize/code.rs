use std::collections::HashMap;
use crate::deserialize::all::GMData;
use crate::deserialize::code::{GMCodeBytecode15, GMCodes, GMDataType, GMInstanceType, GMInstruction, GMOpcode, GMValue, GMVariableType};
use crate::deserialize::functions::{GMFunction, GMFunctions};
use crate::deserialize::strings::GMStrings;
use crate::deserialize::variables::{GMVariable, GMVariables};
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};


pub fn build_chunk_code(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>), String> {
    let mut builder = ChunkBuilder::new(data_builder, "CODE");
    let len: usize = gm_data.codes.codes_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.write_pointer_placeholder(&mut builder, GMPointer::CodeMeta(i))?;
    }

    let (variable_occurrences_map, function_occurrences_map) = if gm_data.general_info.bytecode_version <= 14 {
        build_code_b14(data_builder, &mut builder, &gm_data.strings, &gm_data.variables, &gm_data.functions, &gm_data.codes)?
    } else {
        build_code_b15(data_builder, &mut builder, &gm_data.strings, &gm_data.variables, &gm_data.functions, &gm_data.codes)?
    };

    builder.finish(data_builder)?;
    Ok((variable_occurrences_map, function_occurrences_map))
}


fn build_code_b14(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    strings: &GMStrings,
    variables: &GMVariables,
    functions: &GMFunctions,
    codes: &GMCodes,
) -> Result<(HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>), String> {
    let mut variable_occurrences_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut function_occurrences_map: HashMap<usize, Vec<usize>> = HashMap::new();

    for (i, code) in codes.codes_by_index.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::CodeMeta(i))?;

        builder.write_gm_string(data_builder, &code.name)?;

        // write placeholder for code length
        let length_placeholder_pos: usize = builder.len();
        builder.write_u32(0xDEAD);

        data_builder.resolve_pointer(builder, GMPointer::Code(i))?;     // TODO probably wrong??????
        for (j, instruction) in code.instructions.iter().enumerate() {
            build_instruction(builder, true, variables, functions, instruction, &mut variable_occurrences_map, &mut function_occurrences_map)
                .map_err(|e| format!("{e} while building Instruction #{j} of Code #{i} with name \"{}\"", code.name.display(strings)))?;
        }

        // overwrite placeholder for code length
        let code_length: usize = builder.len() - length_placeholder_pos - 4;        // -4 because the length specification doesn't count
        builder.overwrite_usize(code_length, length_placeholder_pos)?;
    }

    Ok((variable_occurrences_map, function_occurrences_map))
}


fn build_code_b15(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    strings: &GMStrings,
    variables: &GMVariables,
    functions: &GMFunctions,
    codes: &GMCodes,
) -> Result<(HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>), String> {
    let mut variable_occurrences_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut function_occurrences_map: HashMap<usize, Vec<usize>> = HashMap::new();

    let mut length_placeholders: Vec<usize> = Vec::with_capacity(codes.codes_by_index.len());
    let mut start_placeholders: Vec<usize> = Vec::with_capacity(codes.codes_by_index.len());

    for (i, code) in codes.codes_by_index.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::CodeMeta(i))?;

        builder.write_gm_string(data_builder, &code.name)?;

        // write placeholder for code length
        length_placeholders.push(builder.len());
        builder.write_u32(0xDEAD);

        // write bytecode15 info
        let b15_info: &GMCodeBytecode15 = code.bytecode15_info.as_ref()
            .ok_or_else(|| format!("Bytecode 15 info not set for Code #{i} with name \"{}\"", code.name.display(&strings)))?;
        builder.write_u16(b15_info.locals_count);
        builder.write_u16(b15_info.arguments_count | if b15_info.weird_local_flag { 0x8000 } else { 0 });

        // write placeholder for code instructions start address
        start_placeholders.push(builder.len());
        builder.write_u32(0xDEAD);

        // write offset thingy
        builder.write_usize(b15_info.offset);
    }

    for (i, code) in codes.codes_by_index.iter().enumerate() {
        let start: usize = builder.len();

        data_builder.resolve_pointer(builder, GMPointer::Code(i))?;         // TODO also probably wrong?? maybe it's just the same as CodeMeta?
        for (j, instruction) in code.instructions.iter().enumerate() {
            build_instruction(builder, false, variables, functions, instruction, &mut variable_occurrences_map, &mut function_occurrences_map)
                .map_err(|e| format!("{e} while building Instruction #{j} of Code #{i} with name \"{}\"", code.name.display(strings)))?;
        }

        let end: usize = builder.len();
        // overwrite code length placeholder
        let code_length: usize = end - start;
        builder.overwrite_usize(code_length, length_placeholders[i])?;

        // overwrite code instructions start address placeholder
        let bytecode_relative_address: usize = end - start_placeholders[i] - code_length;
        builder.overwrite_usize(bytecode_relative_address, start_placeholders[i])?;
    }

    Ok((variable_occurrences_map, function_occurrences_map))
}


fn build_instruction(
    builder: &mut ChunkBuilder,
    bytecode14: bool,
    variables: &GMVariables,
    functions: &GMFunctions,
    instruction: &GMInstruction,
    variable_occurrences_map: &mut HashMap<usize, Vec<usize>>,
    function_occurrences_map: &mut HashMap<usize, Vec<usize>>,
) -> Result<(), String> {
    let abs_pos: usize = builder.abs_pos + builder.len();
    
    match instruction {
        GMInstruction::SingleType(instr) => {
            let opcode_raw: u8 = if !bytecode14 { instr.opcode.into() } else {
                match instr.opcode {
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

        GMInstruction::DoubleType(instr) => {
            let opcode_raw: u8 = if !bytecode14 { instr.opcode.into() } else {
                match instr.opcode {
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

        GMInstruction::Comparison(instr) => {
            let opcode_raw: u8 = if bytecode14 {
                u8::from(instr.comparison_type) + 0x10
            } else {
                u8::from(instr.opcode)     // always GMOpcode::Cmp
            };
            let type1: u8 = instr.type1.into();
            let type2: u8 = instr.type2.into();

            builder.write_u8(0);
            builder.write_u8(instr.comparison_type.into());
            builder.write_u8(type1 | type2 << 4);
            builder.write_u8(opcode_raw);
        }

        GMInstruction::Goto(instr) => {
            let opcode_raw: u8 = if !bytecode14 { instr.opcode.into() } else {
                match instr.opcode {
                    GMOpcode::B => 0xB7,
                    GMOpcode::Bt => 0xB8,
                    GMOpcode::Bf => 0xB9,
                    GMOpcode::PushEnv => 0xBB,
                    GMOpcode::PopEnv => 0xBC,
                    other => return Err(format!("Invalid Goto Instruction opcode {other:?} while building instructions")),
                }
            };

            if bytecode14 {
                builder.write_i32(instr.jump_offset);
            } else if instr.popenv_exit_magic {
                builder.write_i32(0xF00000);    // idek
            } else {
                // If not using popenv exit magic, encode jump offset as 23-bit signed integer
                builder.write_i32(instr.jump_offset & 0x7fffff);    // TODO verify that this works
            }

            builder.raw_data.pop();   // write jump_offset as int24; using 3 bytes
            builder.write_u8(opcode_raw);
        }

        GMInstruction::Pop(instr) => {
            if instr.type1 == GMDataType::Int16 {
                return Err("Int16 Data Type not yet supported while building Pop Instruction".to_string())
            }

            let opcode_raw: u8 = if !bytecode14 { instr.opcode.into() } else {
                match instr.opcode {
                    GMOpcode::Pop => 0x41,
                    other => return Err(format!("Invalid Pop Instruction opcode {other:?} while building instructions")),
                }
            };
            let type1: u8 = instr.type1.into();
            let type2: u8 = instr.type2.into();

            builder.write_i16(build_instance_type(&instr.instance_type));
            builder.write_u8(type1 | type2 << 4);
            builder.write_u8(opcode_raw);
            
            let variable: &GMVariable = instr.destination.variable.resolve(&variables.variables)?;
            write_occurrence(builder, variable_occurrences_map, instr.destination.variable.index, abs_pos, variable.name_string_id, Some(instr.destination.variable_type))?;
        }

        GMInstruction::Push(instr) => {
            // Write 16-bit integer, instance type, or empty data
            builder.write_i16(match &instr.value {
                GMValue::Int16(int16) => *int16,
                GMValue::Variable(variable) => build_instance_type(&variable.variable.resolve(&variables.variables)?.instance_type),
                _ => 0
            });

            builder.write_u8(instr.data_type.into());
            // builder.write_u8(if bytecode14 { instr.opcode.into() } else { 0xC0 });
            builder.write_u8(instr.opcode.into());

            match &instr.value {
                GMValue::Double(double) => builder.write_f64(*double),
                GMValue::Float(float) => builder.write_f32(*float),
                GMValue::Int32(int32) => builder.write_i32(*int32),
                GMValue::Int64(int64) => builder.write_i64(*int64),
                GMValue::Boolean(boolean) => builder.write_u8(if *boolean {1} else {0}),
                GMValue::String(string_ref) => builder.write_usize(string_ref.index),
                GMValue::Int16(_) => {}     // nothing because it was already written inside the instruction
                GMValue::Variable(code_variable) => {
                    let variable: &GMVariable = code_variable.variable.resolve(&variables.variables)?;
                    write_occurrence(builder, variable_occurrences_map, code_variable.variable.index, abs_pos, variable.name_string_id, Some(code_variable.variable_type))?;
                }
            }
        }

        GMInstruction::Call(instr) => {
            builder.write_u8(instr.arguments_count);
            builder.write_u8(0);        // TODO check if writing zero is ok since b1 isn't checked or saved
            builder.write_u8(instr.data_type.into());
            builder.write_u8(instr.opcode.into());  // v removing this (also for push instruction) might break bytecode14 but the line below is wrong too
            // builder.write_u8(if bytecode14 { instr.opcode.into() } else { 0xDA });

            let function: &GMFunction = instr.function.resolve(&functions.functions_by_index)?;
            write_occurrence(builder, function_occurrences_map, instr.function.index, abs_pos, function.name_string_id, None)?;
        }

        GMInstruction::Break(instr) => {
            builder.write_i16(instr.value);
            builder.write_u8(instr.data_type.into());
            builder.write_u8(instr.opcode.into());
            if instr.data_type == GMDataType::Int32 {
                let int_argument: i32 = instr.int_argument.ok_or_else(|| "Int argument not set but Data Type is Int32 while building Break Instruction")?;
                builder.write_i32(int_argument);
            }
        }
    }

    Ok(())
}


pub fn build_instance_type(instance_type: &GMInstanceType) -> i16 {
    // If > 0; then game object id. If < 0, then variable instance type.
    match instance_type {
        GMInstanceType::Undefined => 0,
        GMInstanceType::Instance(None) => -1,
        GMInstanceType::Instance(Some(game_object_ref)) => game_object_ref.index as i16,
        GMInstanceType::Other => -2,
        GMInstanceType::All => -3,
        GMInstanceType::Noone => -4,
        GMInstanceType::Global => -5,
        GMInstanceType::Builtin => -6,
        GMInstanceType::Local => -7,
        GMInstanceType::StackTop => -8,
        GMInstanceType::Argument => -15,
        GMInstanceType::Static => -16,
    }
}



fn write_occurrence(
    builder: &mut ChunkBuilder,
    occurrence_map: &mut HashMap<usize, Vec<usize>>,
    gm_index: usize,
    occurrence_position: usize,
    name_string_id: i32,
    variable_type: Option<GMVariableType>,
) -> Result<(), String> {
    let entry: &mut Vec<usize> = occurrence_map
        .entry(gm_index)
        .or_insert_with(Vec::new);

    if let Some(last_occurrence_position) = entry.last() {
        // replace last occurrence (which is name string id) with next occurrence offset
        let occurrence_offset: i32 = occurrence_position as i32 - *last_occurrence_position as i32;
        let variable_type_raw: u8 = if let Some(var_type) = variable_type { var_type.into() } else { 0 };       // TODO idk if variable types are always the same (probably not)
        let occurrence_offset_full: i32 = occurrence_offset & 0x07FFFFFF | (((variable_type_raw & 0xF8) as i32) << 24);
        log::warn!("hfjjdf {} {} {:08X} {:?}", occurrence_offset, variable_type_raw, occurrence_offset_full, variable_type);
        builder.overwrite_i32(occurrence_offset_full, last_occurrence_position - builder.abs_pos + 4)?;
    }
    
    // write name string id for this occurrence. this is correct if it is the last occurrence.
    // otherwise, it will be overwritten later by the code above.
    let variable_type_raw: u8 = if let Some(var_type) = variable_type { var_type.into() } else { 0 };
    builder.write_i32(name_string_id & 0x07FFFFFF | (((variable_type_raw & 0xF8) as i32) << 24));
    
    entry.push(occurrence_position);
    Ok(())
}

