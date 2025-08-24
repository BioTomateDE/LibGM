use std::collections::HashMap;
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{get_data_type_from_value, GMCodeValue, GMDataType};
use crate::gamemaker::elements::code::GMComparisonType;
use crate::gamemaker::elements::code::CodeVariable;
use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::elements::code::GMInstanceType;
use crate::gamemaker::elements::code::GMInstruction;
use crate::gamemaker::elements::code::GMInstructionData;
use crate::gamemaker::elements::code::GMOpcode;
use crate::gamemaker::elements::code::GMVariableType;
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::elements::variables::GMVariable;

pub fn disassemble_code(gm_data: &GMData, code: &GMCode) -> Result<String, String> {
    let mut assembly: String = String::new();

    for instruction in &code.instructions {
        let line: String = disassemble_instruction(gm_data, instruction)?;
        assembly += &(line + "\n");
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
        GMInstructionData::Empty => 1,
        GMInstructionData::SingleType(_) => 1,
        GMInstructionData::Duplicate(_) => 1,
        GMInstructionData::DuplicateSwap(_) => 1,
        GMInstructionData::CallVariable(_) => 1,
        GMInstructionData::DoubleType(_) => 1,
        GMInstructionData::Comparison(_) => 1,
        GMInstructionData::Goto(_) => 1,
        GMInstructionData::Pop(_) => 2,
        GMInstructionData::PopSwap(_) => 2,
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
        GMInstructionData::Extended16(_) => 1,
        GMInstructionData::Extended32(_) => 2,
        GMInstructionData::ExtendedFunc(_) => 2,
    }
}


pub fn disassemble_instruction(gm_data: &GMData, instruction: &GMInstruction) -> Result<String, String> {
    let mut line: String;
    let opcode: &str = opcode_to_string(instruction.opcode);

    match &instruction.kind {
        GMInstructionData::Empty => {
            line = opcode.to_string();
        }

        GMInstructionData::SingleType(instr) => {
            line = format!(
                "{}.{}",
                opcode,
                data_type_to_string(instr.data_type),
            );
        }

        GMInstructionData::Duplicate(instr) => {
            line = format!(
                "{}.{} {}",
                opcode,
                data_type_to_string(instr.data_type),
                instr.size,
            );
        }

        GMInstructionData::DuplicateSwap(instr) => {
            line = format!(
                "{}.{} {} {}",
                opcode,
                data_type_to_string(instr.data_type),
                instr.size1,
                instr.size2,
            );
        }

        GMInstructionData::CallVariable(instr) => {
            line = format!(
                "{}.{} {}",
                opcode,
                data_type_to_string(instr.data_type),
                instr.argument_count,
            );
        }

        GMInstructionData::DoubleType(instr) => {
            line = format!(
                "{}.{}.{}",
                opcode,
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
            );
        }

        GMInstructionData::Comparison(instr) => {
            line = format!(
                "{}.{}.{} {}",
                opcode,
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
                comparison_type_to_string(instr.comparison_type),
            );
        }

        GMInstructionData::Goto(instr) => {
            if let Some(jump_offset) = instr.jump_offset {
                line = format!(
                    "{} {}",
                    opcode,
                    jump_offset,
                );
            } else {
                line = format!(
                    "{} <drop>",
                    opcode,
                );
            }
        }

        GMInstructionData::Pop(instr) => {
            line = format!(
                "{}.{}.{} ",
                opcode,
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
            );

            let dest: &CodeVariable = &instr.destination;
            if instr.type1 == GMDataType::Variable && dest.instance_type != GMInstanceType::Undefined {
                if dest.variable_type == GMVariableType::Instance && !matches!(dest.instance_type, GMInstanceType::RoomInstance(_)) {
                    return Err(format!(
                        "Expected Pop instruction destination Variable's instance type to be RoomInstance\
                        (because the variable type is Instance), but actually found instance type {}",
                        dest.instance_type,
                    ))
                }   // ^ probably redundant check
                line += &instance_type_to_string(gm_data, &dest.instance_type)?;
                line += ".";
            }
            line += &variable_to_string(gm_data, &instr.destination)?;
        }

        // TODO: (global) verify var and func names (no spaces etc)

        GMInstructionData::PopSwap(instr) => {
            line = format!(
                "{}.{} {}",
                opcode,
                data_type_to_string(GMDataType::Int16),
                instr.size,
            );
        }

        GMInstructionData::Push(instr) => {
            let value: String = match &instr.value {
                GMCodeValue::Variable(code_variable) => {
                    let prefix: &str = if code_variable.is_int32 {"[variable]"} else {""};
                    let variable_string: String = variable_to_string(gm_data, &code_variable)?;
                    if code_variable.instance_type == GMInstanceType::Undefined {
                        format!("{}{}", prefix, variable_string)
                    } else {
                        let inst: String = instance_type_to_string(gm_data, &code_variable.instance_type)?;
                        format!("{}{}.{}", prefix, inst, variable_string)
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

            line = format!(
                "{}.{} {}",
                opcode,
                data_type_to_string(get_data_type_from_value(&instr.value)),
                value,
            );
        }

        GMInstructionData::Call(instr) => {
            line = format!(
                "{}.{} {}(argc={})",
                opcode,
                data_type_to_string(instr.data_type),
                function_to_string(gm_data, instr.function)?,
                instr.arguments_count,
            );
        }

        GMInstructionData::Extended16(instr) => {
            line = format!(
                "{}.{}",
                extended_id_to_string(instr.kind)?,
                data_type_to_string(GMDataType::Int16),
            );
        }

        GMInstructionData::Extended32(instr) => {
            line = format!(
                "{}.{} {}",
                extended_id_to_string(instr.kind)?,
                data_type_to_string(GMDataType::Int32),
                instr.int_argument
            );
        }

        GMInstructionData::ExtendedFunc(instr) => {
            line = format!(
                "{}.{} [function]{}",
                extended_id_to_string(instr.kind)?,
                data_type_to_string(GMDataType::Int32),
                function_to_string(gm_data, instr.function)?,
            );
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
        GMOpcode::BranchIf => "jt",
        GMOpcode::BranchUnless => "jf",
        GMOpcode::PushWithContext => "pushenv",
        GMOpcode::PopWithContext => "popenv",
        GMOpcode::Push => "push",
        GMOpcode::PushLocal => "pushloc",
        GMOpcode::PushGlobal => "pushglb",
        GMOpcode::PushBuiltin => "pushbltn",
        GMOpcode::PushImmediate => "pushim",
        GMOpcode::Call => "call",
        GMOpcode::CallVariable => "callvar",
        GMOpcode::Extended => "break",
    }
}


fn data_type_to_string(data_type: GMDataType) -> &'static str {
    match data_type {
        GMDataType::Int16 => "e",
        GMDataType::Int32 => "i",
        GMDataType::Int64 => "l",
        GMDataType::Double => "d",
        GMDataType::Float => "f",
        GMDataType::Boolean => "b",
        GMDataType::String => "s",
        GMDataType::Variable => "v",
    }
}


fn comparison_type_to_string(comparison_type: GMComparisonType) -> &'static str {
    match comparison_type {
        GMComparisonType::LessThan => "LT",
        GMComparisonType::LessOrEqual => "LTE",
        GMComparisonType::Equal => "EQ",
        GMComparisonType::NotEqual => "NEQ",
        GMComparisonType::GreaterOrEqual => "GTE",
        GMComparisonType::GreaterThan => "GT",
    }
}


fn instance_type_to_string(gm_data: &GMData, instance_type: &GMInstanceType) -> Result<String, String> {
    Ok(match instance_type {
        GMInstanceType::Undefined => return Err("Did not expect Instance Type Undefined here; please report this error.".to_string()),
        GMInstanceType::Self_(Some(obj_ref)) => {
            let obj: &GMGameObject = obj_ref.resolve(&gm_data.game_objects.game_objects)?;
            let name: &String = obj.name.resolve(&gm_data.strings.strings)?;
            format!("[object]{name}")
        }
        GMInstanceType::Self_(None) => "self".to_string(),
        GMInstanceType::RoomInstance(instance_id) => format!("[roominst]{instance_id}"),
        GMInstanceType::Other => "other".to_string(),
        GMInstanceType::All => "all".to_string(),
        GMInstanceType::None => "none".to_string(),
        GMInstanceType::Global => "global".to_string(),
        GMInstanceType::Builtin => "builtin".to_string(),
        GMInstanceType::Local => "local".to_string(),
        GMInstanceType::StackTop => "stacktop".to_string(),
        GMInstanceType::Argument => "arg".to_string(),
        GMInstanceType::Static => "static".to_string(),
    })
}


fn variable_type_to_string(variable_type: GMVariableType) -> &'static str {
    match variable_type {
        GMVariableType::Array => "array",
        GMVariableType::StackTop => "stacktop",
        GMVariableType::Normal => "normal",
        GMVariableType::Instance => "instance",
        GMVariableType::ArrayPushAF => "arraypushaf",
        GMVariableType::ArrayPopAF => "arraypopaf",
    }
}


fn variable_to_string(gm_data: &GMData, code_variable: &CodeVariable) -> Result<String, String> {
    // NOTE: in utmt, it just prints null instead of throwing.
    let variable: &GMVariable = code_variable.variable.resolve(&gm_data.variables.variables)?;
    let name: &String = variable.name.resolve(&gm_data.strings.strings)?;
    if !is_valid_identifier(name) {
        return Err(format!("Invalid variable identifier: {}", format_literal_string(gm_data, variable.name)?))
    }

    let string: String = if code_variable.variable_type == GMVariableType::Normal {
        name.clone()
    } else {
        let instance_type: &GMInstanceType = variable.b15_data.as_ref().map_or(&GMInstanceType::Undefined, |b15| &b15.instance_type);
        format!(
            "[{}]{}.{}",
            variable_type_to_string(code_variable.variable_type),
            instance_type_to_string(gm_data, instance_type)?,
            name,
        )
    };

    Ok(string)
}


fn function_to_string(gm_data: &GMData, function_ref: GMRef<GMFunction>) -> Result<&String, String> {
    // NOTE: in utmt, it just prints null instead of throwing.
    let function: &GMFunction = function_ref.resolve(&gm_data.functions.functions)?;
    let name: &String = function.name.resolve(&gm_data.strings.strings)?;
    if !is_valid_identifier(name) {
        return Err(format!("Invalid function identifier: {}", format_literal_string(gm_data, function.name)?))
    }
    Ok(name)
}


pub fn format_literal_string(gm_data: &GMData, gm_string_ref: GMRef<String>) -> Result<String, String> {
    let string: String = gm_string_ref.resolve(&gm_data.strings.strings)?
        .replace("\\", "\\\\")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
        .replace("\"", "\\\"");
    
    Ok(format!(
        "\"{}\"",
        string,
    ))
}


fn extended_id_to_string(extended_id: i16) -> Result<&'static str, String> {
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


/// Check whether an identifier (function/variable name) is valid for assembling properly.
/// I wanted follow official GameMaker rules
/// (https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Overview/Variables_And_Variable_Scope.htm#:~:text=Naming%20Rules),
/// but since these rules do not include exceptions like `$$$$temp$$$$` or `@@This@@`
/// and generated function names ignore the 64-character limit, I don't cling onto them.
fn is_valid_identifier(s: &str) -> bool {
    const SPECIALS: [char; 3] = ['_', '$', '@'];

    if s.is_empty() {
        return false
    }

    let first_char = s.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && !SPECIALS.contains(&first_char) {
        return false
    }

    s.chars().all(|c| c.is_ascii_alphanumeric() || SPECIALS.contains(&c))
}

