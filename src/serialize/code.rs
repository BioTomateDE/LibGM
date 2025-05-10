use crate::deserialize::all::GMData;
use crate::deserialize::code::{GMDataType, GMInstanceType, GMInstruction, GMOpcode, GMValue};
use crate::deserialize::variables::GMVariables;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};


pub fn build_chunk_code(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "CODE", abs_pos: data_builder.len() };
    let len: usize = gm_data.codes.codes_by_index.len();
    builder.write_usize(len);

    let bytecode14: bool = gm_data.general_info.bytecode_version <= 14;

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::code_meta(i))?;
    }

    let mut code_meta_placeholders: Vec<usize> = Vec::with_capacity(len);

    for (i, code) in gm_data.codes.codes_by_index.iter().enumerate() {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::code_meta(i))?;

        builder.write_gm_string(data_builder, &code.name)?;

        code_meta_placeholders.push(builder.len());
        builder.write_u32(0);   // PLACEHOLDER CODE INSTRUCTIONS LENGTH
        builder.write_u32(0);   // PLACEHOLDER CODE START OFFSET

        builder.write_u32(code.locals_count);
        builder.write_u32(code.arguments_count);
    }

    for (i, code) in gm_data.codes.codes_by_index.iter().enumerate() {
        let placeholder_position: usize = code_meta_placeholders[i];
        let start_offset: usize = builder.len() - placeholder_position + 4;
        for (j, byte) in start_offset.to_le_bytes().iter().enumerate() {
            builder.raw_data[placeholder_position + 4 + j] = *byte;
        }
        let start_position: usize = builder.len();

        for instruction in &code.instructions {
            build_instruction(data_builder, &mut builder, bytecode14, &gm_data.variables, &instruction)?;
        }

        let instructions_length: usize = builder.len() - start_position;
        for (j, byte) in instructions_length.to_le_bytes().iter().enumerate() {
            builder.raw_data[placeholder_position + j] = *byte;
        }
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}

fn build_instruction(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    bytecode14: bool,
    variables: &GMVariables,
    instruction: &GMInstruction,
) -> Result<(), String> {
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
                    other => return Err(format!("Invalid Single Type Instruction opcode {other:?} while building instructions.")),
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
                    other => return Err(format!("Invalid Double Type Instruction opcode {other:?} while building instructions.")),
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
                instr.comparison_type.into()
            } else {
                instr.opcode.into()     // always GMOpcode::Cmp
            };
            let type1: u8 = instr.type1.into();
            let type2: u8 = instr.type2.into();

            builder.write_u8(0);
            builder.write_u8(0);
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
                    other => return Err(format!("Invalid Goto Instruction opcode {other:?} while building instructions.")),
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
                return Err("Int16 Data Type not yet supported while building Pop Instruction.".to_string())
            }

            let opcode_raw: u8 = if !bytecode14 { instr.opcode.into() } else {
                match instr.opcode {
                    GMOpcode::Pop => 0x41,
                    other => return Err(format!("Invalid Pop Instruction opcode {other:?} while building instructions.")),
                }
            };
            let type1: u8 = instr.type1.into();
            let type2: u8 = instr.type2.into();

            builder.write_i16(build_instance_type(&instr.instance_type));
            builder.write_u8(type1 | type2 << 4);
            builder.write_u8(opcode_raw);
        }

        GMInstruction::Push(instr) => {
            // Write 16-bit integer, instance type, or empty data
            builder.write_i16(match &instr.value {
                GMValue::Int16(int16) => *int16,
                GMValue::Variable(variable) => build_instance_type(&variable.variable.resolve(&variables.variables)?.instance_type),
                _ => 0
            });

            builder.write_u8(instr.data_type.into());
            builder.write_u8(if bytecode14 { instr.opcode.into() } else { 0xC0 });
            
            match &instr.value {
                GMValue::Double(double) => builder.write_f64(*double),
                GMValue::Float(float) => builder.write_f32(*float),
                GMValue::Int32(int32) => builder.write_i32(*int32),
                GMValue::Int64(int64) => builder.write_i64(*int64),
                GMValue::Boolean(boolean) => builder.write_u8(if *boolean {1} else {0}),
                GMValue::String(string_ref) => builder.write_usize(string_ref.index),
                GMValue::Variable(_) | GMValue::Int16(_) => {}     // nothing because it was already written inside the instruction
            }
        }

        GMInstruction::Call(instr) => {
            builder.write_u8(instr.arguments_count as u8);
            builder.write_u8(instr.data_type.into());
            builder.write_u8(if bytecode14 { instr.opcode.into() } else { 0xDA });
            data_builder.push_pointer_placeholder(builder, GMPointer::function(instr.function.index))?;
        }

        GMInstruction::Break(instr) => {
            builder.write_i16(instr.value);
            builder.write_u8(instr.data_type.into());
            builder.write_u8(instr.opcode.into());
            if instr.data_type == GMDataType::Int32 {
                let int_argument: i32 = instr.int_argument.ok_or_else(|| "Int argument not set but Data Type is Int32 while building Break Instruction.")?;
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
        GMInstanceType::Instance(Some(game_object_ref)) => game_object_ref.index as i16,
        GMInstanceType::Instance(None) => -1,
        GMInstanceType::Other => -2,
        GMInstanceType::All => -3,
        GMInstanceType::Noone => -4,
        GMInstanceType::Global => -5,
        GMInstanceType::Local => -7,
        GMInstanceType::Stacktop => -8,
        GMInstanceType::Arg => -15,
        GMInstanceType::Static => -16,
    }
}

