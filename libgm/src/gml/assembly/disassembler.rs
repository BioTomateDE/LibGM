use crate::{
    gamemaker::{
        data::GMData,
        elements::{functions::GMFunction, game_objects::GMGameObject, variables::GMVariable},
        reference::GMRef,
    },
    gml::instructions::{
        CodeVariable, GMAssetReference, GMCode, GMCodeValue, GMComparisonType, GMDataType,
        GMInstanceType, GMInstruction, GMVariableType,
    },
    prelude::*,
    util::fmt::typename,
};

pub fn disassemble_code(gm_data: &GMData, code: &GMCode) -> Result<String> {
    disassemble_instructions(gm_data, &code.instructions)
}

pub fn disassemble_instructions(
    gm_data: &GMData,
    instructions: &[GMInstruction],
) -> Result<String> {
    let mut assembly: String = String::new();

    for instruction in instructions {
        let line: String = disassemble_instruction(gm_data, instruction)?;
        assembly += &line;
        assembly += "\n";
    }

    Ok(assembly)
}

pub fn disassemble_instruction(gm_data: &GMData, instruction: &GMInstruction) -> Result<String> {
    let line: String;
    let opcode: &str = opcode_to_string(instruction);

    match &instruction {
        GMInstruction::Exit
        | GMInstruction::Return
        | GMInstruction::PopSwap { .. }
        | GMInstruction::PopWithContextExit
        | GMInstruction::CheckArrayIndex
        | GMInstruction::PushArrayFinal
        | GMInstruction::PopArrayFinal
        | GMInstruction::PushArrayContainer
        | GMInstruction::SetArrayOwner
        | GMInstruction::HasStaticInitialized
        | GMInstruction::SetStaticInitialized
        | GMInstruction::SaveArrayReference
        | GMInstruction::RestoreArrayReference
        | GMInstruction::IsNullishValue => {
            line = opcode.to_string();
        },

        GMInstruction::Negate { data_type }
        | GMInstruction::Not { data_type }
        | GMInstruction::PopDiscard { data_type } => {
            line = format!("{}.{}", opcode, data_type_to_string(*data_type));
        },

        GMInstruction::CallVariable { argument_count } => {
            line = format!("{opcode} {argument_count}");
        },

        GMInstruction::Duplicate { data_type, size } => {
            line = format!("{}.{} {}", opcode, data_type_to_string(*data_type), size);
        },

        GMInstruction::DuplicateSwap { data_type, size1, size2 } => {
            line = format!(
                "{}.{} {} {}",
                opcode,
                data_type_to_string(*data_type),
                size1,
                size2,
            );
        },

        GMInstruction::Branch { jump_offset }
        | GMInstruction::BranchIf { jump_offset }
        | GMInstruction::BranchUnless { jump_offset }
        | GMInstruction::PushWithContext { jump_offset }
        | GMInstruction::PopWithContext { jump_offset } => {
            line = format!("{opcode} {jump_offset}");
        },

        GMInstruction::Convert { from: type1, to: type2 }
        | GMInstruction::Multiply { multiplicand: type2, multiplier: type1 }
        | GMInstruction::Divide { dividend: type2, divisor: type1 }
        | GMInstruction::Remainder { dividend: type2, divisor: type1 }
        | GMInstruction::Modulus { dividend: type2, divisor: type1 }
        | GMInstruction::Add { augend: type2, addend: type1 }
        | GMInstruction::Subtract { minuend: type2, subtrahend: type1 }
        | GMInstruction::And { lhs: type2, rhs: type1 }
        | GMInstruction::Or { lhs: type2, rhs: type1 }
        | GMInstruction::Xor { lhs: type2, rhs: type1 }
        | GMInstruction::ShiftLeft { value: type2, shift_amount: type1 }
        | GMInstruction::ShiftRight { value: type2, shift_amount: type1 } => {
            line = format!(
                "{}.{}.{}",
                opcode,
                data_type_to_string(*type1),
                data_type_to_string(*type2),
            );
        },

        GMInstruction::Compare { lhs, rhs, comparison_type } => {
            line = format!(
                "{}.{}.{} {}",
                opcode,
                data_type_to_string(*rhs),
                data_type_to_string(*lhs),
                comparison_type_to_string(*comparison_type),
            );
        },

        GMInstruction::Pop { variable, type1, type2 } => {
            // TODO: find the instance type of the variable
            line = format!(
                "{}.{}.{} {}",
                opcode,
                data_type_to_string(*type1),
                data_type_to_string(*type2),
                variable_to_string(gm_data, variable)?,
            );
        },

        GMInstruction::Push { value } => {
            let literal: String = match value {
                GMCodeValue::Variable(code_variable) => variable_to_string(gm_data, code_variable)?,
                GMCodeValue::Boolean(true) => "true".to_string(),
                GMCodeValue::Boolean(false) => "false".to_string(),
                GMCodeValue::Function(function_ref) => {
                    format!("(function){}", function_to_string(gm_data, *function_ref)?)
                },
                GMCodeValue::String(string) => format_literal_string(string.clone())?,
                GMCodeValue::Int16(integer) => integer.to_string(),
                GMCodeValue::Int32(integer) => integer.to_string(),
                GMCodeValue::Int64(integer) => integer.to_string(),
                GMCodeValue::Double(float) => float.to_string(),
            };

            line = format!(
                "{}.{} {}",
                opcode,
                data_type_to_string(value.data_type()),
                literal,
            );
        },
        GMInstruction::PushLocal { variable }
        | GMInstruction::PushGlobal { variable }
        | GMInstruction::PushBuiltin { variable } => {
            line = format!("{} {}", opcode, variable_to_string(gm_data, variable)?);
        },

        GMInstruction::PushImmediate { integer } => {
            line = format!("{opcode} {integer}");
        },

        GMInstruction::Call { function, argument_count } => {
            line = format!(
                "{} {}(argc={})",
                opcode,
                function_to_string(gm_data, *function)?,
                argument_count,
            );
        },

        GMInstruction::PushReference { asset_reference } => {
            line = format!(
                "{} {}",
                opcode,
                asset_reference_to_string(gm_data, asset_reference)?,
            );
        },
    }

    Ok(line)
}

#[must_use]
const fn opcode_to_string(instruction: &GMInstruction) -> &'static str {
    match instruction {
        GMInstruction::Convert { .. } => "conv",
        GMInstruction::Multiply { .. } => "mul",
        GMInstruction::Divide { .. } => "div",
        GMInstruction::Remainder { .. } => "rem",
        GMInstruction::Modulus { .. } => "mod",
        GMInstruction::Add { .. } => "add",
        GMInstruction::Subtract { .. } => "sub",
        GMInstruction::And { .. } => "and",
        GMInstruction::Or { .. } => "or",
        GMInstruction::Xor { .. } => "xor",
        GMInstruction::Negate { .. } => "neg",
        GMInstruction::Not { .. } => "not",
        GMInstruction::ShiftLeft { .. } => "shl",
        GMInstruction::ShiftRight { .. } => "shr",
        GMInstruction::Compare { .. } => "cmp",
        GMInstruction::Pop { .. } => "pop",
        GMInstruction::PopSwap { is_array: false } => "popswap",
        GMInstruction::PopSwap { is_array: true } => "popswaparr",
        GMInstruction::Duplicate { .. } => "dup",
        GMInstruction::DuplicateSwap { .. } => "dupswap",
        GMInstruction::Return => "ret",
        GMInstruction::Exit => "exit",
        GMInstruction::PopDiscard { .. } => "popz",
        GMInstruction::Branch { .. } => "jmp",
        GMInstruction::BranchIf { .. } => "jt",
        GMInstruction::BranchUnless { .. } => "jf",
        GMInstruction::PushWithContext { .. } => "pushenv",
        GMInstruction::PopWithContext { .. } => "popenv",
        GMInstruction::PopWithContextExit => "popenvexit",
        GMInstruction::Push { .. } => "push",
        GMInstruction::PushLocal { .. } => "pushloc",
        GMInstruction::PushGlobal { .. } => "pushglb",
        GMInstruction::PushBuiltin { .. } => "pushbltn",
        GMInstruction::PushImmediate { .. } => "pushim",
        GMInstruction::Call { .. } => "call",
        GMInstruction::CallVariable { .. } => "callvar",
        GMInstruction::CheckArrayIndex => "chkindex",
        GMInstruction::PushArrayFinal => "pushaf",
        GMInstruction::PopArrayFinal => "popaf",
        GMInstruction::PushArrayContainer => "pushac",
        GMInstruction::SetArrayOwner => "setowner",
        GMInstruction::HasStaticInitialized => "isstaticok",
        GMInstruction::SetStaticInitialized => "setstatic",
        GMInstruction::SaveArrayReference => "savearef",
        GMInstruction::RestoreArrayReference => "restorearef",
        GMInstruction::IsNullishValue => "isnullish",
        GMInstruction::PushReference { .. } => "pushref",
    }
}

fn asset_get_name<T>(
    gm_elements: &Vec<T>,
    gm_ref: GMRef<T>,
    get_name: impl FnOnce(&T) -> &String,
) -> Result<&String> {
    let element: &T = gm_ref
        .resolve(gm_elements)
        .context("resolving asset reference for PushReference Instruction")?;

    let name: &String = get_name(element);

    if !is_valid_identifier(name) {
        bail!("Invalid {} identifier: {:?}", typename::<T>(), name)
    }
    Ok(name)
}

fn asset_reference_to_string(gm_data: &GMData, asset_ref: &GMAssetReference) -> Result<String> {
    Ok(match asset_ref {
        &GMAssetReference::Object(gm_ref) => {
            "(object)".to_string() + asset_get_name(&gm_data.game_objects, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Sprite(gm_ref) => {
            "(sprite)".to_string() + asset_get_name(&gm_data.sprites, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Sound(gm_ref) => {
            "(sound)".to_string() + asset_get_name(&gm_data.sounds, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Room(gm_ref) => {
            "(sprite)".to_string() + asset_get_name(&gm_data.rooms, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Background(gm_ref) => {
            "(background)".to_string() + asset_get_name(&gm_data.backgrounds, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Path(gm_ref) => {
            "(path)".to_string() + asset_get_name(&gm_data.paths, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Script(gm_ref) => {
            "(script)".to_string() + asset_get_name(&gm_data.scripts, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Font(gm_ref) => {
            "(font)".to_string() + asset_get_name(&gm_data.fonts, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Timeline(gm_ref) => {
            "(timeline)".to_string() + asset_get_name(&gm_data.timelines, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Shader(gm_ref) => {
            "(shader)".to_string() + asset_get_name(&gm_data.shaders, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::Sequence(gm_ref) => {
            "(sequence)".to_string() + asset_get_name(&gm_data.sequences, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::AnimCurve(gm_ref) => {
            "(animcurve)".to_string()
                + asset_get_name(&gm_data.animation_curves, gm_ref, |x| &x.name)?
        },
        &GMAssetReference::ParticleSystem(gm_ref) => {
            "(particlesys".to_string()
                + asset_get_name(&gm_data.particle_systems, gm_ref, |x| &x.name)?
        },
        GMAssetReference::RoomInstance(id) => format!("(roominstance){id}"),
        GMAssetReference::Function(gm_ref) => {
            format!("(function){}", function_to_string(gm_data, *gm_ref)?)
        },
    })
}

#[must_use]
const fn data_type_to_string(data_type: GMDataType) -> &'static str {
    match data_type {
        GMDataType::Int16 => "e",
        GMDataType::Int32 => "i",
        GMDataType::Int64 => "l",
        GMDataType::Double => "d",
        GMDataType::Boolean => "b",
        GMDataType::String => "s",
        GMDataType::Variable => "v",
    }
}

#[must_use]
const fn comparison_type_to_string(comparison_type: GMComparisonType) -> &'static str {
    match comparison_type {
        GMComparisonType::LessThan => "LT",
        GMComparisonType::LessOrEqual => "LTE",
        GMComparisonType::Equal => "EQ",
        GMComparisonType::NotEqual => "NEQ",
        GMComparisonType::GreaterOrEqual => "GTE",
        GMComparisonType::GreaterThan => "GT",
    }
}

fn instance_type_to_string(
    gm_data: &GMData,
    instance_type: &GMInstanceType,
    variable_ref: GMRef<GMVariable>,
) -> Result<String> {
    Ok(match instance_type {
        GMInstanceType::Undefined => {
            unreachable!("Did not expect Instance Type Undefined here; please report this error")
        },
        GMInstanceType::Self_(Some(obj_ref)) => {
            let obj: &GMGameObject = obj_ref.resolve(&gm_data.game_objects)?;
            format!("self<{}>", obj.name)
        },
        GMInstanceType::Self_(None) => "self".to_string(),
        GMInstanceType::RoomInstance(instance_id) => {
            format!("roominstance<{instance_id}>")
        },
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

const fn variable_type_to_string(variable_type: GMVariableType) -> &'static str {
    match variable_type {
        GMVariableType::Array => "[array]",
        GMVariableType::StackTop => "[stacktop]",
        GMVariableType::Normal => "",
        GMVariableType::Instance => "",
        GMVariableType::ArrayPushAF => "[arraypushaf]",
        GMVariableType::ArrayPopAF => "[arraypopaf]",
    }
}

fn variable_to_string(gm_data: &GMData, code_variable: &CodeVariable) -> Result<String> {
    let variable: &GMVariable = code_variable.variable.resolve(&gm_data.variables)?;
    let name = &variable.name;
    if !is_valid_identifier(name) && name != "$$$$temp$$$$" {
        bail!("Invalid variable identifier {name:?}");
    }

    let prefix: &str = if code_variable.is_int32 {
        "(variable)"
    } else {
        ""
    };

    let instance_type: &GMInstanceType = if code_variable.instance_type == GMInstanceType::Undefined {
        // TODO: this will not work with b14
        variable
            .b15_data
            .as_ref()
            .map_or(&GMInstanceType::Undefined, |b15| &b15.instance_type)
    } else {
        &code_variable.instance_type
    };
    let instance_type: String =
        instance_type_to_string(gm_data, instance_type, code_variable.variable)?;

    let variable_type: &str = variable_type_to_string(code_variable.variable_type);

    Ok(format!("{prefix}{variable_type}{instance_type}.{name}"))
}

fn function_to_string(gm_data: &GMData, function_ref: GMRef<GMFunction>) -> Result<&String> {
    let function: &GMFunction = function_ref.resolve(&gm_data.functions)?;
    let name = &function.name;
    if !is_valid_identifier(name) {
        let is_special = name.starts_with("@@")
            && name.ends_with("@@")
            && is_valid_identifier(&name[2..name.len() - 2]);

        if !is_special {
            bail!("Invalid function identifier: {name:?}");
        }
    }
    Ok(name)
}

pub(super) fn format_literal_string(string: String) -> Result<String> {
    let string: String = string
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('"', "\\\"");
    Ok(format!("\"{string}\""))
}

/// Check whether an identifier / asset name is valid for assembling properly.
/// Exceptions like `$$$$temp$$$$` for variables or `@@This@@` for functions will have to be handled separately.
/// ## Rules:
/// - At least one character long
/// - First character is not a digit
/// - Letters and underscores are allowed
/// - Only ascii characters
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
