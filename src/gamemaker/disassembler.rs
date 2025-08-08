use std::collections::HashMap;
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{get_data_type_from_value, GMCodeValue, GMDataType};
use crate::gamemaker::elements::code::GMComparisonType;
use crate::gamemaker::elements::code::GMCodeVariable;
use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::elements::code::GMInstanceType;
use crate::gamemaker::elements::code::GMInstruction;
use crate::gamemaker::elements::code::GMInstructionData;
use crate::gamemaker::elements::code::GMOpcode;
use crate::gamemaker::elements::code::GMVariableType;
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::variables::GMVariable;

pub fn disassemble_code(gm_data: &GMData, code: &GMCode) -> Result<String, String> {
    let instructions: &[GMInstruction] = &code.instructions;
    let blocks: HashMap<u32, usize> = find_blocks(instructions);
    let mut current_address: u32 = 0;
    let mut current_index: usize = 0;
    let mut assembly: String = String::new();

    for instruction in instructions {
        let line: String = disassemble_instruction(
            gm_data,
            instruction,
            |jump_offset| {
                let target_address: u32 = (current_address as i32 + jump_offset) as u32;
                let target_index: usize = *blocks.get(&target_address)
                    .ok_or_else(|| format!(
                        "Could not resolve branch target instruction with jump offset {} for \"{:?}\" instruction with code address {}",
                        jump_offset, instruction.opcode, target_address,
                    ))?;
                let target_index_rel: i32 = target_index as i32 - current_index as i32;
                Ok(target_index_rel)
            }
        )?;
        assembly += &(line + "\n");
        current_address += get_instruction_size(instruction);
        current_index += 1;
    }

    Ok(assembly)
}


pub fn find_blocks(instructions: &[GMInstruction]) -> HashMap<u32, usize> {
    let mut blocks: HashMap<u32, usize> = HashMap::with_capacity(instructions.len());
    let mut current_address: u32 = 0;

    for (i, instruction) in instructions.iter().enumerate() {
        blocks.insert(current_address, i);
        current_address += get_instruction_size(instruction);
    }

    // insert end block
    blocks.insert(current_address, instructions.len());

    blocks
}


pub fn get_instruction_size(instruction: &GMInstruction) -> u32 {
    // TODO: maybe caching these sizes is faster (store in list)? but needs benchmarks
    match &instruction.kind {
        GMInstructionData::SingleType(_) => 1,
        GMInstructionData::DoubleType(_) => 1,
        GMInstructionData::Comparison(_) => 1,
        GMInstructionData::Goto(_) => 1,
        GMInstructionData::Pop(_) => 2,
        GMInstructionData::Push(instr) => match instr.value {
            GMCodeValue::Int16(_) => 1,
            GMCodeValue::Int32(_) => 2,
            GMCodeValue::Float(_) => 2,
            GMCodeValue::Boolean(_) => 2,
            GMCodeValue::String(_) => 2,
            GMCodeValue::Variable(_) => 2,
            GMCodeValue::Function(_) => 2,
            GMCodeValue::Int64(_) => 3,
            GMCodeValue::Double(_) => 3,
        }
        GMInstructionData::Call(_) => 2,
        GMInstructionData::Break(instr) => {
            if instr.data_type == GMDataType::Int32 { 2 } else { 1 }
        }
    }
}


pub fn disassemble_instruction(gm_data: &GMData, instruction: &GMInstruction, resolve_goto_target: impl Fn(i32) -> Result<i32, String>) -> Result<String, String> {
    let mut line: String;

    match &instruction.kind {
        GMInstructionData::SingleType(instr) => {
            line = format!(
                "{}.{}",
                opcode_to_string(instruction.opcode),
                data_type_to_string(instr.data_type),
            );
            if matches!(instruction.opcode, GMOpcode::Duplicate | GMOpcode::CallVariable) {
                line += &format!(" {}", instr.extra);
                // {~~} handle special dup instruction with comparison type??
            }
        }

        GMInstructionData::DoubleType(instr) => {
            line = format!(
                "{}.{}.{}",
                opcode_to_string(instruction.opcode),
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
            );
        }

        GMInstructionData::Comparison(instr) => {
            line = format!(
                "{}.{}.{} {}",
                opcode_to_string(instruction.opcode),
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
                comparison_type_to_string(instr.comparison_type),
            );
        }

        GMInstructionData::Goto(instr) => {
            if instr.popenv_exit_magic {
                line = format!(
                    "{} <drop>",
                    opcode_to_string(instruction.opcode),
                );
            } else {
                line = format!(
                    "{} {}",
                    opcode_to_string(instruction.opcode),
                    resolve_goto_target(instr.jump_offset)?,
                );
            }
        }

        GMInstructionData::Pop(instr) => {
            // {~~} handle special swap instruction
            line = format!(
                "{}.{}.{} ",
                opcode_to_string(instruction.opcode),
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
            );

            let dest: &GMCodeVariable = &instr.destination;
            if instr.type1 == GMDataType::Variable && dest.instance_type != GMInstanceType::Undefined {
                if dest.variable_type == GMVariableType::Instance && !matches!(dest.instance_type, GMInstanceType::RoomInstance(_)) {
                    return Err(format!(
                        "Expected Pop instruction destination Variable's instance type to be RoomInstance\
                        (because the variable type is Instance), but actually found instance type {}",
                        dest.instance_type,
                    ))
                }
                line += &instance_type_to_string(&dest.instance_type)?;
                line += ".";
            }
            line += &variable_to_string(gm_data, &instr.destination)?;
        }

        GMInstructionData::Push(instr) => {
            line = format!(
                "{}.{} ",
                opcode_to_string(instruction.opcode),
                data_type_to_string(get_data_type_from_value(&instr.value)),
            );

            let string: String = match &instr.value {
                GMCodeValue::Variable(code_variable) => {
                    let variable_string: String = variable_to_string(gm_data, &code_variable)?;
                    if code_variable.instance_type == GMInstanceType::Undefined {
                        variable_string
                    } else {
                        instance_type_to_string(&code_variable.instance_type)? + "." + &variable_string
                    }
                }

                GMCodeValue::Boolean(bool) => if *bool {
                    // in UTMT, it just doesn't append anything for booleans
                    "true".to_string()
                } else {
                    "false".to_string()
                }

                GMCodeValue::Function(function_ref) => {
                    format!("[function]{}", function_to_string(gm_data, *function_ref)?)
                }

                GMCodeValue::String(string_ref) => format_literal_string(gm_data, *string_ref)?,
                GMCodeValue::Int16(integer) => integer.to_string(),
                GMCodeValue::Int32(integer) => integer.to_string(),
                GMCodeValue::Int64(integer) => integer.to_string(),
                GMCodeValue::Double(float) => float.to_string(),
                GMCodeValue::Float(float) => float.to_string(),
            };

            line += &string;
        }

        GMInstructionData::Call(instr) => {
            line = format!(
                "{}.{} {}(argc={})",
                opcode_to_string(instruction.opcode),
                data_type_to_string(instr.data_type),
                function_to_string(gm_data, instr.function)?,
                instr.arguments_count,
            );
        }

        GMInstructionData::Break(instr) => {
            match break_id_to_string(instr.extended_kind) {
                Ok(extended_kind) => line = format!(
                    "{}.{}",
                    extended_kind,
                    data_type_to_string(instr.data_type),
                ),
                Err(_) => line = format!(
                    "{}.{} {}",
                    opcode_to_string(instruction.opcode),
                    data_type_to_string(instr.data_type),
                    instr.extended_kind,
                ),
            }

            if let Some(integer) = instr.int_argument {
                // TODO: support function references
                line += &format!(" {integer}");
            }

        }

    }

    Ok(line)
}


fn opcode_to_string(opcode: GMOpcode) -> &'static str {
    match opcode {
        GMOpcode::Convert => "conv",
        GMOpcode::Multiply => "mul",
        GMOpcode::Divide => "div",
        GMOpcode::Remainder => "rem",
        GMOpcode::Modulus => "mod",
        GMOpcode::Add => "add",
        GMOpcode::Subtract => "sub",
        GMOpcode::And => "and",
        GMOpcode::Or => "or",
        GMOpcode::Xor => "xor",
        GMOpcode::Negate => "neg",
        GMOpcode::Not => "not",
        GMOpcode::ShiftLeft => "shl",
        GMOpcode::ShiftRight => "shr",
        GMOpcode::Compare => "cmp",
        GMOpcode::Pop => "pop",
        GMOpcode::Duplicate => "dup",
        GMOpcode::Return => "ret",
        GMOpcode::Exit => "exit",
        GMOpcode::PopDiscard => "popz",
        GMOpcode::Branch => "jmp",
        GMOpcode::BranchIf => "bt",
        GMOpcode::BranchUnless => "bf",
        GMOpcode::PushWithContext => "pushenv",
        GMOpcode::PopWithContext => "popenv",
        GMOpcode::Push => "push",
        GMOpcode::PushLocal => "pushloc",
        GMOpcode::PushGlobal => "pushglb",
        GMOpcode::PushBuiltin => "pushbltn",
        GMOpcode::PushImmediate => "pushim",
        GMOpcode::Call => "call",
        GMOpcode::CallVariable => "callv",
        GMOpcode::Extended => "break",
    }
}


fn data_type_to_string(data_type: GMDataType) -> &'static str {
    match data_type {
        GMDataType::Int16 => "i16",
        GMDataType::Int32 => "i32",
        GMDataType::Int64 => "i64",
        GMDataType::Double => "f64",
        GMDataType::Float => "f32",
        GMDataType::Boolean => "bol",
        GMDataType::String => "str",
        GMDataType::Variable => "var",
    }
}


fn comparison_type_to_string(comparison_type: GMComparisonType) -> &'static str {
    match comparison_type {
        GMComparisonType::LT => "LT",
        GMComparisonType::LTE => "LTE",
        GMComparisonType::EQ => "EQ",
        GMComparisonType::NEQ => "NEQ",
        GMComparisonType::GTE => "GTE",
        GMComparisonType::GT => "GT",
    }
}


fn instance_type_to_string(instance_type: &GMInstanceType) -> Result<String, String> {
    Ok(match instance_type {
        GMInstanceType::Undefined => return Err("Did not expect Instance Type Undefined here; please report this error.".to_string()),
        GMInstanceType::Self_(Some(obj)) => obj.index.to_string(),
        GMInstanceType::Self_(None) => "self".to_string(),
        GMInstanceType::RoomInstance(instance_id) => instance_id.to_string(),
        GMInstanceType::Other => "other".to_string(),
        GMInstanceType::All => "all".to_string(),
        GMInstanceType::None => "none".to_string(),
        GMInstanceType::Global => "global".to_string(),
        GMInstanceType::Builtin => "builtin".to_string(),
        GMInstanceType::Local => "local".to_string(),
        GMInstanceType::StackTop => "stacktop".to_string(),
        GMInstanceType::Argument => "argument".to_string(),
        GMInstanceType::Static => "static".to_string(),
    })
}


fn variable_type_to_string(variable_type: GMVariableType) -> &'static str {
    match variable_type {
        GMVariableType::Array => "array",
        GMVariableType::StackTop => "stacktop",
        GMVariableType::Normal => "normal",
        GMVariableType::Instance => "instance",
        GMVariableType::MultiPush => "arraypopaf",
        GMVariableType::MultiPushPop => "arraypushaf",
    }
}


fn variable_to_string(gm_data: &GMData, code_variable: &GMCodeVariable) -> Result<String, String> {
    // NOTE: in utmt, it just prints null instead of throwing.
    let variable: &GMVariable = code_variable.variable.resolve(&gm_data.variables.variables)?;
    let name: &String = variable.name.resolve(&gm_data.strings.strings)?;

    // Depending on the context, the instance type of this instruction could be unset (e.g. pop instructions).
    // In that case, try to get the variable's "static" instance type instead (only bytecode 15+).
    let mut instance_type: &GMInstanceType = &code_variable.instance_type;
    if matches!(instance_type, GMInstanceType::Undefined) {
        if let Some(ref b15_data) = variable.b15_data {
            instance_type = &b15_data.instance_type;
        }
    }

    let string: String = if code_variable.variable_type == GMVariableType::Normal {
        name.clone()
    } else {
        format!(
            "[{}]{}.{}",
            variable_type_to_string(code_variable.variable_type),
            instance_type_to_string(instance_type)?,
            name,
        )
    };

    Ok(string)
}


fn function_to_string(gm_data: &GMData, function_ref: GMRef<GMFunction>) -> Result<&String, String> {
    // NOTE: in utmt, it just prints null instead of throwing.
    let function: &GMFunction = function_ref.resolve(&gm_data.functions.functions)?;
    let name: &String = function.name.resolve(&gm_data.strings.strings)?;
    Ok(name)
}


pub fn format_literal_string(gm_data: &GMData, gm_string_ref: GMRef<String>) -> Result<String, String> {
    let string: String = gm_string_ref.resolve(&gm_data.strings.strings)?
        .replace("\\", "\\\\")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\"", "\\\"");
    
    Ok(format!(
        "\"{}\"@{}",
        string,
        gm_string_ref.index,
    ))
}


fn break_id_to_string(extended_id: i16) -> Result<&'static str, String> {
    Ok(match extended_id {
        -1 => "chkindex",
        -2 => "pushaf",
        -3 => "popaf",
        -4 => "pushac",
        -5 => "setowner",
        -6 => "isstaticok",
        -7 => "setstatic",
        -8 => "savearef",
        -9 => "restorearef",
        -10 => "chknullish",
        -11 => "pushref",
        _ => return Err(format!("Unknown Break ID {extended_id}"))
    })
}

