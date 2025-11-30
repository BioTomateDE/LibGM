mod data_types;
mod reader;

use std::{fmt::Display, str::FromStr};

use crate::{
    gamemaker::{
        data::GMData,
        elements::{
            functions::{GMFunction, GMFunctions},
            game_objects::GMGameObject,
            variables::GMVariable,
        },
        reference::GMRef,
    },
    gml::{
        assembly::assembler::{data_types::DataTypes, reader::Reader},
        instructions::{
            CodeVariable, GMAssetReference, GMCodeValue, GMComparisonType, GMDataType,
            GMInstanceType, GMInstruction, GMVariableType,
        },
    },
    prelude::*,
    util::fmt::typename,
};

pub fn assemble_code(assembly: &str, gm_data: &GMData) -> Result<Vec<GMInstruction>> {
    let mut instructions: Vec<GMInstruction> = Vec::new();

    for line in assembly.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let instruction: GMInstruction = assemble_instruction(line, gm_data)
            .with_context(|| format!("assembling instruction: {line}"))?;
        instructions.push(instruction);
    }

    Ok(instructions)
}

pub fn assemble_instruction(line: &str, gm_data: &GMData) -> Result<GMInstruction> {
    let mut reader = Reader::new(line.trim());
    let mnemonic: String;

    let opcode_end: Option<usize> = reader.line.find(['.', ' ']);

    if let Some(index) = opcode_end {
        mnemonic = reader.consume_to(index).to_string();
    } else {
        // Opcode takes up entire line
        mnemonic = reader.clear().to_string();
    }

    let mut types = DataTypes::new();
    while reader.starts_with(".") {
        reader.consume_dot()?;
        let raw_type: char = reader
            .consume_char()
            .ok_or("Unexpected EOL when trying to parse instruction data type")?;
        let data_type = GMDataType::from_char(raw_type)?;
        types.push(data_type)?;
    }

    match reader.peek_char() {
        Some(' ') => {
            reader.consume_space()?;
        },
        None => {},
        _ => bail!("Expected space; found remaining string {line:?}"),
    }

    let instruction = parse_instruction(&mut reader, &mnemonic, types, gm_data)?;

    if !reader.is_empty() {
        bail!(
            "Expected end of line; found remaining string {:?}",
            reader.line
        )
    }

    Ok(instruction)
}

fn parse_instruction(
    reader: &mut Reader,
    mnemonic: &str,
    types: DataTypes,
    gm_data: &GMData,
) -> Result<GMInstruction> {
    let instruction = match mnemonic {
        "conv" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Convert { from: types[0], to: types[1] }
        },
        "mul" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Multiply {
                multiplicand: types[1],
                multiplier: types[0],
            }
        },
        "div" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Divide { dividend: types[1], divisor: types[0] }
        },
        "rem" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Remainder { dividend: types[1], divisor: types[0] }
        },
        "mod" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Modulus { dividend: types[1], divisor: types[0] }
        },
        "add" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Add { augend: types[1], addend: types[0] }
        },
        "sub" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Subtract { minuend: types[1], subtrahend: types[0] }
        },
        "and" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::And { lhs: types[1], rhs: types[0] }
        },
        "or" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Or { lhs: types[1], rhs: types[0] }
        },
        "xor" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::Xor { lhs: types[1], rhs: types[0] }
        },
        "neg" => {
            types.assert_count(1, mnemonic)?;
            GMInstruction::Negate { data_type: types[0] }
        },
        "not" => {
            types.assert_count(1, mnemonic)?;
            GMInstruction::Not { data_type: types[0] }
        },
        "shl" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::ShiftLeft { value: types[1], shift_amount: types[0] }
        },
        "shr" => {
            types.assert_count(2, mnemonic)?;
            GMInstruction::ShiftRight { value: types[1], shift_amount: types[0] }
        },
        "cmp" => parse_comparison(types, reader)?,
        "pop" => {
            types.assert_count(2, mnemonic)?;
            let variable: CodeVariable = parse_variable(reader, gm_data)?;
            GMInstruction::Pop {
                type1: types[0],
                type2: types[1],
                variable,
            }
        },
        "popswap" => {
            types.assert_count(0, mnemonic)?;
            GMInstruction::PopSwap { is_array: false }
        },
        "popswaparr" => {
            types.assert_count(0, mnemonic)?;
            GMInstruction::PopSwap { is_array: true }
        },
        "dup" => parse_duplicate(types, reader)?,
        "dupswap" => parse_duplicate_swap(types, reader)?,
        "ret" => {
            types.assert_count(0, mnemonic)?;
            GMInstruction::Return
        },
        "exit" => {
            types.assert_count(0, mnemonic)?;
            GMInstruction::Exit
        },
        "popz" => {
            types.assert_count(1, mnemonic)?;
            GMInstruction::PopDiscard { data_type: types[0] }
        },
        "jmp" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = reader.parse_int()?;
            GMInstruction::Branch { jump_offset }
        },
        "jt" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = reader.parse_int()?;
            GMInstruction::BranchIf { jump_offset }
        },
        "jf" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = reader.parse_int()?;
            GMInstruction::BranchUnless { jump_offset }
        },
        "pushenv" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = reader.parse_int()?;
            GMInstruction::PushWithContext { jump_offset }
        },
        "popenv" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = reader.parse_int()?;
            GMInstruction::PopWithContext { jump_offset }
        },
        "popenvexit" => {
            types.assert_count(0, mnemonic)?;
            GMInstruction::PopWithContextExit
        },
        "push" => GMInstruction::Push {
            value: parse_push(types, reader, gm_data)?,
        },
        "pushloc" => {
            types.assert_count(0, mnemonic)?;
            let variable = parse_variable(reader, gm_data)?;
            GMInstruction::PushLocal { variable }
        },
        "pushglb" => {
            types.assert_count(0, mnemonic)?;
            let variable = parse_variable(reader, gm_data)?;
            GMInstruction::PushGlobal { variable }
        },
        "pushbltn" => {
            types.assert_count(0, mnemonic)?;
            let variable = parse_variable(reader, gm_data)?;
            GMInstruction::PushBuiltin { variable }
        },
        "pushim" => {
            types.assert_count(0, mnemonic)?;
            let integer: i16 = reader.parse_int()?;
            GMInstruction::PushImmediate { integer }
        },
        "call" => parse_call(types, reader, gm_data)?,
        "callvar" => {
            types.assert_count(0, mnemonic)?;
            let argument_count: u16 = reader.parse_uint()?;
            GMInstruction::CallVariable { argument_count }
        },
        "chkindex" => GMInstruction::CheckArrayIndex,
        "pushaf" => GMInstruction::PushArrayFinal,
        "popaf" => GMInstruction::PopArrayFinal,
        "pushac" => GMInstruction::PushArrayContainer,
        "setowner" => GMInstruction::SetArrayOwner,
        "isstaticok" => GMInstruction::HasStaticInitialized,
        "setstatic" => GMInstruction::SetStaticInitialized,
        "savearef" => GMInstruction::SaveArrayReference,
        "restorearef" => GMInstruction::RestoreArrayReference,
        "isnullish" => GMInstruction::IsNullishValue,
        "pushref" => {
            types.assert_count(0, mnemonic)?;
            let asset_reference = parse_asset_reference(reader, gm_data)?;
            GMInstruction::PushReference { asset_reference }
        },
        _ => bail!("Invalid opcode mnemonic {mnemonic:?}"),
    };

    Ok(instruction)
}

fn asset_by_name<T>(
    reader: &mut Reader,
    elements: &Vec<T>,
    get_name: impl Fn(&T) -> &String,
) -> Result<GMRef<T>> {
    let target_name: &str = reader.parse_identifier()?;

    for (i, element) in elements.iter().enumerate() {
        if get_name(element) == target_name {
            return Ok(GMRef::from(i));
        }
    }

    bail!(
        "Could not resolve Asset of type {} with name {:?}",
        stringify!(T),
        target_name,
    );
}

fn parse_asset_reference(reader: &mut Reader, gm_data: &GMData) -> Result<GMAssetReference> {
    let line = reader.line;
    let asset_type = reader
        .consume_round_brackets()?
        .ok_or_else(|| format!("Expected asset type within round brackets; found {line:?}"))?;

    // This can probably be made cleaner
    #[rustfmt::skip]
    let asset_reference = match asset_type {
        "object" => GMAssetReference::Object(asset_by_name(reader, &gm_data.game_objects, |x| &x.name)?),
        "sprite" => GMAssetReference::Sprite(asset_by_name(reader, &gm_data.sprites, |x| &x.name)?),
        "sound" => GMAssetReference::Sound(asset_by_name(reader, &gm_data.sounds, |x| &x.name)?),
        "room" => GMAssetReference::Room(asset_by_name(reader, &gm_data.rooms, |x| &x.name)?),
        "background" => GMAssetReference::Background(asset_by_name(reader, &gm_data.backgrounds, |x| &x.name)?),
        "path" => GMAssetReference::Path(asset_by_name(reader, &gm_data.paths, |x| &x.name)?),
        "script" => GMAssetReference::Script(asset_by_name(reader, &gm_data.scripts, |x| &x.name)?),
        "font" => GMAssetReference::Font(asset_by_name(reader, &gm_data.fonts, |x| &x.name)?),
        "timeline" => GMAssetReference::Timeline(asset_by_name(reader, &gm_data.timelines, |x| &x.name)?),
        "shader" => GMAssetReference::Shader(asset_by_name(reader, &gm_data.shaders, |x| &x.name)?),
        "sequence" => GMAssetReference::Sequence(asset_by_name(reader, &gm_data.sequences, |x| &x.name)?),
        "animcurve" => GMAssetReference::AnimCurve(asset_by_name(reader, &gm_data.animation_curves, |x| &x.name)?), 
        "particlesystem" => GMAssetReference::ParticleSystem(asset_by_name(reader, &gm_data.particle_systems, |x| &x.name)?),
        "roominstance" => GMAssetReference::RoomInstance(reader.parse_int()?),
        "function" => GMAssetReference::Function(parse_function(reader, &gm_data.functions)?),
        _ => bail!("Invalid Type Cast to asset type {asset_type:?}"),
    };

    Ok(asset_reference)
}

fn parse_comparison(types: DataTypes, reader: &mut Reader) -> Result<GMInstruction> {
    types.assert_count(2, "cmp")?;
    let comparison_type: &str = reader.parse_identifier()?;
    let comparison_type = match comparison_type {
        "EQ" => GMComparisonType::Equal,
        "NEQ" => GMComparisonType::NotEqual,
        "LT" => GMComparisonType::LessThan,
        "LTE" => GMComparisonType::LessOrEqual,
        "GTE" => GMComparisonType::GreaterOrEqual,
        "GT" => GMComparisonType::GreaterThan,
        _ => bail!("Invalid Comparison Type {comparison_type:?}"),
    };
    Ok(GMInstruction::Compare {
        lhs: types[1],
        rhs: types[0],
        comparison_type,
    })
}

fn parse_duplicate(types: DataTypes, reader: &mut Reader) -> Result<GMInstruction> {
    types.assert_count(1, "dup")?;
    let size: u8 = reader.parse_uint()?;
    Ok(GMInstruction::Duplicate { data_type: types[0], size })
}

fn parse_duplicate_swap(types: DataTypes, reader: &mut Reader) -> Result<GMInstruction> {
    types.assert_count(1, "dupswap")?;
    let size1: u8 = reader.parse_uint()?;
    reader.consume_space()?;
    let size2: u8 = reader.parse_uint()?;
    Ok(GMInstruction::DuplicateSwap { data_type: types[0], size1, size2 })
}

fn parse_push(types: DataTypes, reader: &mut Reader, gm_data: &GMData) -> Result<GMCodeValue> {
    types.assert_count(1, "push")?;

    let value: GMCodeValue = match types[0] {
        GMDataType::Int16 => GMCodeValue::Int16(parse_int(reader.clear())?),
        GMDataType::Int32 => {
            if let Some(type_cast) = reader.consume_round_brackets()? {
                match type_cast {
                    "function" => {
                        GMCodeValue::Function(parse_function(reader, &gm_data.functions)?)
                    },
                    "variable" => {
                        let mut variable: CodeVariable = parse_variable(reader, gm_data)?;
                        variable.is_int32 = true;
                        GMCodeValue::Variable(variable)
                    },
                    _ => bail!(
                        "Invalid type cast {type_cast:?}; expected \"function\" or \"variable\""
                    ),
                }
            } else {
                GMCodeValue::Int32(parse_int(reader.clear())?)
            }
        },
        GMDataType::Int64 => GMCodeValue::Int64(parse_int(reader.clear())?),
        GMDataType::Double => {
            let line: &str = reader.clear();
            let float: f64 = line
                .parse()
                .ok()
                .ok_or("Invalid float literal {line:?}")?;
            GMCodeValue::Double(float)
        },
        GMDataType::Boolean => {
            let line: &str = reader.clear();
            let bool: bool = match line {
                "true" => true,
                "false" => false,
                _ => bail!("Invalid boolean {line:?}"),
            };
            GMCodeValue::Boolean(bool)
        },
        GMDataType::String => {
            let string: String = parse_string_literal(reader)?;
            GMCodeValue::String(string)
        },
        GMDataType::Variable => GMCodeValue::Variable(parse_variable(reader, gm_data)?),
    };
    Ok(value)
}

fn parse_call(types: DataTypes, reader: &mut Reader, gm_data: &GMData) -> Result<GMInstruction> {
    types.assert_count(0, "call")?;
    let function: GMRef<GMFunction> = parse_function(reader, &gm_data.functions)?;

    let line = reader.line;
    let argc_str: &str = reader.consume_round_brackets()?.ok_or_else(|| {
        format!("Expected round brackets with argument count for function call; found {line:?}",)
    })?;

    let argument_count: u16 = if let Some(str) = argc_str.strip_prefix("argc=") {
        str.parse()
            .ok()
            .ok_or_else(|| format!("Invalid argument count {str}"))?
    } else {
        bail!(
            "Expected \"argc=\" for function call parameters; found {:?}",
            reader.line
        );
    };
    Ok(GMInstruction::Call { function, argument_count })
}

impl GMVariableType {
    fn from_string(variable_type: &str) -> Result<Self> {
        Ok(match variable_type {
            "stacktop" => Self::StackTop,
            "array" => Self::Array,
            "roominstance" => Self::Instance,
            "arraypushaf" => Self::ArrayPushAF,
            "arraypopaf" => Self::ArrayPopAF,
            _ => bail!("Invalid Variable Reference Type {variable_type:?}"),
        })
    }
}

fn parse_variable(reader: &mut Reader, gm_data: &GMData) -> Result<CodeVariable> {
    let mut variable_type = GMVariableType::Normal;
    if let Some(variable_type_str) = reader.consume_square_brackets()? {
        variable_type = GMVariableType::from_string(variable_type_str)?;
    }

    let instance_type_raw = reader.parse_identifier()?.to_string();
    let instance_type_arg = reader
        .consume_angle_brackets()?
        .unwrap_or_default()
        .to_string();
    reader.consume_dot()?;

    let mut variable_ref: Option<GMRef<GMVariable>> = None;
    let instance_type: GMInstanceType = match instance_type_raw.as_str() {
        "self" if instance_type_arg.is_empty() => GMInstanceType::Self_(None),
        "self" => {
            let object_ref: GMRef<GMGameObject> =
                gm_data.game_objects.get_ref_by_name(&instance_type_arg)?;
            GMInstanceType::Self_(Some(object_ref))
        },
        "local" => {
            let var_index: u32 = parse_int(&instance_type_arg)?;
            variable_ref = Some(GMRef::new(var_index));
            GMInstanceType::Local
        },
        "roominstance" => {
            variable_type = GMVariableType::Instance;
            let instance_id: i16 = parse_int(&instance_type_arg)?;
            GMInstanceType::RoomInstance(instance_id)
        },
        "stacktop" => GMInstanceType::StackTop,
        "builtin" => GMInstanceType::Builtin,
        "global" => GMInstanceType::Global,
        "arg" => GMInstanceType::Argument,
        "other" => GMInstanceType::Other,
        "static" => GMInstanceType::Static,
        "all" => GMInstanceType::All,
        "none" => GMInstanceType::None,
        _ => bail!("Invalid Instance Type {instance_type_raw:?}"),
    };

    let name: &str = parse_variable_identifier(reader)?;

    if instance_type != GMInstanceType::Local {
        // Convert instance type because of some bullshit
        let vari_instance_type: GMInstanceType = instance_type.as_vari();

        for (i, var) in gm_data.variables.iter().enumerate() {
            if var.name != name {
                continue;
            }
            if let Some(b15) = &var.b15_data
                && b15.instance_type != vari_instance_type
            {
                continue;
            }
            // Found var
            variable_ref = Some(GMRef::new(i as u32));
            break;
        }
    }

    let Some(variable) = variable_ref else {
        bail!("Cannot resolve variable with name {name:?}");
    };

    // I need to throw away the instance type so that the tests pass
    let mut instance_type = instance_type;
    if variable_type != GMVariableType::Normal && variable_type != GMVariableType::Instance {
        instance_type = GMInstanceType::Undefined;
    } // TODO: comment out this block if not testing assembler

    Ok(CodeVariable {
        variable,
        variable_type,
        instance_type,
        is_int32: false, // This has to be modified afterward, if necessary
    })
}

fn parse_variable_identifier<'a>(reader: &'a mut Reader) -> Result<&'a str> {
    // TODO: This path can be marked as cold
    if reader.consume_str("$$$$temp$$$$").is_some() {
        Ok("$$$$temp$$$$")
    } else {
        reader.parse_identifier()
    }
}

fn parse_function(reader: &mut Reader, gm_functions: &GMFunctions) -> Result<GMRef<GMFunction>> {
    let identifier = parse_function_identifier(reader)?;

    for (i, func) in gm_functions.iter().enumerate() {
        if func.name == identifier {
            return Ok(i.into());
        }
    }

    bail!("Function {identifier:?} does not exist")
}

fn parse_function_identifier(reader: &mut Reader) -> Result<String> {
    // Try standard identifier first
    let error = match reader.parse_identifier() {
        Ok(ident) => return Ok(ident.to_string()),
        Err(err) => err,
    };

    // Try special @@identifier@@ syntax
    if reader.consume_str("@@").is_some()
        && let Ok(ident) = reader.parse_identifier() {
            let ident = ident.to_string();
            if reader.consume_str("@@").is_some() {
                return Ok(format!("@@{ident}@@"));
            }
        }

    // If neither works, return the original parse error
    Err(error)
}

fn parse_int<T: FromStr>(string: &str) -> Result<T>
where
    <T as FromStr>::Err: Display,
{
    match string.parse() {
        Ok(int) => Ok(int),
        Err(err) => {
            bail!("Invalid {} Integer {}: {}", typename::<T>(), string, err);
        },
    }
}

/// Assumes the entire rest of the line is the string literal
fn parse_string_literal(reader: &mut Reader) -> Result<String> {
    let line = reader.line;
    if reader.consume_char() != Some('"') {
        bail!("Expected string literal; found {line:?}");
    }

    let mut escaping: bool = false;
    let mut string: String = String::with_capacity(reader.line.len());

    for (i, char) in reader.line.char_indices() {
        if escaping {
            match char {
                '"' => string.push('"'),
                '\\' => string.push('\\'),
                'n' => string.push('\n'),
                't' => string.push('\t'),
                'r' => string.push('\r'),
                _ => bail!("Invalid escape character '{char}'"),
            }
            escaping = false;
        } else if char == '"' {
            reader.consume_to(i + 1);
            return Ok(string);
        } else if char == '\\' {
            escaping = true;
        } else {
            string.push(char);
        }
    }

    bail!("String literal's quotation marks were never closed")
}
