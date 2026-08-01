// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::Code;
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
use crate::wad::elem::GMNamedElement;
use crate::wad::elem::function::Function;
use crate::wad::elem::game_object::GameObject;
use crate::wad::elem::string::Strings;
use crate::wad::elem::validate_identifier;
use crate::wad::elem::variable::Variable;

macro_rules! write {
    ($buffer:ident, $($args:tt)*) => {{
        use std::fmt::Write as _;
        let _ = $buffer.write_fmt(format_args!($($args)*));
        // Writing to a [`String`] can never fail.
    }};
}

fn slice_instructions_by_bytes(
    instructions: &[Instruction],
    start_offset: u32,
) -> Result<&[Instruction]> {
    let mut index = 0;
    let mut offset = 0;
    while offset < start_offset {
        #[rustfmt::skip] // currently, there is an internal bug in rustfmt
        let instr = instructions.get(index).ok_or_else(||
            format!("Given start byte offset {start_offset} is out of range in instructions with byte length {offset}"
        ))?;
        index += 1;
        offset += instr.size_bytes();
    }
    if offset != start_offset {
        bail!(
            "Given start byte offset {start_offset} is misaligned (reached offset {offset} \
             instead)"
        );
    }
    Ok(&instructions[index..])
}

/// Disassembles a code entry's instructions.
///
/// This function mostly exists for convenience since it is short to write
/// `disassemble_code(code, data)` instead of
/// `disassemble_instructions(&code.instructions, data)`.
///
/// However, it *does* have one behavioral difference:
/// It correctly disassembles child code entries by looking up
/// the instructions in the specified parent code and slicing them.
/// This is needed since the `instructions` field for child code entries is
/// always empty.
///
/// TODO: Maybe certain directives can be added in the future
/// to identify the code entry name as well as local and argument count.
pub fn disassemble_code(code: &Code, gm_data: &GMData) -> Result<String> {
    if let Some(modern) = &code.modern_data {
        if modern.parent.is_some() {
            let parent: &Code = gm_data
                .codes
                .by_ref(modern.parent)
                .ctx("resolving parent code entry")?;
            // can there be nested parents? cuz it only works for one layer rn
            let instrs = slice_instructions_by_bytes(&parent.instructions, modern.execution_offset)
                .ctx("preparing to disassemble child code entry")?;
            return disassemble_instructions(instrs, gm_data);
        } else if modern.execution_offset != 0 {
            bail!(
                "Root code entry has non-zero byte offset {}",
                modern.execution_offset
            );
        }
    }

    disassemble_instructions(&code.instructions, gm_data)
        .ctx(|| format!("disassembling code entry {:?}", code.name))
}

/// Disassembles multiple instructions and joins the instructions by newline.
///
/// If you need to handle child code entries correctly, consider using
/// [`disassemble_code`]. Otherwise, an empty string will be returned (since
/// child code entries store no instructions).
pub fn disassemble_instructions(instructions: &[Instruction], gm_data: &GMData) -> Result<String> {
    let mut assembly: String = String::new();

    for instruction in instructions {
        disassemble_instruction_to_buffer(&mut assembly, instruction, gm_data)?;
        assembly.push('\n');
    }

    Ok(assembly)
}

/// Disassembles a single instruction into one line.
pub fn disassemble_instruction(instruction: &Instruction, gm_data: &GMData) -> Result<String> {
    let mut buffer = String::new();
    disassemble_instruction_to_buffer(&mut buffer, instruction, gm_data)?;
    Ok(buffer)
}

/// Disassembles a single instruction into one line, appending the assembly
/// string to a buffer.
#[allow(clippy::too_many_lines)]
pub fn disassemble_instruction_to_buffer(
    buffer: &mut String,
    instruction: &Instruction,
    gm_data: &GMData,
) -> Result<()> {
    let mnemonic: &str = get_mnemonic(instruction);

    match *instruction {
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
            buffer.push_str(mnemonic);
        }

        Instruction::Negate { data_type }
        | Instruction::Not { data_type }
        | Instruction::PopDiscard { data_type } => {
            write!(buffer, "{}.{}", mnemonic, data_type.as_char());
        }

        Instruction::CallVariable { arg_count } => {
            write!(buffer, "{mnemonic} {arg_count}");
        }

        Instruction::Duplicate { data_type, count } => {
            write!(buffer, "{}.{} {}", mnemonic, data_type.as_char(), count);
        }

        Instruction::DuplicateSwap { data_type, size1, size2 } => {
            write!(
                buffer,
                "{}.{} {} {}",
                mnemonic,
                data_type.as_char(),
                size1,
                size2,
            );
        }

        Instruction::Branch { jump_offset }
        | Instruction::BranchIf { jump_offset }
        | Instruction::BranchUnless { jump_offset }
        | Instruction::PushWithContext { jump_offset }
        | Instruction::PopWithContext { jump_offset } => {
            write!(buffer, "{mnemonic} {jump_offset}");
        }

        Instruction::Convert { from, to } => {
            write!(buffer, "{}.{}.{}", mnemonic, from.as_char(), to.as_char());
        }

        Instruction::Binary { op: _, lhs, rhs } => {
            write!(buffer, "{}.{}.{}", mnemonic, rhs.as_char(), lhs.as_char());
        }

        Instruction::Compare { lhs, rhs, comparison_type } => {
            write!(
                buffer,
                "{}.{}.{} {}",
                mnemonic,
                rhs.as_char(),
                lhs.as_char(),
                comparison_type.as_str(),
            );
        }

        Instruction::Pop { variable, type1, type2 } => {
            write!(
                buffer,
                "{}.{}.{} ",
                mnemonic,
                type1.as_char(),
                type2.as_char()
            );
            write_variable(variable, buffer, gm_data)?;
        }

        Instruction::Push { value } => {
            write!(buffer, "{}.{} ", mnemonic, value.data_type().as_char());
            write_push_instruction(value, buffer, gm_data)?;
        }
        Instruction::PushLocal { variable }
        | Instruction::PushGlobal { variable }
        | Instruction::PushBuiltin { variable } => {
            write!(buffer, "{mnemonic} ");
            write_variable(variable, buffer, gm_data)?;
        }

        Instruction::PushImmediate { integer } => {
            write!(buffer, "{mnemonic} {integer}");
        }

        Instruction::Call { function, arg_count } => {
            write!(
                buffer,
                "{} {} {}",
                mnemonic,
                resolve_function_name(function, gm_data)?,
                arg_count,
            );
        }

        Instruction::PushReference { asset_reference } => {
            write!(buffer, "{mnemonic} ");
            write_asset_reference(asset_reference, buffer, gm_data)?;
        }
    }

    Ok(())
}

const fn get_mnemonic(instruction: &Instruction) -> &'static str {
    match *instruction {
        Instruction::Binary { op, .. } => get_mnemonic_binary(op),
        Instruction::Negate { .. } => "neg",
        Instruction::Not { .. } => "not",
        Instruction::Convert { .. } => "conv",
        Instruction::Compare { .. } => "cmp",
        Instruction::Pop { .. } => "pop",
        Instruction::PopSwap { is_array: false } => "popswap",
        Instruction::PopSwap { is_array: true } => "popswaparr",
        Instruction::Duplicate { .. } => "dup",
        Instruction::DuplicateSwap { .. } => "dupswap",
        Instruction::Return => "ret",
        Instruction::Exit => "exit",
        Instruction::PopDiscard { .. } => "popz",
        Instruction::Branch { .. } => "br",
        Instruction::BranchIf { .. } => "bt",
        Instruction::BranchUnless { .. } => "bf",
        Instruction::PushWithContext { .. } => "pushenv",
        Instruction::PopWithContext { .. } => "popenv",
        Instruction::PopWithContextExit => "popenvx",
        Instruction::Push { .. } => "push",
        Instruction::PushLocal { .. } => "pushloc",
        Instruction::PushGlobal { .. } => "pushglb",
        Instruction::PushBuiltin { .. } => "pushbltn",
        Instruction::PushImmediate { .. } => "pushim",
        Instruction::Call { .. } => "call",
        Instruction::CallVariable { .. } => "callvar",
        Instruction::CheckArrayIndex => "chkindex",
        Instruction::PushArrayFinal => "pushaf",
        Instruction::PopArrayFinal => "popaf",
        Instruction::PushArrayContainer => "pushac",
        Instruction::SetArrayOwner => "setowner",
        Instruction::HasStaticInitialized => "isstaticok",
        Instruction::SetStaticInitialized => "setstatic",
        Instruction::SaveArrayReference => "savearef",
        Instruction::RestoreArrayReference => "restorearef",
        Instruction::IsNullishValue => "isnullish",
        Instruction::PushReference { .. } => "pushref",
    }
}

const fn get_mnemonic_binary(binary_op: BinaryOp) -> &'static str {
    match binary_op {
        BinaryOp::Add => "add",
        BinaryOp::Subtract => "sub",
        BinaryOp::Multiply => "mul",
        BinaryOp::Divide => "div",
        BinaryOp::Remainder => "rem",
        BinaryOp::Modulus => "mod",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Xor => "xor",
        BinaryOp::ShiftLeft => "shl",
        BinaryOp::ShiftRight => "shr",
    }
}

impl DataType {
    #[must_use]
    const fn as_char(self) -> char {
        match self {
            Self::Short => 'e',
            Self::Int => 'i',
            Self::Long => 'l',
            Self::Double => 'd',
            Self::Bool => 'b',
            Self::String => 's',
            Self::Variable => 'v',
        }
    }
}

impl ComparisonType {
    #[must_use]
    const fn as_str(self) -> &'static str {
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

impl ReferenceType {
    #[must_use]
    const fn as_str(self) -> &'static str {
        match self {
            Self::Normal | Self::Instance => "",
            Self::Array => "[array]",
            Self::StackTop => "[stacktop]",
            Self::MultiPush => "[arraypushaf]",
            Self::MultiPop => "[arraypopaf]",
        }
    }
}

fn write_push_instruction(value: PushValue, buffer: &mut String, gm_data: &GMData) -> Result<()> {
    match value {
        PushValue::Short(integer) => {
            write!(buffer, "{integer}");
        }
        PushValue::Int(integer) => {
            write!(buffer, "{integer}");
        }
        PushValue::Long(integer) => {
            write!(buffer, "{integer}");
        }
        PushValue::Double(float) => {
            write!(buffer, "{float}");
        }
        PushValue::Bool(true) => {
            write!(buffer, "true");
        }
        PushValue::Bool(false) => {
            write!(buffer, "false");
        }
        PushValue::String(string) => write_gm_string(string, &gm_data.strings, buffer)?,
        PushValue::Variable(code_variable) => {
            write_variable(code_variable, buffer, gm_data)?;
        }
        PushValue::Function(function_ref) => {
            write!(
                buffer,
                "(function){}",
                resolve_function_name(function_ref, gm_data)?
            );
        }
    }
    Ok(())
}

fn asset_get_name<'a, T, C>(
    chunk: &C,
    gm_ref: GMRef<T>,
    gm_strings: &'a Strings,
) -> Result<&'a String>
where
    T: GMNamedElement + 'a,
    C: GMAssetListChunk<Element = T>,
{
    const CTX: &str = "resolving asset reference for PushReference Instruction";

    let element: &T = chunk.by_ref(gm_ref).ctx(CTX)?;

    element
        .validate_name(gm_strings)
        .ctx(|| format!("validating name of {}", typename::<T>()))
        .ctx(CTX)?;

    let name: &'a String = element.name(gm_strings)?;
    Ok(name)
}

fn write_asset_reference(
    asset_ref: AssetReference,
    buffer: &mut String,
    gm_data: &GMData,
) -> Result<()> {
    let kind: &str = match asset_ref {
        AssetReference::Object(_) => "object",
        AssetReference::Sprite(_) => "sprite",
        AssetReference::Sound(_) => "sound",
        AssetReference::Room(_) => "room",
        AssetReference::Path(_) => "path",
        AssetReference::Script(_) => "script",
        AssetReference::Font(_) => "font",
        AssetReference::Timeline(_) => "timeline",
        AssetReference::Shader(_) => "shader",
        AssetReference::Sequence(_) => "sequence",
        AssetReference::AnimationCurve(_) => "animationcurve",
        AssetReference::ParticleSystem(_) => "particlesystem",
        AssetReference::Tileset(_) => "tileset",
        AssetReference::Instance(_) => "instance",
        AssetReference::Function(_) => "function",
    };

    let strg = &gm_data.strings;
    let name: &str = match asset_ref {
        AssetReference::Object(gm_ref) => asset_get_name(&gm_data.game_objects, gm_ref, strg)?,
        AssetReference::Sprite(gm_ref) => asset_get_name(&gm_data.sprites, gm_ref, strg)?,
        AssetReference::Sound(gm_ref) => asset_get_name(&gm_data.sounds, gm_ref, strg)?,
        AssetReference::Room(gm_ref) => asset_get_name(&gm_data.rooms, gm_ref, strg)?,
        AssetReference::Tileset(gm_ref) => asset_get_name(&gm_data.tilesets, gm_ref, strg)?,
        AssetReference::Path(gm_ref) => asset_get_name(&gm_data.paths, gm_ref, strg)?,
        AssetReference::Script(gm_ref) => asset_get_name(&gm_data.scripts, gm_ref, strg)?,
        AssetReference::Font(gm_ref) => asset_get_name(&gm_data.fonts, gm_ref, strg)?,
        AssetReference::Timeline(gm_ref) => asset_get_name(&gm_data.timelines, gm_ref, strg)?,
        AssetReference::Shader(gm_ref) => asset_get_name(&gm_data.shaders, gm_ref, strg)?,
        AssetReference::Sequence(gm_ref) => asset_get_name(&gm_data.sequences, gm_ref, strg)?,
        AssetReference::AnimationCurve(gm_ref) => {
            asset_get_name(&gm_data.animation_curves, gm_ref, strg)?
        }
        AssetReference::ParticleSystem(gm_ref) => {
            asset_get_name(&gm_data.particle_systems, gm_ref, strg)?
        }
        AssetReference::Instance(_) => "", // stub
        AssetReference::Function(gm_ref) => resolve_function_name(gm_ref, gm_data)?,
    };

    let id = asset_ref.asset_id();

    if let AssetReference::Instance(_) = asset_ref {
        write!(buffer, "{kind} {id}");
    } else if validate_identifier(name).is_ok() {
        write!(buffer, "{kind} {id}  # {name}");
    } else {
        write!(buffer, "{kind} {id}  # {name:?}");
    }
    Ok(())
}

fn write_instance_type(
    instance_type: InstanceType,
    buffer: &mut String,
    variable_ref: GMRef<Variable>,
    gm_data: &GMData,
) -> Result<()> {
    match instance_type {
        InstanceType::Self_ => write!(buffer, "self"),
        InstanceType::GameObject(obj_ref) => {
            let obj: &GameObject = gm_data.game_objects.by_ref(obj_ref)?;
            obj.validate_name(&gm_data.strings)
                .ctx("validating game object name")?;
            write!(buffer, "object<{}>", obj.name(&gm_data.strings)?);
        }
        InstanceType::Instance(instance_id) => {
            write!(buffer, "instance<{}>", instance_id.0);
        }
        InstanceType::Local => write!(buffer, "local<{}>", variable_ref.index),
        InstanceType::Other => write!(buffer, "other"),
        InstanceType::All => write!(buffer, "all"),
        InstanceType::NoOne => write!(buffer, "noone"),
        InstanceType::Global => write!(buffer, "global"),
        InstanceType::Builtin => write!(buffer, "builtin"),
        InstanceType::StackTop => write!(buffer, "stacktop"),
        InstanceType::Argument => write!(buffer, "arg"),
        InstanceType::Static => write!(buffer, "static"),
    }

    Ok(())
}

fn write_variable(
    code_variable: CodeVariable,
    buffer: &mut String,
    gm_data: &GMData,
) -> Result<()> {
    let variable: &Variable = gm_data.variables.by_ref(code_variable.variable)?;
    variable
        .validate_name(&gm_data.strings)
        .ctx("validating variable identifier")?;
    let name: &String = variable.name(&gm_data.strings)?;

    if code_variable.is_int32 {
        write!(buffer, "(variable)");
    }
    write!(buffer, "{}", code_variable.ref_type.as_str());
    write_instance_type(
        code_variable.instance_type,
        buffer,
        code_variable.variable,
        gm_data,
    )?;
    write!(buffer, ".{name}");

    Ok(())
}

fn resolve_function_name(function_ref: GMRef<Function>, gm_data: &GMData) -> Result<&String> {
    let function: &Function = gm_data.functions.by_ref(function_ref)?;
    let name: &String = gm_data.strings.by_ref(function.name)?;
    validate_identifier(name).ctx("validating function identifier")?;
    Ok(name)
}

fn write_gm_string(
    string_ref: GMRef<String>,
    gm_strings: &Strings,
    buffer: &mut String,
) -> Result<()> {
    let string: &String = gm_strings.by_ref(string_ref)?;
    write_literal_string(string, buffer);
    write!(buffer, "@{}", string_ref.index);
    Ok(())
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
            '\x07' => buffer.push_str("\\a"), // 07 - Bell (Alert)
            '\x08' => buffer.push_str("\\b"), // 08 - Backspace
            '\t' => buffer.push_str("\\t"),   // 09 - Horizontal Tab
            '\n' => buffer.push_str("\\n"),   // 0A - Line Feed
            '\x0B' => buffer.push_str("\\v"), // 0B - Vertical Tab
            '\x0C' => buffer.push_str("\\f"), // 0C - Form Feed
            '\r' => buffer.push_str("\\r"),   // 0D - Carriage Return
            _ => buffer.push(c),
        }
    }

    buffer.push('"');
}
