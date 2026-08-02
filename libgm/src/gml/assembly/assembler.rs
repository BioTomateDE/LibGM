// SPDX-License-Identifier: GPL-3.0-only
// TODO: abysmal performance
mod data_types;
mod reader;

use std::str::FromStr;

use crate::gml::assembly::assembler::data_types::DataTypes;
use crate::gml::assembly::assembler::reader::Reader;
use crate::gml::instruction::AssetReference;
use crate::gml::instruction::BinaryOp;
use crate::gml::instruction::CodeVariable;
use crate::gml::instruction::ComparisonType;
use crate::gml::instruction::DataType;
use crate::gml::instruction::InstanceType;
use crate::gml::instruction::Instruction;
use crate::gml::instruction::PushValue;
use crate::gml::instruction::ReferenceType;
use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::data::GMData;
use crate::wad::elem::function::Function;
use crate::wad::elem::function::Functions;
use crate::wad::elem::game_object::GameObject;
use crate::wad::elem::room::InstanceID;
use crate::wad::elem::string::Strings;
use crate::wad::elem::validate_identifier;
use crate::wad::elem::variable::Variable;

/// Assembles multiple instructions separated by newline.
///
/// Empty lines, and lines containing only whitespace/comments are skipped.
/// Comments start with the `#` character and reach until the end of the line.
pub fn assemble_instructions(assembly: &str, gm_data: &GMData) -> Result<Vec<Instruction>> {
    let heuristic = assembly.lines().count();
    let mut instructions: Vec<Instruction> = Vec::with_capacity(heuristic);

    for line in assembly.lines() {
        let line = line.trim_start();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let instruction: Instruction = assemble_instruction(line, gm_data)
            .ctx(|| format!("assembling instruction: {line}"))?;
        instructions.push(instruction);
    }

    Ok(instructions)
}

/// Assembles a single instruction on one line.
pub fn assemble_instruction(line: &str, gm_data: &GMData) -> Result<Instruction> {
    let mut reader = Reader::new(line.trim_start());
    let mnemonic: &str;

    let opcode_end: Option<usize> = reader.line.find(['.', ' ', '#']);
    if let Some(index) = opcode_end {
        mnemonic = reader.consume_to(index);
    } else {
        // Opcode takes up entire line
        mnemonic = reader.clear();
    }

    let mut types = DataTypes::new();
    while reader.starts_with(".") {
        reader.consume_dot()?;
        let raw_type: char = reader
            .consume_char()
            .ok_or("Unexpected EOL when trying to parse instruction data type")?;
        let data_type = DataType::from_char(raw_type)?;
        types.push(data_type)?;
    }

    match reader.peek_char() {
        Some(' ') => {
            reader.consume_space()?;
        }
        None => {}
        _ => bail!("Expected space; found remaining string {line:?}"),
    }

    let instruction = parse_instruction(&mut reader, mnemonic, types, gm_data)?;

    if !reader.is_empty() && !reader.line.trim_start().starts_with('#') {
        bail!("Expected end of line; found remaining string {:?}", reader.line);
    }

    Ok(instruction)
}

#[expect(clippy::too_many_lines)]
fn parse_instruction(
    reader: &mut Reader,
    mnemonic: &str,
    types: DataTypes,
    gm_data: &GMData,
) -> Result<Instruction> {
    let instruction = match mnemonic {
        "add" => parse_binary(types, mnemonic, BinaryOp::Add)?,
        "sub" => parse_binary(types, mnemonic, BinaryOp::Subtract)?,
        "mul" => parse_binary(types, mnemonic, BinaryOp::Multiply)?,
        "div" => parse_binary(types, mnemonic, BinaryOp::Divide)?,
        "rem" => parse_binary(types, mnemonic, BinaryOp::Remainder)?,
        "mod" => parse_binary(types, mnemonic, BinaryOp::Modulus)?,
        "and" => parse_binary(types, mnemonic, BinaryOp::And)?,
        "or" => parse_binary(types, mnemonic, BinaryOp::Or)?,
        "xor" => parse_binary(types, mnemonic, BinaryOp::Xor)?,
        "shl" => parse_binary(types, mnemonic, BinaryOp::ShiftLeft)?,
        "shr" => parse_binary(types, mnemonic, BinaryOp::ShiftRight)?,
        "neg" => {
            types.assert_count(1, mnemonic)?;
            Instruction::Negate { data_type: types[0] }
        }
        "not" => {
            types.assert_count(1, mnemonic)?;
            Instruction::Not { data_type: types[0] }
        }
        "conv" => {
            types.assert_count(2, mnemonic)?;
            Instruction::Convert { from: types[0], to: types[1] }
        }
        "cmp" => parse_comparison(types, reader)?,
        "pop" => {
            types.assert_count(2, mnemonic)?;
            let variable: CodeVariable = parse_variable(reader, gm_data)?;
            Instruction::Pop { type1: types[0], type2: types[1], variable }
        }
        "popswap" => {
            types.assert_count(0, mnemonic)?;
            Instruction::PopSwap { is_array: false }
        }
        "popswaparr" => {
            types.assert_count(0, mnemonic)?;
            Instruction::PopSwap { is_array: true }
        }
        "dup" => parse_duplicate(types, reader)?,
        "dupswap" => parse_duplicate_swap(types, reader)?,
        "ret" => {
            types.assert_count(0, mnemonic)?;
            Instruction::Return
        }
        "exit" => {
            types.assert_count(0, mnemonic)?;
            Instruction::Exit
        }
        "popz" => {
            types.assert_count(1, mnemonic)?;
            Instruction::PopDiscard { data_type: types[0] }
        }
        "br" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = parse_int(reader.clear())?;
            Instruction::Branch { jump_offset }
        }
        "bt" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = parse_int(reader.clear())?;
            Instruction::BranchIf { jump_offset }
        }
        "bf" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = parse_int(reader.clear())?;
            Instruction::BranchUnless { jump_offset }
        }
        "pushenv" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = parse_int(reader.clear())?;
            Instruction::PushWithContext { jump_offset }
        }
        "popenv" => {
            types.assert_count(0, mnemonic)?;
            let jump_offset: i32 = parse_int(reader.clear())?;
            Instruction::PopWithContext { jump_offset }
        }
        "popenvx" => {
            types.assert_count(0, mnemonic)?;
            Instruction::PopWithContextExit
        }
        "push" => Instruction::Push { value: parse_push(types, reader, gm_data)? },
        "pushloc" => {
            types.assert_count(0, mnemonic)?;
            let variable = parse_variable(reader, gm_data)?;
            Instruction::PushLocal { variable }
        }
        "pushglb" => {
            types.assert_count(0, mnemonic)?;
            let variable = parse_variable(reader, gm_data)?;
            Instruction::PushGlobal { variable }
        }
        "pushbltn" => {
            types.assert_count(0, mnemonic)?;
            let variable = parse_variable(reader, gm_data)?;
            Instruction::PushBuiltin { variable }
        }
        "pushim" => {
            types.assert_count(0, mnemonic)?;
            let integer: i16 = parse_int(reader.clear())?;
            Instruction::PushImmediate { integer }
        }
        "call" => parse_call(types, reader, gm_data)?,
        "callvar" => {
            types.assert_count(0, mnemonic)?;
            let arg_count: u16 = parse_int(reader.clear())?;
            Instruction::CallVariable { arg_count }
        }
        "chkindex" => Instruction::CheckArrayIndex,
        "pushaf" => Instruction::PushArrayFinal,
        "popaf" => Instruction::PopArrayFinal,
        "pushac" => Instruction::PushArrayContainer,
        "setowner" => Instruction::SetArrayOwner,
        "isstaticok" => Instruction::HasStaticInitialized,
        "setstatic" => Instruction::SetStaticInitialized,
        "savearef" => Instruction::SaveArrayReference,
        "restorearef" => Instruction::RestoreArrayReference,
        "isnullish" => Instruction::IsNullishValue,
        "pushref" => {
            types.assert_count(0, mnemonic)?;
            let asset_reference = parse_asset_reference(reader)?;
            Instruction::PushReference { asset_reference }
        }
        _ => bail!("Invalid opcode mnemonic {mnemonic:?}"),
    };

    Ok(instruction)
}

fn parse_binary(types: DataTypes, mnemonic: &str, op: BinaryOp) -> Result<Instruction> {
    types.assert_count(2, mnemonic)?;
    Ok(Instruction::Binary { op, lhs: types[1], rhs: types[0] })
}

fn parse_asset_reference(reader: &mut Reader) -> Result<AssetReference> {
    let asset_type: &str = reader.parse_identifier()?;
    reader.consume_space()?;
    let id: i32 = reader.parse_int()?;

    let asset_reference = match asset_type {
        "object" => AssetReference::Object(GMRef::new(id)),
        "sprite" => AssetReference::Sprite(GMRef::new(id)),
        "sound" => AssetReference::Sound(GMRef::new(id)),
        "room" => AssetReference::Room(GMRef::new(id)),
        "tileset" => AssetReference::Tileset(GMRef::new(id)),
        "path" => AssetReference::Path(GMRef::new(id)),
        "script" => AssetReference::Script(GMRef::new(id)),
        "font" => AssetReference::Font(GMRef::new(id)),
        "timeline" => AssetReference::Timeline(GMRef::new(id)),
        "shader" => AssetReference::Shader(GMRef::new(id)),
        "sequence" => AssetReference::Sequence(GMRef::new(id)),
        "animcurve" => AssetReference::AnimationCurve(GMRef::new(id)),
        "particlesystem" => AssetReference::ParticleSystem(GMRef::new(id)),
        "instance" => AssetReference::Instance(InstanceID(id)),
        "function" => AssetReference::Function(GMRef::new(id)),
        _ => bail!("Invalid Type Cast to asset type {asset_type:?}"),
    };

    Ok(asset_reference)
}

fn parse_comparison(types: DataTypes, reader: &mut Reader) -> Result<Instruction> {
    types.assert_count(2, "cmp")?;
    let comparison_type: &str = reader.clear();
    let comparison_type = match comparison_type {
        "EQ" => ComparisonType::Equal,
        "NEQ" => ComparisonType::NotEqual,
        "LT" => ComparisonType::LessThan,
        "LTE" => ComparisonType::LessOrEqual,
        "GTE" => ComparisonType::GreaterOrEqual,
        "GT" => ComparisonType::GreaterThan,
        _ => bail!("Invalid Comparison Type {comparison_type:?}"),
    };
    Ok(Instruction::Compare { lhs: types[1], rhs: types[0], comparison_type })
}

fn parse_duplicate(types: DataTypes, reader: &mut Reader) -> Result<Instruction> {
    types.assert_count(1, "dup")?;
    let count: u8 = parse_int(reader.clear())?;
    Ok(Instruction::Duplicate { data_type: types[0], count })
}

fn parse_duplicate_swap(types: DataTypes, reader: &mut Reader) -> Result<Instruction> {
    types.assert_count(1, "dupswap")?;
    let size1: u8 = reader.parse_uint()?;
    reader.consume_space()?;
    let size2: u8 = parse_int(reader.clear())?;
    Ok(Instruction::DuplicateSwap { data_type: types[0], size1, size2 })
}

fn parse_push(types: DataTypes, reader: &mut Reader, gm_data: &GMData) -> Result<PushValue> {
    types.assert_count(1, "push")?;

    let value: PushValue = match types[0] {
        DataType::Short => PushValue::Short(parse_int(reader.clear())?),
        DataType::Int => {
            if let Some(type_cast) = reader.consume_round_brackets()? {
                match type_cast {
                    "function" => PushValue::Function(parse_function(
                        reader,
                        &gm_data.functions,
                        &gm_data.strings,
                    )?),
                    "variable" => {
                        let mut variable: CodeVariable = parse_variable(reader, gm_data)?;
                        variable.is_int32 = true;
                        PushValue::Variable(variable)
                    }
                    _ => bail!(
                        "Invalid type cast {type_cast:?}; expected \"function\" or \"variable\""
                    ),
                }
            } else {
                PushValue::Int(parse_int(reader.clear())?)
            }
        }
        DataType::Long => PushValue::Long(parse_int(reader.clear())?),
        DataType::Double => {
            let float: &str = reader.clear();
            let float: f64 =
                float.parse().ok().ok_or_else(|| format!("Invalid float literal {float:?}"))?;
            PushValue::Double(float)
        }
        DataType::Bool => {
            let bool: &str = reader.clear();
            let bool: bool = match bool {
                "true" => true,
                "false" => false,
                _ => bail!("Invalid boolean {bool:?}"),
            };
            PushValue::Bool(bool)
        }
        DataType::String => {
            if reader.peek_char() == Some('@') {
                // Example: push.s @420
                let id: i32 = parse_i32(reader.clear())?;
                PushValue::String(GMRef::new(id))
            } else if let Some(string_id) = extract_string_id(reader.line) {
                // Example: push.s "hello"@420
                reader.clear();
                let id: i32 = parse_i32(string_id)?;
                PushValue::String(GMRef::new(id))
            } else {
                // Example: push.s "hello"
                let string: String = parse_string_literal(reader)?;
                // GMData is not borrowed mutably (this is a conscious tradeoff).
                // Therefore, it can only get string that are already used in the data file somewhere.
                // If this is a problem, use `GMStrings::make` before assembling.
                let existing_string_ref = gm_data.strings.find(&string)?;
                PushValue::String(existing_string_ref)
            }
        }
        DataType::Variable => PushValue::Variable(parse_variable(reader, gm_data)?),
    };
    Ok(value)
}

fn parse_call(types: DataTypes, reader: &mut Reader, gm_data: &GMData) -> Result<Instruction> {
    types.assert_count(0, "call")?;
    let ident: &str = reader.parse_identifier()?;
    let function: GMRef<Function> = resolve_function(ident, &gm_data.functions, &gm_data.strings)?;
    reader.consume_space()?;
    let arg_count: u16 = reader.parse_uint()?;
    Ok(Instruction::Call { function, arg_count })
}

impl ReferenceType {
    fn from_string(variable_type: &str) -> Result<Self> {
        Ok(match variable_type {
            "stacktop" => Self::StackTop,
            "array" => Self::Array,
            "instance" => Self::Instance,
            "arraypushaf" => Self::MultiPush,
            "arraypopaf" => Self::MultiPop,
            _ => bail!("Invalid Variable Reference Type {variable_type:?}"),
        })
    }
}

fn parse_variable(reader: &mut Reader, gm_data: &GMData) -> Result<CodeVariable> {
    let mut ref_type = if let Some(ty) = reader.consume_square_brackets()? {
        ReferenceType::from_string(ty)?
    } else {
        ReferenceType::Normal
    };

    let instance_type_raw = reader.parse_identifier()?;
    let instance_type_arg = reader.consume_angle_brackets()?.unwrap_or_default();
    reader.consume_dot()?;

    let mut variable: GMRef<Variable> = GMRef::none();
    let instance_type: InstanceType = match instance_type_raw {
        "self" => InstanceType::Self_,
        "object" => {
            let object_ref: GMRef<GameObject> =
                gm_data.game_objects.ref_by_name(instance_type_arg, &gm_data.strings)?;
            InstanceType::GameObject(object_ref)
        }
        "instance" => {
            ref_type = ReferenceType::Instance;
            let instance_id: i32 = parse_int(instance_type_arg)?;
            InstanceType::Instance(InstanceID(instance_id))
        }
        "local" => {
            let var_index: i32 = parse_i32(instance_type_arg)?;
            variable = GMRef::new(var_index);
            InstanceType::Local
        }
        "stacktop" => InstanceType::StackTop,
        "builtin" => InstanceType::Builtin,
        "global" => InstanceType::Global,
        "arg" => InstanceType::Argument,
        "other" => InstanceType::Other,
        "static" => InstanceType::Static,
        "all" => InstanceType::All,
        "noone" => InstanceType::NoOne,
        _ => bail!("Invalid Instance Type {instance_type_raw:?}"),
    };

    let name: &str = parse_variable_identifier(reader)?;

    let var_type = instance_type.as_variable_type();
    if instance_type != InstanceType::Local {
        variable = gm_data.variables.find(name, var_type, &gm_data.strings);
    }

    if variable.is_none() {
        bail!("Cannot resolve variable with name {name:?}");
    }

    Ok(CodeVariable {
        variable,
        ref_type,
        instance_type,
        is_int32: false, // This has to be modified afterward, if necessary
    })
}

fn parse_variable_identifier<'a>(reader: &'a mut Reader) -> Result<&'a str> {
    if reader.consume_str("$$$$temp$$$$") { Ok("$$$$temp$$$$") } else { reader.parse_identifier() }
}

fn parse_function(
    reader: &mut Reader,
    gm_functions: &Functions,
    gm_strings: &Strings,
) -> Result<GMRef<Function>> {
    let ident = reader.clear();
    validate_identifier(ident)?;
    resolve_function(ident, gm_functions, gm_strings)
}

fn resolve_function(
    ident: &str,
    gm_functions: &Functions,
    gm_strings: &Strings,
) -> Result<GMRef<Function>> {
    let func = gm_functions.find(ident, gm_strings);
    if func.is_some() {
        return Ok(func);
    }
    bail!("Function {ident:?} does not exist (needs to be created using GMFunctions::make first)")
}

fn parse_int<T: FromStr + Copy>(string: &str) -> Result<T> {
    string.parse().map_err(|_| err!("Invalid {} Integer {:?}", typename::<T>(), string))
}

fn parse_i32(string: &str) -> Result<i32> {
    let int: i32 = parse_int(string)?;
    if int < 0 {
        bail!("Negative i32 Integer {int} is not allowed here");
    }
    Ok(int)
}

fn extract_string_id(line: &str) -> Option<&str> {
    for (idx, byte) in line.bytes().enumerate().rev() {
        if byte == b'@' {
            return Some(&line[idx + 1..]);
        }
        if byte == b'"' {
            return None;
        }
    }
    None
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
            let append_char = match char {
                '\\' => '\\',
                '"' => '"',
                'a' => '\x07', // 07 - Bell (Alert)
                'b' => '\x08', // 08 - Backspace
                't' => '\t',   // 09 - Horizontal Tab
                'n' => '\n',   // 0A - Line Feed
                'v' => '\x0B', // 0B - Vertical Tab
                'f' => '\x0C', // 0C - Form Feed
                'r' => '\r',   // 0D - Carriage Return
                _ => bail!("Invalid escape character {char:?}"),
            };
            string.push(append_char);
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
