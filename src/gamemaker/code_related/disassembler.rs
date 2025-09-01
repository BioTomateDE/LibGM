use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{get_data_type_from_value, GMCodeValue, GMDataType, GMExtendedKind};
use crate::gamemaker::elements::code::GMComparisonType;
use crate::gamemaker::elements::code::CodeVariable;
use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::elements::code::GMInstanceType;
use crate::gamemaker::elements::code::GMInstruction;
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


pub fn disassemble_instruction(gm_data: &GMData, instruction: &GMInstruction) -> Result<String, String> {
    let line: String;
    let opcode: &str = opcode_to_string(instruction);

    match &instruction {
        GMInstruction::Exit(_) => {
            line = opcode.to_string();
        }

        GMInstruction::Negate(instr) |
        GMInstruction::Not(instr) |
        GMInstruction::Return(instr) |
        GMInstruction::PopDiscard(instr) => {
            line = format!(
                "{}.{}",
                opcode,
                data_type_to_string(instr.data_type),
            );
        }

        GMInstruction::CallVariable(instr) => {
            line = format!(
                "{}.{} {}",
                opcode,
                data_type_to_string(instr.data_type),
                instr.argument_count,
            );
        }

        GMInstruction::Duplicate(instr) => {
            line = format!(
                "{}.{} {}",
                opcode,
                data_type_to_string(instr.data_type),
                instr.size,
            );
        }

        GMInstruction::DuplicateSwap(instr) => {
            line = format!(
                "{}.{} {} {}",
                opcode,
                data_type_to_string(instr.data_type),
                instr.size1,
                instr.size2,
            );
        }
        
        GMInstruction::PopSwap(instr) => {
            line = format!(
                "{} {}",
                opcode,
                instr.size,
            );
        }


        GMInstruction::Branch(instr) |
        GMInstruction::BranchIf(instr) |
        GMInstruction::BranchUnless(instr) |
        GMInstruction::PushWithContext(instr) |
        GMInstruction::PopWithContext(instr) => {
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

        GMInstruction::Convert(instr) |
        GMInstruction::Multiply(instr) |
        GMInstruction::Divide(instr) |
        GMInstruction::Remainder(instr) |
        GMInstruction::Modulus(instr) |
        GMInstruction::Add(instr) |
        GMInstruction::Subtract(instr) |
        GMInstruction::And(instr) |
        GMInstruction::Or(instr) |
        GMInstruction::Xor(instr) |
        GMInstruction::ShiftLeft(instr) |
        GMInstruction::ShiftRight(instr) => {
            line = format!(
                "{}.{}.{}",
                opcode,
                data_type_to_string(instr.right),
                data_type_to_string(instr.left),
            );
        }

        GMInstruction::Compare(instr) => {
            line = format!(
                "{}.{}.{} {}",
                opcode,
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
                comparison_type_to_string(instr.comparison_type),
            );
        }

        GMInstruction::Pop(instr) => {
            line = format!(
                "{}.{}.{} {}",
                opcode,
                data_type_to_string(instr.type1),
                data_type_to_string(instr.type2),
                variable_to_string(gm_data, &instr.destination)?,
            );
        }

        GMInstruction::Push(instr) |
        GMInstruction::PushLocal(instr) |
        GMInstruction::PushGlobal(instr) |
        GMInstruction::PushBuiltin(instr) |
        GMInstruction::PushImmediate(instr) => {
            let value: String = match &instr.value {
                GMCodeValue::Variable(code_variable) => variable_to_string(gm_data, &code_variable)?,
                GMCodeValue::Boolean(true) => "true".to_string(),
                GMCodeValue::Boolean(false) => "false".to_string(),
                GMCodeValue::Function(function_ref) => format!("(function){}", function_to_string(gm_data, *function_ref)?),
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

        GMInstruction::Call(instr) => {
            line = format!(
                "{}.{} {}(argc={})",
                opcode,
                data_type_to_string(instr.data_type),
                function_to_string(gm_data, instr.function)?,
                instr.arguments_count,
            );
        }

        GMInstruction::Extended16(instr) => {
            line = format!(
                "{}.{}",
                extended_kind_to_string(instr.kind),
                data_type_to_string(GMDataType::Int16),
            );
        }

        GMInstruction::Extended32(instr) => {
            line = format!(
                "{}.{} {}",
                extended_kind_to_string(instr.kind),
                data_type_to_string(GMDataType::Int32),
                instr.int_argument
            );
        }

        GMInstruction::ExtendedFunc(instr) => {
            line = format!(
                "{}.{} (function){}",
                extended_kind_to_string(instr.kind),
                data_type_to_string(GMDataType::Int32),
                function_to_string(gm_data, instr.function)?,
            );
        }

    }

    Ok(line)
}


fn opcode_to_string(instruction: &GMInstruction) -> &'static str {
    match instruction {
        GMInstruction::Convert(_) => "conv",
        GMInstruction::Multiply(_) => "mul",
        GMInstruction::Divide(_) => "div",
        GMInstruction::Remainder(_) => "rem",
        GMInstruction::Modulus(_) => "mod",
        GMInstruction::Add(_) => "add",
        GMInstruction::Subtract(_) => "sub",
        GMInstruction::And(_) => "and",
        GMInstruction::Or(_) => "or",
        GMInstruction::Xor(_) => "xor",
        GMInstruction::Negate(_) => "neg",
        GMInstruction::Not(_) => "not",
        GMInstruction::ShiftLeft(_) => "shl",
        GMInstruction::ShiftRight(_) => "shr",
        GMInstruction::Compare(_) => "cmp",
        GMInstruction::Pop(_) => "pop",
        GMInstruction::PopSwap(_) => "popswap",
        GMInstruction::Duplicate(_) => "dup",
        GMInstruction::DuplicateSwap(_) => "dupswap",
        GMInstruction::Return(_) => "ret",
        GMInstruction::Exit(_) => "exit",
        GMInstruction::PopDiscard(_) => "popz",
        GMInstruction::Branch(_) => "jmp",
        GMInstruction::BranchIf(_) => "jt",
        GMInstruction::BranchUnless(_) => "jf",
        GMInstruction::PushWithContext(_) => "pushenv",
        GMInstruction::PopWithContext(_) => "popenv",
        GMInstruction::Push(_) => "push",
        GMInstruction::PushLocal(_) => "pushloc",
        GMInstruction::PushGlobal(_) => "pushglb",
        GMInstruction::PushBuiltin(_) => "pushbltn",
        GMInstruction::PushImmediate(_) => "pushim",
        GMInstruction::Call(_) => "call",
        GMInstruction::CallVariable(_) => "callvar",
        GMInstruction::Extended16(_) => "extended",     // this should never happen though
        GMInstruction::Extended32(_) => "extended",     // this should never happen though
        GMInstruction::ExtendedFunc(_) => "extended",   // this should never happen though
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


fn instance_type_to_string(gm_data: &GMData, instance_type: &GMInstanceType, variable_ref: GMRef<GMVariable>) -> Result<String, String> {
    Ok(match instance_type {
        GMInstanceType::Undefined => return Err("Did not expect Instance Type Undefined here; please report this error.".to_string()),
        GMInstanceType::Self_(Some(obj_ref)) => {
            let obj: &GMGameObject = obj_ref.resolve(&gm_data.game_objects.game_objects)?;
            let name: &String = obj.name.resolve(&gm_data.strings.strings)?;
            format!("self<{name}>")
        }
        GMInstanceType::Self_(None) => "self".to_string(),
        GMInstanceType::RoomInstance(instance_id) => format!("roominst<{instance_id}>"),
        GMInstanceType::Other => "other".to_string(),
        GMInstanceType::All => "all".to_string(),
        GMInstanceType::None => "none".to_string(),
        GMInstanceType::Global => "global".to_string(),
        GMInstanceType::Builtin => "builtin".to_string(),
        GMInstanceType::Local => format!("local<{}>", variable_ref.index),
        GMInstanceType::StackTop => "stacktop".to_string(),
        GMInstanceType::Argument => "arg".to_string(),
        GMInstanceType::Static => "static".to_string(),
    })
}


fn variable_type_to_string(variable_type: GMVariableType) -> &'static str {
    match variable_type {
        GMVariableType::Array => "[array]",
        GMVariableType::StackTop => "[stacktop]",
        GMVariableType::Normal => "",
        GMVariableType::Instance => "[instance]",
        GMVariableType::ArrayPushAF => "[arraypushaf]",
        GMVariableType::ArrayPopAF => "[arraypopaf]",
    }
}


fn variable_to_string(gm_data: &GMData, code_variable: &CodeVariable) -> Result<String, String> {
    let variable: &GMVariable = code_variable.variable.resolve(&gm_data.variables.variables)?;
    let name: &String = variable.name.resolve(&gm_data.strings.strings)?;
    if !is_valid_identifier(name) {
        return Err(format!("Invalid variable identifier: {}", format_literal_string(gm_data, variable.name)?))
    }

    let prefix: &str = if code_variable.is_int32 {"(variable)"} else {""};

    let instance_type: &GMInstanceType = if code_variable.instance_type != GMInstanceType::Undefined {
        &code_variable.instance_type
    } else {
        // TODO: this will not work with b14
        variable.b15_data.as_ref().map_or(&GMInstanceType::Undefined, |b15| &b15.instance_type)
    };
    let instance_type: String = instance_type_to_string(gm_data, instance_type, code_variable.variable)?;

    let variable_type: &str = variable_type_to_string(code_variable.variable_type);

    Ok(format!("{prefix}{variable_type}{instance_type}.{name}"))
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


fn extended_kind_to_string(extended_kind: GMExtendedKind) -> &'static str {
    match extended_kind {
        GMExtendedKind::CheckArrayIndex => "chkindex",
        GMExtendedKind::PushArrayFinal => "pushaf",
        GMExtendedKind::PopArrayFinal => "popaf",
        GMExtendedKind::PushArrayContainer => "pushac",
        GMExtendedKind::SetArrayOwner => "setowner",
        GMExtendedKind::HasStaticInitialized => "isstaticok",
        GMExtendedKind::SetStaticInitialized => "setstatic",
        GMExtendedKind::SaveArrayReference => "savearef",
        GMExtendedKind::RestoreArrayReference => "restorearef",
        GMExtendedKind::IsNullishValue => "isknullish",
        GMExtendedKind::PushReference => "pushref",
    }
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

