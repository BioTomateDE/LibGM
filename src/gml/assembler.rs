use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::GMDataType;
use crate::gamemaker::elements::code::GMInstruction;
use crate::gamemaker::elements::code::{
    CodeVariable, GMAssetReference, GMCallInstruction, GMCodeValue, GMComparisonInstruction, GMEmptyInstruction,
    GMGotoInstruction, GMInstanceType, GMPopInstruction, GMPopSwapInstruction, GMPopenvExitMagicInstruction,
    GMPushInstruction, GMSingleTypeInstruction, GMVariableType,
};
use crate::gamemaker::elements::code::{
    GMCallVariableInstruction, GMComparisonType, GMDoubleTypeInstruction, GMDuplicateInstruction,
    GMDuplicateSwapInstruction,
};
use crate::gamemaker::elements::functions::{GMFunction, GMFunctions};
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::variables::{GMVariable, to_vari_instance_type};
use crate::prelude::*;
use crate::util::fmt::typename;
use arrayvec::ArrayVec;
use std::ops::Neg;
use std::str::{Chars, FromStr};

pub fn assemble_code(assembly: &str, gm_data: &mut GMData) -> Result<Vec<GMInstruction>> {
    let mut instructions: Vec<GMInstruction> = Vec::new();

    for line in assembly.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let instruction: GMInstruction =
            assemble_instruction(line, gm_data).with_context(|| format!("assembling instruction: {line}"))?;
        instructions.push(instruction);
    }

    Ok(instructions)
}

pub fn assemble_instruction(line: &str, gm_data: &mut GMData) -> Result<GMInstruction> {
    let line: &mut &str = &mut line.trim();
    let mnemonic: String;

    let opcode_end: Option<usize> = line.find(|c: char| !c.is_ascii_alphanumeric());
    if let Some(index) = opcode_end {
        let (head, tail) = line.split_at(index);
        mnemonic = head.to_string();
        *line = tail;
    } else {
        // Opcode takes up entire line
        mnemonic = line.to_string();
        *line = "";
    }

    let mut types: ArrayVec<GMDataType, 2> = ArrayVec::new();
    while line.chars().next() == Some('.') {
        consume_dot(line)?;
        let raw_type: char = consume_char(line).context("Unexpected EOL when trying to parse instruction data type")?;
        let data_type: GMDataType = data_type_from_char(raw_type)?;
        types
            .try_push(data_type)
            .context("Opcodes can only have up to two types")?;
    }

    match line.chars().next() {
        Some(' ') => {
            consume_char(line);
        }
        None => {}
        _ => bail!("Expected space; found remaining string {line:?}"),
    }

    let instruction: GMInstruction = match mnemonic.as_str() {
        "conv" => GMInstruction::Convert(parse_double_type(&types)?),
        "mul" => GMInstruction::Multiply(parse_double_type(&types)?),
        "div" => GMInstruction::Divide(parse_double_type(&types)?),
        "rem" => GMInstruction::Remainder(parse_double_type(&types)?),
        "mod" => GMInstruction::Modulus(parse_double_type(&types)?),
        "add" => GMInstruction::Add(parse_double_type(&types)?),
        "sub" => GMInstruction::Subtract(parse_double_type(&types)?),
        "and" => GMInstruction::And(parse_double_type(&types)?),
        "or" => GMInstruction::Or(parse_double_type(&types)?),
        "xor" => GMInstruction::Xor(parse_double_type(&types)?),
        "neg" => GMInstruction::Negate(parse_single_type(&types)?),
        "not" => GMInstruction::Not(parse_single_type(&types)?),
        "shl" => GMInstruction::ShiftLeft(parse_double_type(&types)?),
        "shr" => GMInstruction::ShiftRight(parse_double_type(&types)?),
        "cmp" => GMInstruction::Compare(parse_comparison(&types, line)?),
        "pop" => GMInstruction::Pop(parse_pop(&types, line, gm_data)?),
        "popswap" => GMInstruction::PopSwap(parse_pop_swap(&types, line)?),
        "dup" => GMInstruction::Duplicate(parse_duplicate(&types, line)?),
        "dupswap" => GMInstruction::DuplicateSwap(parse_duplicate_swap(&types, line)?),
        "ret" => GMInstruction::Return(parse_single_type(&types)?),
        "exit" => GMInstruction::Exit(GMEmptyInstruction),
        "popz" => GMInstruction::PopDiscard(parse_single_type(&types)?),
        "jmp" => GMInstruction::Branch(parse_goto(&types, line)?),
        "jt" => GMInstruction::BranchIf(parse_goto(&types, line)?),
        "jf" => GMInstruction::BranchUnless(parse_goto(&types, line)?),
        "pushenv" => GMInstruction::PushWithContext(parse_goto(&types, line)?),
        "popenv" => GMInstruction::PopWithContext(parse_goto(&types, line)?),
        "popenvexit" => GMInstruction::PopWithContextExit(GMPopenvExitMagicInstruction),
        "push" => GMInstruction::Push(parse_push(&types, line, gm_data)?),
        "pushloc" => GMInstruction::PushLocal(parse_push(&types, line, gm_data)?),
        "pushglb" => GMInstruction::PushGlobal(parse_push(&types, line, gm_data)?),
        "pushbltn" => GMInstruction::PushBuiltin(parse_push(&types, line, gm_data)?),
        "pushim" => GMInstruction::PushImmediate(parse_push_immediate(&types, line)?),
        "call" => GMInstruction::Call(parse_call(&types, line, gm_data)?),
        "callvar" => GMInstruction::CallVariable(parse_call_var(&types, line)?),
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
        "pushref" => GMInstruction::PushReference(parse_asset_reference(gm_data, line)?),
        _ => bail!("Invalid opcode mnemonic {mnemonic:?}"),
    };

    if !line.is_empty() {
        bail!("Expected end of line; found remaining string {line:?}")
    }

    Ok(instruction)
}

macro_rules! asset_by_name {
    ($gm_data: expr, $typename: ident, $namefn: expr) => {{
        let target_name: String = $namefn;
        let mut found = None;
        for (i, element) in $gm_data.$typename.$typename.iter().enumerate() {
            let name = element
                .name
                .resolve(&$gm_data.strings.strings)
                .with_context(|| format!("Cannot resolve {} asset's name", stringify!($typename)))?;
            if *name == target_name {
                found = Some(GMRef::new(i as u32));
                break;
            }
        }
        found.with_context(|| {
            format!(
                "Could not resolve Asset of type {} with name {:?}",
                stringify!($typename),
                target_name
            )
        })?
    }};
}

fn parse_asset_reference(gm_data: &GMData, line: &mut &str) -> Result<GMAssetReference> {
    let asset_type: String = consume_round_brackets(line)?
        .with_context(|| format!("Expected type cast with round brackets; found {line:?}"))?;

    Ok(match asset_type.as_str() {
        "object" => GMAssetReference::Object(asset_by_name!(gm_data, game_objects, parse_identifier(line)?)),
        "sprite" => GMAssetReference::Sprite(asset_by_name!(gm_data, sprites, parse_identifier(line)?)),
        "sound" => GMAssetReference::Sound(asset_by_name!(gm_data, sounds, parse_identifier(line)?)),
        "room" => GMAssetReference::Room(asset_by_name!(gm_data, rooms, parse_identifier(line)?)),
        "background" => GMAssetReference::Background(asset_by_name!(gm_data, backgrounds, parse_identifier(line)?)),
        "path" => GMAssetReference::Path(asset_by_name!(gm_data, paths, parse_identifier(line)?)),
        "script" => GMAssetReference::Script(asset_by_name!(gm_data, scripts, parse_identifier(line)?)),
        "font" => GMAssetReference::Font(asset_by_name!(gm_data, fonts, parse_identifier(line)?)),
        "timeline" => GMAssetReference::Timeline(asset_by_name!(gm_data, timelines, parse_identifier(line)?)),
        "shader" => GMAssetReference::Shader(asset_by_name!(gm_data, shaders, parse_identifier(line)?)),
        "sequence" => GMAssetReference::Sequence(asset_by_name!(gm_data, sequences, parse_identifier(line)?)),
        "animcurve" => GMAssetReference::AnimCurve(asset_by_name!(gm_data, animation_curves, parse_identifier(line)?)),
        "particlesystem" => {
            GMAssetReference::ParticleSystem(asset_by_name!(gm_data, particle_systems, parse_identifier(line)?))
        }
        "roominstance" => GMAssetReference::RoomInstance(parse_int(line)?),
        "function" => GMAssetReference::Function(parse_function(line, &gm_data.strings, &gm_data.functions)?),
        _ => bail!("Invalid Type Cast to asset type {asset_type:?}"),
    })
}

fn parse_double_type(types: &ArrayVec<GMDataType, 2>) -> Result<GMDoubleTypeInstruction> {
    assert_type_count(&types, 2)?;
    Ok(GMDoubleTypeInstruction { right: types[0], left: types[1] })
}

fn parse_single_type(types: &ArrayVec<GMDataType, 2>) -> Result<GMSingleTypeInstruction> {
    assert_type_count(&types, 1)?;
    Ok(GMSingleTypeInstruction { data_type: types[0] })
}

fn parse_comparison(types: &ArrayVec<GMDataType, 2>, line: &mut &str) -> Result<GMComparisonInstruction> {
    assert_type_count(&types, 2)?;
    let comparison_type_raw: String = parse_identifier(line)?;
    let comparison_type: GMComparisonType = comparison_type_from_string(&comparison_type_raw)?;
    Ok(GMComparisonInstruction { comparison_type, type1: types[0], type2: types[1] })
}

fn parse_pop(types: &ArrayVec<GMDataType, 2>, line: &mut &str, gm_data: &GMData) -> Result<GMPopInstruction> {
    assert_type_count(&types, 2)?;
    let destination: CodeVariable = parse_variable(line, &gm_data)?;
    Ok(GMPopInstruction { type1: types[0], type2: types[1], destination })
}

fn parse_pop_swap(types: &ArrayVec<GMDataType, 2>, line: &mut &str) -> Result<GMPopSwapInstruction> {
    assert_type_count(&types, 0)?;
    let size: u8 = parse_uint(line)?;
    Ok(GMPopSwapInstruction { size })
}

fn parse_duplicate(types: &ArrayVec<GMDataType, 2>, line: &mut &str) -> Result<GMDuplicateInstruction> {
    assert_type_count(&types, 1)?;
    let size: u8 = parse_uint(line)?;
    Ok(GMDuplicateInstruction { data_type: types[0], size })
}

fn parse_duplicate_swap(types: &ArrayVec<GMDataType, 2>, line: &mut &str) -> Result<GMDuplicateSwapInstruction> {
    assert_type_count(&types, 1)?;
    let size1: u8 = parse_uint(line)?;
    consume_space(line)?;
    let size2: u8 = parse_uint(line)?;
    Ok(GMDuplicateSwapInstruction { data_type: types[0], size1, size2 })
}

fn parse_goto(types: &ArrayVec<GMDataType, 2>, line: &mut &str) -> Result<GMGotoInstruction> {
    assert_type_count(&types, 0)?;
    let jump_offset: i32 = parse_int(line)?;
    Ok(GMGotoInstruction { jump_offset })
}

fn parse_push(types: &ArrayVec<GMDataType, 2>, line: &mut &str, gm_data: &mut GMData) -> Result<GMPushInstruction> {
    assert_type_count(&types, 1)?;
    let value: GMCodeValue = match types[0] {
        GMDataType::Int16 => GMCodeValue::Int16(parse_int(line)?),
        GMDataType::Int32 => {
            if let Some(type_cast) = consume_round_brackets(line)? {
                match type_cast.as_str() {
                    "function" => GMCodeValue::Function(parse_function(line, &gm_data.strings, &gm_data.functions)?),
                    "variable" => {
                        let mut variable: CodeVariable = parse_variable(line, &gm_data)?;
                        variable.is_int32 = true;
                        GMCodeValue::Variable(variable)
                    }
                    _ => bail!("Invalid type cast {type_cast:?}; expected \"function\" or \"variable\""),
                }
            } else {
                GMCodeValue::Int32(parse_int(line)?)
            }
        }
        GMDataType::Int64 => GMCodeValue::Int64(parse_int(line)?),
        GMDataType::Double => GMCodeValue::Double(parse_float(line)?),
        GMDataType::Boolean => GMCodeValue::Boolean(parse_bool(line)?),
        GMDataType::String => {
            let string_text: String = parse_string_literal(line)?;
            let string_ref: GMRef<String> = gm_data.make_string(&string_text);
            GMCodeValue::String(string_ref)
        }
        GMDataType::Variable => GMCodeValue::Variable(parse_variable(line, &gm_data)?),
    };
    Ok(GMPushInstruction { value })
}

fn parse_push_immediate(types: &ArrayVec<GMDataType, 2>, line: &mut &str) -> Result<i16> {
    assert_type_count(&types, 0)?;
    parse_int(line)
}

fn parse_call(types: &ArrayVec<GMDataType, 2>, line: &mut &str, gm_data: &GMData) -> Result<GMCallInstruction> {
    assert_type_count(&types, 0)?;
    let function: GMRef<GMFunction> = parse_function(line, &gm_data.strings, &gm_data.functions)?;
    let argc_str: String = consume_round_brackets(line)?
        .with_context(|| format!("Expected round brackets with argument count for function call; found {line:?}"))?;
    let argument_count: u16 = if let Some(str) = argc_str.strip_prefix("argc=") {
        str.parse().with_context(|| format!("Invalid argument count {str}"))?
    } else {
        bail!("Expected \"argc=\" for function call parameters; found {line:?}");
    };
    Ok(GMCallInstruction { argument_count, function })
}

fn parse_call_var(types: &ArrayVec<GMDataType, 2>, line: &mut &str) -> Result<GMCallVariableInstruction> {
    assert_type_count(&types, 1)?;
    let argument_count: u16 = parse_uint(line)?;
    Ok(GMCallVariableInstruction { data_type: types[0], argument_count })
}

fn assert_type_count(types: &ArrayVec<GMDataType, 2>, n: usize) -> Result<()> {
    if types.len() != n {
        bail!("Expected {} types for this instruction; got {}", n, types.len());
    }
    Ok(())
}

fn consume_char(line: &mut &str) -> Option<char> {
    let mut chars: Chars = line.chars();
    let first_char: Option<char> = chars.next();
    *line = chars.as_str();
    first_char
}

#[must_use]
fn consume_str(line: &mut &str, string: &'static str) -> Option<()> {
    if line.starts_with(string) {
        *line = &line[string.bytes().len()..];
        Some(())
    } else {
        None
    }
}

fn consume_space(line: &mut &str) -> Result<()> {
    let char: char = consume_char(line).context("Expected space, got EOL")?;
    if char != ' ' {
        bail!("Expected space, got '{char}'");
    }
    Ok(())
}

fn consume_dot(line: &mut &str) -> Result<()> {
    let char: char = consume_char(line).context("Expected dot, got EOL")?;
    if char != '.' {
        bail!("Expected dot, got '{char}'");
    }
    Ok(())
}

fn consume_brackets(line: &mut &str, open: char, close: char) -> Result<Option<String>> {
    if !line.starts_with(open) {
        return Ok(None);
    }
    consume_char(line);
    let close_pos = line.find(close).ok_or_else(|| format!("'{open}' was never closed"))?;
    let inside = line[..close_pos].to_string();
    *line = &line[close_pos + 1..];
    Ok(Some(inside))
}

fn consume_round_brackets(line: &mut &str) -> Result<Option<String>> {
    consume_brackets(line, '(', ')')
}

fn consume_square_brackets(line: &mut &str) -> Result<Option<String>> {
    consume_brackets(line, '[', ']')
}

fn consume_angle_brackets(line: &mut &str) -> Result<Option<String>> {
    consume_brackets(line, '<', '>')
}

fn data_type_from_char(data_type: char) -> Result<GMDataType> {
    Ok(match data_type {
        'v' => GMDataType::Variable,
        'i' => GMDataType::Int32,
        's' => GMDataType::String,
        'e' => GMDataType::Int16,
        'd' => GMDataType::Double,
        'l' => GMDataType::Int64,
        'b' => GMDataType::Boolean,
        _ => bail!("Invalid data type '{data_type}'"),
    })
}

fn parse_int<T: FromStr + Neg<Output = T>>(line: &mut &str) -> Result<T> {
    let is_negative: bool = line.starts_with('-');
    if is_negative {
        consume_char(line); // Consume minus sign
    }
    let integer: T = parse_uint(line)?;
    if is_negative { Ok(-integer) } else { Ok(integer) }
}

fn parse_uint<T: FromStr>(line: &mut &str) -> Result<T> {
    let end: usize = line.find(|c: char| !c.is_ascii_digit()).unwrap_or_else(|| line.len());
    if end == 0 {
        bail!("Expected integer, got {line:?}");
    }
    let integer: T = line[..end].parse().ok().with_context(|| {
        format!(
            "Integer {} is out of bounds for integer type {}",
            &line[..end],
            typename::<T>()
        )
    })?;
    *line = &line[end..];
    Ok(integer)
}

/// This only works if the line is ONLY the float
fn parse_float<T: FromStr>(line: &mut &str) -> Result<T> {
    let float: T = line
        .parse()
        .ok()
        .with_context(|| format!("Invalid float literal {line:?}"))?;
    *line = ""; // Consume entire line
    Ok(float)
}

fn parse_bool(line: &mut &str) -> Result<bool> {
    let bool: String = parse_identifier(line)?;
    match bool.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("Invalid boolean {bool:?}"),
    }
}

fn comparison_type_from_string(comparison_type: &str) -> Result<GMComparisonType> {
    Ok(match comparison_type {
        "EQ" => GMComparisonType::Equal,
        "NEQ" => GMComparisonType::NotEqual,
        "LT" => GMComparisonType::LessThan,
        "LTE" => GMComparisonType::LessOrEqual,
        "GTE" => GMComparisonType::GreaterOrEqual,
        "GT" => GMComparisonType::GreaterThan,
        _ => bail!("Invalid Comparison Type {comparison_type:?}"),
    })
}

fn variable_type_from_string(variable_type: &str) -> Result<GMVariableType> {
    Ok(match variable_type {
        "stacktop" => GMVariableType::StackTop,
        "array" => GMVariableType::Array,
        "instance" => GMVariableType::Instance,
        "arraypushaf" => GMVariableType::ArrayPushAF,
        "arraypopaf" => GMVariableType::ArrayPopAF,
        _ => bail!("Invalid Variable Reference Type {variable_type:?}"),
    })
}

fn parse_variable(line: &mut &str, gm_data: &GMData) -> Result<CodeVariable> {
    let mut variable_type: GMVariableType = GMVariableType::Normal;
    if let Some(variable_type_str) = consume_square_brackets(line)? {
        variable_type = variable_type_from_string(&variable_type_str)?;
    }

    let instance_type_raw: String = parse_identifier(line)?;
    let instance_type_arg: String = consume_angle_brackets(line)?.unwrap_or_default();
    consume_dot(line)?;

    let mut variable_ref: Option<GMRef<GMVariable>> = None;
    let instance_type: GMInstanceType = match instance_type_raw.as_str() {
        "self" if instance_type_arg.is_empty() => GMInstanceType::Self_(None),
        "self" => {
            let object_ref: GMRef<GMGameObject> = gm_data
                .game_objects
                .get_object_ref_by_name(&instance_type_arg, &gm_data.strings)?;
            GMInstanceType::Self_(Some(object_ref))
        }
        "local" => {
            let var_index: u32 = instance_type_arg
                .parse()
                .with_context(|| format!("Invalid index for local variable: {instance_type_arg:?}"))?;
            variable_ref = Some(GMRef::new(var_index));
            GMInstanceType::Local
        }
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

    let name: String = parse_identifier(line).or_else(|err| {
        consume_str(line, "$$$$temp$$$$").ok_or(err)?;
        Ok::<_, Error>("$$$$temp$$$$".to_string())
    })?;

    if instance_type != GMInstanceType::Local {
        // Convert instance type because of some bullshit
        let vari_instance_type: GMInstanceType = to_vari_instance_type(&instance_type);

        for (i, var) in gm_data.variables.variables.iter().enumerate() {
            let var_name: &String = var
                .name
                .resolve(&gm_data.strings.strings)
                .context("Cannot resolve variable name")?;
            if *var_name != name {
                continue;
            }
            if let Some(b15) = &var.b15_data {
                if b15.instance_type != vari_instance_type {
                    continue;
                }
            }
            // Found var
            variable_ref = Some(GMRef::new(i as u32));
            break;
        }
    }

    let Some(variable) = variable_ref else {
        bail!("Cannot resolve variable with name {name:?}");
    };

    // // I need to throw away the instance type so that the tests pass
    // if variable_type != GMVariableType::Normal {
    //     instance_type = GMInstanceType::Undefined;
    // }   // TODO: comment out this block if not testing assembler

    Ok(CodeVariable {
        variable,
        variable_type,
        instance_type,
        is_int32: false, // This has to be modified afterward, if necessary
    })
}

fn parse_function(line: &mut &str, gm_strings: &GMStrings, gm_functions: &GMFunctions) -> Result<GMRef<GMFunction>> {
    let identifier: String = parse_identifier(line).or_else(|_| {
        // Allow special functions like `@@This@@`
        consume_str(line, "@@").context("Invalid function identifier")?;
        let identifier: String = parse_identifier(line)?;
        consume_str(line, "@@").context("Invalid function identifier")?;
        Ok::<_, Error>(format!("@@{identifier}@@"))
    })?;

    for (i, func) in gm_functions.functions.iter().enumerate() {
        let func_name: &String = func
            .name
            .resolve(&gm_strings.strings)
            .context("Cannot resolve name string of parsed function")?;
        if *func_name == identifier {
            return Ok(GMRef::new(i as u32));
        }
    }

    bail!("Function {identifier:?} does not exist")
}

fn parse_identifier(line: &mut &str) -> Result<String> {
    // Identifiers can't start with a digit
    if line.starts_with(|c: char| c.is_ascii_digit()) {
        bail!("Expected identifier; found {line:?}");
    }

    for (i, char) in line.char_indices() {
        // Checks ordered in descending average occurrence count
        match char {
            'a'..='z' => continue,
            '0'..='9' => continue,
            'A'..='Z' => continue,
            '_' => continue,
            _ => (),
        }
        let identifier: String = line[..i].to_string();
        if identifier.is_empty() {
            bail!("Expected identifier; found {line:?}");
        }
        *line = &line[i..];
        return Ok(identifier);
    }

    // Identifier goes to end of line
    let identifier: String = line.to_string();
    *line = ""; // Consume line
    Ok(identifier)
}

/// Assumes the entire rest of the line is the string literal
fn parse_string_literal(line: &mut &str) -> Result<String> {
    if consume_char(line) != Some('"') {
        bail!("Expected string literal; found {line:?}");
    }

    let mut escaping: bool = false;
    let mut string: String = String::with_capacity(line.bytes().len());

    for (i, char) in line.char_indices() {
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
            *line = &line[i + 1..];
            return Ok(string);
        } else if char == '\\' {
            escaping = true;
        } else {
            string.push(char);
        }
    }

    bail!("Unexpected EOL while reading string literal")
}
