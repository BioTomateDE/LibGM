use crate::{
    gamemaker::{
        data::GMData,
        elements::{
            GMListChunk, GMNamedElement, function::GMFunction, game_object::GMGameObject,
            variable::GMVariable,
        },
        reference::GMRef,
    },
    gml::instructions::{
        CodeVariable, GMAssetReference, GMCode, GMComparisonType, GMDataType, GMInstanceType,
        GMInstruction, GMVariableType, PushValue,
    },
    prelude::*,
    util::fmt::typename,
};

macro_rules! write {
    ($buffer:ident, $($args:tt)*) => {{
        use std::fmt::Write as _;
        let _ = $buffer.write_fmt(format_args!($($args)*));
        // Writing to a [`String`] can never fail.
    }};
}

pub fn disassemble_code(code: &GMCode, gm_data: &GMData) -> Result<String> {
    disassemble_instructions(&code.instructions, gm_data)
}

pub fn disassemble_instructions(
    instructions: &[GMInstruction],
    gm_data: &GMData,
) -> Result<String> {
    let mut assembly: String = String::new();

    for instruction in instructions {
        disassemble_instr(instruction, &mut assembly, gm_data)?;
        assembly.push('\n');
    }

    Ok(assembly)
}

pub fn disassemble_instruction(instruction: &GMInstruction, gm_data: &GMData) -> Result<String> {
    let mut buffer = String::new();
    disassemble_instr(instruction, &mut buffer, gm_data)?;
    Ok(buffer)
}

fn disassemble_instr(
    instruction: &GMInstruction,
    buffer: &mut String,
    gm_data: &GMData,
) -> Result<()> {
    let mnemonic: &str = instruction.mnemonic();

    match instruction {
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
            write!(buffer, "{mnemonic}");
        },

        GMInstruction::Negate { data_type }
        | GMInstruction::Not { data_type }
        | GMInstruction::PopDiscard { data_type } => {
            write!(buffer, "{}.{}", mnemonic, data_type.to_str());
        },

        GMInstruction::CallVariable { argument_count } => {
            write!(buffer, "{mnemonic} {argument_count}");
        },

        GMInstruction::Duplicate { data_type, size } => {
            write!(buffer, "{}.{} {}", mnemonic, data_type.to_str(), size,);
        },

        GMInstruction::DuplicateSwap { data_type, size1, size2 } => {
            write!(
                buffer,
                "{}.{} {} {}",
                mnemonic,
                data_type.to_str(),
                size1,
                size2,
            );
        },

        GMInstruction::Branch { jump_offset }
        | GMInstruction::BranchIf { jump_offset }
        | GMInstruction::BranchUnless { jump_offset }
        | GMInstruction::PushWithContext { jump_offset }
        | GMInstruction::PopWithContext { jump_offset } => {
            write!(buffer, "{mnemonic} {jump_offset}");
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
            write!(buffer, "{}.{}.{}", mnemonic, type1.to_str(), type2.to_str());
        },

        GMInstruction::Compare { lhs, rhs, comparison_type } => {
            write!(
                buffer,
                "{}.{}.{} {}",
                mnemonic,
                rhs.to_str(),
                lhs.to_str(),
                comparison_type.to_str(),
            );
        },

        GMInstruction::Pop { variable, type1, type2 } => {
            // TODO: find the instance type of the variable? idek
            write!(
                buffer,
                "{}.{}.{} ",
                mnemonic,
                type1.to_str(),
                type2.to_str()
            );
            write_variable(variable, buffer, gm_data)?;
        },

        GMInstruction::Push { value } => {
            write!(buffer, "{}.{} ", mnemonic, value.data_type().to_str());
            write_push_instruction(value, buffer, gm_data)?;
        },
        GMInstruction::PushLocal { variable }
        | GMInstruction::PushGlobal { variable }
        | GMInstruction::PushBuiltin { variable } => {
            write!(buffer, "{mnemonic} ");
            write_variable(variable, buffer, gm_data)?;
        },

        GMInstruction::PushImmediate { integer } => {
            write!(buffer, "{mnemonic} {integer}");
        },

        &GMInstruction::Call { function, argument_count } => {
            write!(
                buffer,
                "{} {}(argc={})",
                mnemonic,
                resolve_function_name(function, gm_data)?,
                argument_count,
            );
        },

        GMInstruction::PushReference { asset_reference } => {
            write!(buffer, "{mnemonic} ");
            write_asset_reference(asset_reference, buffer, gm_data)?;
        },
    }

    Ok(())
}

impl GMInstruction {
    #[must_use]
    const fn mnemonic(&self) -> &'static str {
        match self {
            Self::Convert { .. } => "conv",
            Self::Multiply { .. } => "mul",
            Self::Divide { .. } => "div",
            Self::Remainder { .. } => "rem",
            Self::Modulus { .. } => "mod",
            Self::Add { .. } => "add",
            Self::Subtract { .. } => "sub",
            Self::And { .. } => "and",
            Self::Or { .. } => "or",
            Self::Xor { .. } => "xor",
            Self::Negate { .. } => "neg",
            Self::Not { .. } => "not",
            Self::ShiftLeft { .. } => "shl",
            Self::ShiftRight { .. } => "shr",
            Self::Compare { .. } => "cmp",
            Self::Pop { .. } => "pop",
            Self::PopSwap { is_array: false } => "popswap",
            Self::PopSwap { is_array: true } => "popswaparr",
            Self::Duplicate { .. } => "dup",
            Self::DuplicateSwap { .. } => "dupswap",
            Self::Return => "ret",
            Self::Exit => "exit",
            Self::PopDiscard { .. } => "popz",
            Self::Branch { .. } => "jmp",
            Self::BranchIf { .. } => "jt",
            Self::BranchUnless { .. } => "jf",
            Self::PushWithContext { .. } => "pushenv",
            Self::PopWithContext { .. } => "popenv",
            Self::PopWithContextExit => "popenvexit",
            Self::Push { .. } => "push",
            Self::PushLocal { .. } => "pushloc",
            Self::PushGlobal { .. } => "pushglb",
            Self::PushBuiltin { .. } => "pushbltn",
            Self::PushImmediate { .. } => "pushim",
            Self::Call { .. } => "call",
            Self::CallVariable { .. } => "callvar",
            Self::CheckArrayIndex => "chkindex",
            Self::PushArrayFinal => "pushaf",
            Self::PopArrayFinal => "popaf",
            Self::PushArrayContainer => "pushac",
            Self::SetArrayOwner => "setowner",
            Self::HasStaticInitialized => "isstaticok",
            Self::SetStaticInitialized => "setstatic",
            Self::SaveArrayReference => "savearef",
            Self::RestoreArrayReference => "restorearef",
            Self::IsNullishValue => "isnullish",
            Self::PushReference { .. } => "pushref",
        }
    }
}

impl GMDataType {
    #[must_use]
    const fn to_str(self) -> &'static str {
        match self {
            Self::Int16 => "e",
            Self::Int32 => "i",
            Self::Int64 => "l",
            Self::Double => "d",
            Self::Boolean => "b",
            Self::String => "s",
            Self::Variable => "v",
        }
    }
}

impl GMComparisonType {
    #[must_use]
    const fn to_str(self) -> &'static str {
        match self {
            Self::LessThan => "LT",
            Self::LessOrEqual => "LTE",
            Self::Equal => "EQ",
            Self::NotEqual => "NEQ",
            Self::GreaterOrEqual => "GTE",
            Self::GreaterThan => "GT",
        }
    }
}

impl GMVariableType {
    #[must_use]
    const fn to_str(self) -> &'static str {
        match self {
            Self::Normal | Self::Instance => "",
            Self::Array => "[array]",
            Self::StackTop => "[stacktop]",
            Self::ArrayPushAF => "[arraypushaf]",
            Self::ArrayPopAF => "[arraypopaf]",
        }
    }
}

fn write_push_instruction(value: &PushValue, buffer: &mut String, gm_data: &GMData) -> Result<()> {
    match value {
        PushValue::Variable(code_variable) => {
            write_variable(code_variable, buffer, gm_data)?;
        },
        PushValue::Boolean(true) => {
            write!(buffer, "true");
        },
        PushValue::Boolean(false) => {
            write!(buffer, "false");
        },
        PushValue::Function(function_ref) => {
            write!(
                buffer,
                "(function){}",
                resolve_function_name(*function_ref, gm_data)?
            );
        },
        PushValue::String(string) => write_literal_string(string, buffer),
        PushValue::Int16(integer) => {
            write!(buffer, "{integer}");
        },
        PushValue::Int32(integer) => {
            write!(buffer, "{integer}");
        },
        PushValue::Int64(integer) => {
            write!(buffer, "{integer}");
        },
        PushValue::Double(float) => {
            write!(buffer, "{float}");
        },
    }
    Ok(())
}

#[inline]
fn asset_get_name<'a, T: GMNamedElement + 'a, C: GMListChunk<Element = T> + 'a>(
    chunk: &'a C,
    gm_ref: GMRef<T>,
) -> Result<&'a String> {
    const CTX: &str = "resolving asset reference for PushReference Instruction";

    let element: &'a T = chunk.by_ref(gm_ref).context(CTX)?;

    element
        .validate_name()
        .with_context(|| format!("validating name of {}", typename::<T>()))
        .context(CTX)?;

    let name: &'a String = element.name();

    Ok(name)
}

fn write_asset_reference(
    asset_ref: &GMAssetReference,
    buffer: &mut String,
    gm_data: &GMData,
) -> Result<()> {
    match *asset_ref {
        GMAssetReference::Object(gm_ref) => {
            write!(
                buffer,
                "(object){}",
                asset_get_name(&gm_data.game_objects, gm_ref)?
            );
        },
        GMAssetReference::Sprite(gm_ref) => {
            write!(
                buffer,
                "(sprite){}",
                asset_get_name(&gm_data.sprites, gm_ref)?
            );
        },
        GMAssetReference::Sound(gm_ref) => {
            write!(
                buffer,
                "(sound){}",
                asset_get_name(&gm_data.sounds, gm_ref)?
            );
        },
        GMAssetReference::Room(gm_ref) => {
            write!(buffer, "(room){}", asset_get_name(&gm_data.rooms, gm_ref)?);
        },
        GMAssetReference::Background(gm_ref) => {
            write!(
                buffer,
                "(background){}",
                asset_get_name(&gm_data.backgrounds, gm_ref)?
            );
        },
        GMAssetReference::Path(gm_ref) => {
            write!(buffer, "(path){}", asset_get_name(&gm_data.paths, gm_ref)?);
        },
        GMAssetReference::Script(gm_ref) => {
            write!(
                buffer,
                "(script){}",
                asset_get_name(&gm_data.scripts, gm_ref)?
            );
        },
        GMAssetReference::Font(gm_ref) => {
            write!(buffer, "(font){}", asset_get_name(&gm_data.fonts, gm_ref)?);
        },
        GMAssetReference::Timeline(gm_ref) => {
            write!(
                buffer,
                "(timeline){}",
                asset_get_name(&gm_data.timelines, gm_ref)?
            );
        },
        GMAssetReference::Shader(gm_ref) => {
            write!(
                buffer,
                "(shader){}",
                asset_get_name(&gm_data.shaders, gm_ref)?
            );
        },
        GMAssetReference::Sequence(gm_ref) => {
            write!(
                buffer,
                "(sequence){}",
                asset_get_name(&gm_data.sequences, gm_ref)?
            );
        },
        GMAssetReference::AnimCurve(gm_ref) => {
            write!(
                buffer,
                "(animcurve){}",
                asset_get_name(&gm_data.animation_curves, gm_ref)?
            );
        },
        GMAssetReference::ParticleSystem(gm_ref) => {
            write!(
                buffer,
                "(particlesystem){}",
                asset_get_name(&gm_data.particle_systems, gm_ref)?
            );
        },
        GMAssetReference::RoomInstance(id) => {
            write!(buffer, "(roominstance){id}");
        },
        GMAssetReference::Function(gm_ref) => {
            write!(
                buffer,
                "(function){}",
                resolve_function_name(gm_ref, gm_data)?
            );
        },
    }

    Ok(())
}

fn write_instance_type(
    instance_type: GMInstanceType,
    buffer: &mut String,
    variable_ref: GMRef<GMVariable>,
    gm_data: &GMData,
) -> Result<()> {
    match instance_type {
        GMInstanceType::Undefined => {
            bail!("Did not expect Instance Type Undefined here; please report this error");
        },
        GMInstanceType::Self_(Some(obj_ref)) => {
            let obj: &GMGameObject = gm_data.game_objects.by_ref(obj_ref)?;
            write!(buffer, "self<{}>", obj.name);
        },
        GMInstanceType::Self_(None) => write!(buffer, "self"),
        GMInstanceType::RoomInstance(instance_id) => {
            write!(buffer, "roominstance<{instance_id}>");
        },
        GMInstanceType::Other => write!(buffer, "other"),
        GMInstanceType::All => write!(buffer, "all"),
        GMInstanceType::None => write!(buffer, "none"),
        GMInstanceType::Global => write!(buffer, "global"),
        GMInstanceType::Builtin => write!(buffer, "builtin"),
        GMInstanceType::Local => write!(buffer, "local<{}>", variable_ref.index),
        GMInstanceType::StackTop => write!(buffer, "stacktop"),
        GMInstanceType::Argument => write!(buffer, "arg"),
        GMInstanceType::Static => write!(buffer, "static"),
    }

    Ok(())
}

fn write_variable(
    code_variable: &CodeVariable,
    buffer: &mut String,
    gm_data: &GMData,
) -> Result<()> {
    let variable: &GMVariable = gm_data.variables.by_ref(code_variable.variable)?;
    variable
        .validate_name()
        .context("validating variable identifier")?;
    let name = &variable.name;

    if code_variable.is_int32 {
        write!(buffer, "(variable)");
    }

    write!(buffer, "{}", code_variable.variable_type.to_str());

    let instance_type: GMInstanceType = if code_variable.instance_type == GMInstanceType::Undefined
    {
        // TODO: this will not work with b14
        variable
            .b15_data
            .as_ref()
            .map_or(GMInstanceType::Undefined, |b15| b15.instance_type)
    } else {
        code_variable.instance_type
    };

    write_instance_type(instance_type, buffer, code_variable.variable, gm_data)?;
    write!(buffer, ".{name}");

    Ok(())
}

fn resolve_function_name(function_ref: GMRef<GMFunction>, gm_data: &GMData) -> Result<&String> {
    let function: &GMFunction = gm_data.functions.by_ref(function_ref)?;
    function
        .validate_name()
        .context("validating function identifier")?;
    let name = &function.name;
    Ok(name)
}

fn write_literal_string(string_lit: &str, buffer: &mut String) {
    // Fast path: check if any escaping is needed
    if !string_lit
        .bytes()
        .any(|b| matches!(b, b'\\' | b'"' | b'\n' | b'\r' | b'\t'))
    {
        write!(buffer, "\"{string_lit}\"");
        return;
    }

    // Slow path: escaping needed
    // Estimate capacity: original length + 2 quotes + some overhead for escapes
    buffer.reserve(string_lit.len() + string_lit.len() / 4 + 2);
    buffer.push('"');

    for c in string_lit.chars() {
        match c {
            '\\' => buffer.push_str("\\\\"),
            '"' => buffer.push_str("\\\""),
            '\n' => buffer.push_str("\\n"),
            '\r' => buffer.push_str("\\r"),
            '\t' => buffer.push_str("\\t"),
            _ => buffer.push(c),
        }
    }

    buffer.push('"');
}
