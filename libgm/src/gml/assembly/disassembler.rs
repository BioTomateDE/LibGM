use crate::{
    gamemaker::{
        data::GMData,
        elements::{
            GMListChunk, GMNamedElement, function::GMFunction, game_object::GMGameObject,
            variable::GMVariable,
        },
        reference::GMRef,
    },
    gml::instruction::{
        AssetReference, CodeVariable, ComparisonType, DataType, GMCode, InstanceType, Instruction,
        PushValue, VariableType,
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

pub fn disassemble_instructions(instructions: &[Instruction], gm_data: &GMData) -> Result<String> {
    let mut assembly: String = String::new();

    for instruction in instructions {
        disassemble_instr(instruction, &mut assembly, gm_data)?;
        assembly.push('\n');
    }

    Ok(assembly)
}

pub fn disassemble_instruction(instruction: &Instruction, gm_data: &GMData) -> Result<String> {
    let mut buffer = String::new();
    disassemble_instr(instruction, &mut buffer, gm_data)?;
    Ok(buffer)
}

fn disassemble_instr(
    instruction: &Instruction,
    buffer: &mut String,
    gm_data: &GMData,
) -> Result<()> {
    let mnemonic: &str = instruction.mnemonic();

    match instruction {
        Instruction::Exit
        | Instruction::Return
        | Instruction::PopSwap { .. }
        | Instruction::PopWithContextExit
        | Instruction::CheckArrayIndex
        | Instruction::PushArrayFinal
        | Instruction::PopArrayFinal
        | Instruction::PushArrayContainer
        | Instruction::SetArrayOwner
        | Instruction::HasStaticInitialized
        | Instruction::SetStaticInitialized
        | Instruction::SaveArrayReference
        | Instruction::RestoreArrayReference
        | Instruction::IsNullishValue => {
            write!(buffer, "{mnemonic}");
        },

        Instruction::Negate { data_type }
        | Instruction::Not { data_type }
        | Instruction::PopDiscard { data_type } => {
            write!(buffer, "{}.{}", mnemonic, data_type.to_str());
        },

        Instruction::CallVariable { argument_count } => {
            write!(buffer, "{mnemonic} {argument_count}");
        },

        Instruction::Duplicate { data_type, size } => {
            write!(buffer, "{}.{} {}", mnemonic, data_type.to_str(), size);
        },

        Instruction::DuplicateSwap { data_type, size1, size2 } => {
            write!(
                buffer,
                "{}.{} {} {}",
                mnemonic,
                data_type.to_str(),
                size1,
                size2,
            );
        },

        Instruction::Branch { jump_offset }
        | Instruction::BranchIf { jump_offset }
        | Instruction::BranchUnless { jump_offset }
        | Instruction::PushWithContext { jump_offset }
        | Instruction::PopWithContext { jump_offset } => {
            write!(buffer, "{mnemonic} {jump_offset}");
        },

        Instruction::Convert { from: type1, to: type2 }
        | Instruction::Multiply { multiplicand: type2, multiplier: type1 }
        | Instruction::Divide { dividend: type2, divisor: type1 }
        | Instruction::Remainder { dividend: type2, divisor: type1 }
        | Instruction::Modulus { dividend: type2, divisor: type1 }
        | Instruction::Add { augend: type2, addend: type1 }
        | Instruction::Subtract { minuend: type2, subtrahend: type1 }
        | Instruction::And { lhs: type2, rhs: type1 }
        | Instruction::Or { lhs: type2, rhs: type1 }
        | Instruction::Xor { lhs: type2, rhs: type1 }
        | Instruction::ShiftLeft { value: type2, shift_amount: type1 }
        | Instruction::ShiftRight { value: type2, shift_amount: type1 } => {
            write!(buffer, "{}.{}.{}", mnemonic, type1.to_str(), type2.to_str());
        },

        Instruction::Compare { lhs, rhs, comparison_type } => {
            write!(
                buffer,
                "{}.{}.{} {}",
                mnemonic,
                rhs.to_str(),
                lhs.to_str(),
                comparison_type.to_str(),
            );
        },

        Instruction::Pop { variable, type1, type2 } => {
            // TODO: find the instance type of the variable? idek
            write!(
                buffer,
                "{}.{}.{} ",
                mnemonic,
                type1.to_str(),
                type2.to_str(),
            );
            write_variable(variable, buffer, gm_data)?;
        },

        Instruction::Push { value } => {
            write!(buffer, "{}.{} ", mnemonic, value.data_type().to_str());
            write_push_instruction(value, buffer, gm_data)?;
        },
        Instruction::PushLocal { variable }
        | Instruction::PushGlobal { variable }
        | Instruction::PushBuiltin { variable } => {
            write!(buffer, "{mnemonic} ");
            write_variable(variable, buffer, gm_data)?;
        },

        Instruction::PushImmediate { integer } => {
            write!(buffer, "{mnemonic} {integer}");
        },

        &Instruction::Call { function, argument_count } => {
            write!(
                buffer,
                "{} {}(argc={})",
                mnemonic,
                resolve_function_name(function, gm_data)?,
                argument_count,
            );
        },

        Instruction::PushReference { asset_reference } => {
            write!(buffer, "{mnemonic} ");
            write_asset_reference(asset_reference, buffer, gm_data)?;
        },
    }

    Ok(())
}

impl Instruction {
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

impl DataType {
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

impl ComparisonType {
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

impl VariableType {
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
    asset_ref: &AssetReference,
    buffer: &mut String,
    gm_data: &GMData,
) -> Result<()> {
    match *asset_ref {
        AssetReference::Object(gm_ref) => {
            write!(
                buffer,
                "(object){}",
                asset_get_name(&gm_data.game_objects, gm_ref)?
            );
        },
        AssetReference::Sprite(gm_ref) => {
            write!(
                buffer,
                "(sprite){}",
                asset_get_name(&gm_data.sprites, gm_ref)?
            );
        },
        AssetReference::Sound(gm_ref) => {
            write!(
                buffer,
                "(sound){}",
                asset_get_name(&gm_data.sounds, gm_ref)?
            );
        },
        AssetReference::Room(gm_ref) => {
            write!(buffer, "(room){}", asset_get_name(&gm_data.rooms, gm_ref)?);
        },
        AssetReference::Background(gm_ref) => {
            write!(
                buffer,
                "(background){}",
                asset_get_name(&gm_data.backgrounds, gm_ref)?
            );
        },
        AssetReference::Path(gm_ref) => {
            write!(buffer, "(path){}", asset_get_name(&gm_data.paths, gm_ref)?);
        },
        AssetReference::Script(gm_ref) => {
            write!(
                buffer,
                "(script){}",
                asset_get_name(&gm_data.scripts, gm_ref)?
            );
        },
        AssetReference::Font(gm_ref) => {
            write!(buffer, "(font){}", asset_get_name(&gm_data.fonts, gm_ref)?);
        },
        AssetReference::Timeline(gm_ref) => {
            write!(
                buffer,
                "(timeline){}",
                asset_get_name(&gm_data.timelines, gm_ref)?
            );
        },
        AssetReference::Shader(gm_ref) => {
            write!(
                buffer,
                "(shader){}",
                asset_get_name(&gm_data.shaders, gm_ref)?
            );
        },
        AssetReference::Sequence(gm_ref) => {
            write!(
                buffer,
                "(sequence){}",
                asset_get_name(&gm_data.sequences, gm_ref)?
            );
        },
        AssetReference::AnimCurve(gm_ref) => {
            write!(
                buffer,
                "(animcurve){}",
                asset_get_name(&gm_data.animation_curves, gm_ref)?
            );
        },
        AssetReference::ParticleSystem(gm_ref) => {
            write!(
                buffer,
                "(particlesystem){}",
                asset_get_name(&gm_data.particle_systems, gm_ref)?
            );
        },
        AssetReference::RoomInstance(id) => {
            write!(buffer, "(roominstance){id}");
        },
        AssetReference::Function(gm_ref) => {
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
    instance_type: InstanceType,
    buffer: &mut String,
    variable_ref: GMRef<GMVariable>,
    gm_data: &GMData,
) -> Result<()> {
    match instance_type {
        InstanceType::Self_ => write!(buffer, "self"),
        InstanceType::GameObject(obj_ref) => {
            let obj: &GMGameObject = gm_data.game_objects.by_ref(obj_ref)?;
            obj.validate_name().context("validating game object name")?;
            write!(buffer, "object<{}>", obj.name);
        },
        InstanceType::RoomInstance(instance_id) => {
            write!(buffer, "roominstance<{instance_id}>");
        },
        InstanceType::Local => write!(buffer, "local<{}>", variable_ref.index),
        InstanceType::Other => write!(buffer, "other"),
        InstanceType::All => write!(buffer, "all"),
        InstanceType::None => write!(buffer, "none"),
        InstanceType::Global => write!(buffer, "global"),
        InstanceType::Builtin => write!(buffer, "builtin"),
        InstanceType::StackTop => write!(buffer, "stacktop"),
        InstanceType::Argument => write!(buffer, "arg"),
        InstanceType::Static => write!(buffer, "static"),
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
    write_instance_type(
        code_variable.instance_type,
        buffer,
        code_variable.variable,
        gm_data,
    )?;
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
