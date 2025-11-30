use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement, functions::GMFunction, variables::GMVariable},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::{
        instructions::{
            GMAssetReference, GMCode, GMCodeBytecode15, GMCodeValue, GMComparisonType, GMDataType,
            GMInstruction, GMVariableType,
        },
        opcodes,
    },
    prelude::*,
    util::{
        assert::{assert_data_type, assert_int},
        init::{num_enum_from, vec_with_capacity},
    },
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMCodes {
    pub codes: Vec<GMCode>,
    pub exists: bool,
}

impl GMCodes {
    fn index_by_name(&self, name: &str) -> Result<usize> {
        for (i, code) in self.codes.iter().enumerate() {
            if code.name == name {
                return Ok(i);
            }
        }

        bail!("Could not find code entry with name {name:?}");
    }

    pub fn ref_by_name(&self, name: &str) -> Result<GMRef<GMCode>> {
        self.index_by_name(name).map(GMRef::from)
    }

    pub fn by_name(&self, name: &str) -> Result<&GMCode> {
        self.index_by_name(name).map(|index| &self.codes[index])
    }

    pub fn by_name_mut(&mut self, name: &str) -> Result<&mut GMCode> {
        self.index_by_name(name).map(|index| &mut self.codes[index])
    }
}

impl Deref for GMCodes {
    type Target = Vec<GMCode>;
    fn deref(&self) -> &Self::Target {
        &self.codes
    }
}

impl DerefMut for GMCodes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.codes
    }
}

impl GMChunkElement for GMCodes {
    const NAME: &'static str = "CODE";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMCodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.get_chunk_length() == 0 {
            return Ok(Self { codes: vec![], exists: false });
        }

        let pointers: Vec<u32> = reader.read_simple_list()?;
        reader.cur_pos = match pointers.first() {
            Some(&ptr) => ptr,
            None => {
                return Ok(Self { codes: vec![], exists: true });
            },
        };
        let count: usize = pointers.len();
        let mut codes: Vec<GMCode> = Vec::with_capacity(count);
        let mut instructions_ranges: Vec<(u32, u32)> = Vec::with_capacity(count);
        let mut codes_by_pos: HashMap<u32, GMRef<GMCode>> = HashMap::new();
        let mut last_pos = reader.cur_pos;

        for pointer in pointers {
            reader.assert_pos(pointer, "Code")?;
            let name: String = reader.read_gm_string()?;
            let code_length = reader.read_u32()?;

            let instructions_start_pos;
            let instructions_end_pos;
            let bytecode15_info: Option<GMCodeBytecode15>;

            if reader.general_info.bytecode_version <= 14 {
                instructions_start_pos = reader.cur_pos; // Instructions are placed immediately after code metadata; how convenient!
                reader.cur_pos += code_length; // Skip over them; they will get parsed in the next loops
                instructions_end_pos = reader.cur_pos;
                bytecode15_info = None;
            } else {
                let locals_count = reader.read_u16()?;
                let arguments_count_raw = reader.read_u16()?;
                let arguments_count: u16 = arguments_count_raw & 0x7FFF;
                let weird_local_flag: bool = arguments_count_raw & 0x8000 != 0;

                let instructions_start_offset = reader.read_i32()?;
                instructions_start_pos =
                    (instructions_start_offset + reader.cur_pos as i32 - 4) as u32;

                let offset = reader.read_u32()?;
                let b15_info = GMCodeBytecode15 {
                    locals_count,
                    arguments_count,
                    weird_local_flag,
                    offset,
                    parent: None,
                };
                instructions_end_pos = instructions_start_pos + code_length;
                bytecode15_info = Some(b15_info);
            }

            codes.push(GMCode {
                name,
                instructions: vec![],
                bytecode15_info,
            });
            instructions_ranges.push((instructions_start_pos, instructions_end_pos));
            last_pos = reader.cur_pos;
        }

        for (i, (start, end)) in instructions_ranges.into_iter().enumerate() {
            let code: &mut GMCode = &mut codes[i];
            let length = end - start;

            // If bytecode15+ and the instructions pointer is known, then it's a child code entry
            if length > 0
                && let Some(parent_code) = codes_by_pos.get(&start)
                && let Some(b15_info) = &mut code.bytecode15_info
            {
                b15_info.parent = Some(*parent_code);
                continue;
            }

            reader.cur_pos = start;
            // Estimated Size: https://discord.com/channels/566861759210586112/568625491876118528/1424403240258371615
            code.instructions = vec_with_capacity(length / 5)?;

            if length > 0 {
                // Update information to mark this entry as the root (if we have at least 1 instruction)
                codes_by_pos.insert(start, GMRef::new(i as u32));
            }

            while reader.cur_pos < end {
                let instruction = GMInstruction::deserialize(reader)
                    .with_context(|| {
                        format!(
                            "parsing Instruction #{} at position {}",
                            code.instructions.len(),
                            reader.cur_pos,
                        )
                    })
                    .with_context(|| {
                        format!("parsing Code entry {:?} at position {}", code.name, start)
                    })?;
                code.instructions.push(instruction);
            }
        }

        reader.cur_pos = last_pos; // has to be chunk end (since instructions are stored separately in b15+)
        Ok(Self { codes, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.codes.len())?;
        let pointer_list_pos: usize = builder.len();
        for _ in 0..self.codes.len() {
            builder.write_u32(0xDEAD_C0DE);
        }

        // Bytecode 14 my beloved
        if builder.bytecode_version() <= 14 {
            for (i, code) in self.codes.iter().enumerate() {
                builder.overwrite_usize(builder.len(), pointer_list_pos + 4 * i)?;
                builder.write_gm_string(&code.name);
                let length_placeholder_pos: usize = builder.len();
                builder.write_u32(0xDEAD_C0DE);
                let start: usize = builder.len();

                // In bytecode 14, instructions are written immediately
                for (i, instruction) in code.instructions.iter().enumerate() {
                    instruction.serialize(builder).with_context(|| {
                        format!("serializing code #{i} with name {:?}", code.name,)
                    })?;
                }

                let code_length: usize = builder.len() - start;
                builder.overwrite_usize(code_length, length_placeholder_pos)?;
            }
            return Ok(());
        }

        // In bytecode 15, the codes' instructions are written before the codes metadata
        let mut instructions_ranges: Vec<(usize, usize)> = Vec::with_capacity(self.codes.len());

        for (i, code) in self.codes.iter().enumerate() {
            if code.bytecode15_info.as_ref().unwrap().parent.is_some() {
                // If this is a child code entry, don't write instructions; just repeat last pointer
                instructions_ranges.push(*instructions_ranges.last().unwrap());
                // ^ TODO: this unwrap will fail if the first entry is a child entry (which is invalid but still)
                continue;
            }

            let start: usize = builder.len();
            for instruction in &code.instructions {
                instruction
                    .serialize(builder)
                    .with_context(|| format!("serializing code #{i} with name {:?}", code.name))?;
            }
            let end: usize = builder.len();
            instructions_ranges.push((start, end));
        }

        for (i, code) in self.codes.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_pos + 4 * i)?;
            let (start, end) = instructions_ranges[i];
            let length: usize = end - start;
            let b15_info: &GMCodeBytecode15 = code.bytecode15_info.as_ref().ok_or_else(|| {
                format!(
                    "Code bytecode 15 data not set in Bytecode version {}",
                    builder.bytecode_version()
                )
            })?;

            builder.write_gm_string(&code.name);
            builder.write_usize(length)?;
            builder.write_u16(b15_info.locals_count);
            builder.write_u16(
                b15_info.arguments_count | if b15_info.weird_local_flag { 0x8000 } else { 0 },
            );
            let instructions_start_offset: i32 = start as i32 - builder.len() as i32;
            builder.write_i32(instructions_start_offset);
            builder.write_u32(b15_info.offset);
        }

        Ok(())
    }
}

impl GMElement for GMInstruction {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let word = reader.read_u32()?;
        let mut opcode = ((word & 0xFF00_0000) >> 24) as u8;
        let b2 = ((word & 0x00FF_0000) >> 16) as u8;
        let b1 = ((word & 0x0000_FF00) >> 8) as u8;
        let b0 = ((word & 0x0000_00FF) >> 0) as u8;
        let mut b = [b0, b1, b2];

        if reader.general_info.bytecode_version < 15 {
            if matches!(opcode, 0x10..=0x16) {
                // This is needed to preserve the comparison type for pre bytecode 15
                assert_zero_b1(b)?;
                b[1] = opcode - 0x10;
            }
            opcode = opcodes::old_to_new(opcode);
        }

        // log::debug!("{} // {:02X} {:02X} {:02X} {:02X}", reader.cur_pos-4, b[0], b[1], b[2], opcode);

        Ok(match opcode {
            opcodes::CONV => {
                let types = parse_double_type(b).context("parsing Convert Instruction")?;
                Self::Convert { from: types[0], to: types[1] }
            },
            opcodes::MUL => {
                let types = parse_double_type(b).context("parsing Multiply Instruction")?;
                Self::Multiply {
                    multiplicand: types[1],
                    multiplier: types[0],
                }
            },
            opcodes::DIV => {
                let types = parse_double_type(b).context("parsing Divide Instruction")?;
                Self::Divide { dividend: types[1], divisor: types[0] }
            },
            opcodes::REM => {
                let types = parse_double_type(b).context("parsing Remainder Instruction")?;
                Self::Remainder { dividend: types[1], divisor: types[0] }
            },
            opcodes::MOD => {
                let types = parse_double_type(b).context("parsing Modulus Instruction")?;
                Self::Modulus { dividend: types[1], divisor: types[0] }
            },
            opcodes::ADD => {
                let types = parse_double_type(b).context("parsing Add Instruction")?;
                Self::Add { augend: types[1], addend: types[0] }
            },
            opcodes::SUB => {
                let types = parse_double_type(b).context("parsing Subtract Instruction")?;
                Self::Subtract { minuend: types[1], subtrahend: types[0] }
            },
            opcodes::AND => {
                let types = parse_double_type(b).context("parsing And Instruction")?;
                Self::And { lhs: types[1], rhs: types[0] }
            },
            opcodes::OR => {
                let types = parse_double_type(b).context("parsing Or Instruction")?;
                Self::Or { lhs: types[1], rhs: types[0] }
            },
            opcodes::XOR => {
                let types = parse_double_type(b).context("parsing Xor Instruction")?;
                Self::Xor { lhs: types[1], rhs: types[0] }
            },
            opcodes::NEG => {
                let data_type = parse_single_type(b).context("parsing Negate Instruction")?;
                Self::Negate { data_type }
            },
            opcodes::NOT => {
                let data_type = parse_single_type(b).context("parsing Not Instruction")?;
                Self::Not { data_type }
            },
            opcodes::SHL => {
                let types = parse_double_type(b).context("parsing ShiftLeft instruction")?;
                Self::ShiftLeft { value: types[1], shift_amount: types[0] }
            },
            opcodes::SHR => {
                let types = parse_double_type(b).context("parsing ShiftRight Instruction")?;
                Self::ShiftRight { value: types[1], shift_amount: types[0] }
            },
            opcodes::CMP => parse_comparison(b).context("parsing Comparison Instruction")?,
            opcodes::POP => parse_pop(b, reader).context("parsing Pop Instruction")?,
            opcodes::DUP => parse_duplicate(b).context("parsing Duplicate Instruction")?,
            opcodes::RET => {
                let ctx = "parsing Return Instruction";
                let data_type = parse_single_type(b).context(ctx)?;
                assert_type(GMDataType::Variable, data_type).context(ctx)?;
                Self::Return
            },
            opcodes::EXIT => {
                let ctx = "parsing Exit Instruction";
                let data_type = parse_single_type(b).context(ctx)?;
                assert_type(GMDataType::Int32, data_type).context(ctx)?;
                Self::Exit
            },
            opcodes::POPZ => {
                let data_type = parse_single_type(b).context("parsing PopDiscard Instruction")?;
                Self::PopDiscard { data_type }
            },
            opcodes::JMP => Self::Branch { jump_offset: parse_branch(b, reader) },
            opcodes::JT => Self::BranchIf { jump_offset: parse_branch(b, reader) },
            opcodes::JF => Self::BranchUnless { jump_offset: parse_branch(b, reader) },
            opcodes::PUSHENV => Self::PushWithContext { jump_offset: parse_branch(b, reader) },
            opcodes::POPENV if b == [0x00, 0x00, 0xF0] => Self::PopWithContextExit,
            opcodes::POPENV => Self::PopWithContext { jump_offset: parse_branch(b, reader) },
            opcodes::PUSH => {
                let value = parse_push(b, reader).context("parsing Push Instruction")?;
                Self::Push { value }
            },
            opcodes::PUSHLOC => {
                let (variable_ref, variable_type) =
                    parse_push_var(b, reader).context("parsing PushLocal Instruction")?;
                Self::PushLocal { variable_ref, variable_type }
            },
            opcodes::PUSHGLB => {
                let (variable_ref, variable_type) =
                    parse_push_var(b, reader).context("parsing PushGlobal Instruction")?;
                Self::PushGlobal { variable_ref, variable_type }
            },
            opcodes::PUSHBLTN => {
                let (variable_ref, variable_type) =
                    parse_push_var(b, reader).context("parsing PushBuiltin Instruction")?;
                Self::PushBuiltin { variable_ref, variable_type }
            },
            opcodes::PUSHIM => {
                let integer = parse_pushim(b).context("parsing PushImmediate Instruction")?;
                Self::PushImmediate { integer }
            },
            opcodes::CALL => parse_call(b, reader).context("parsing Call Instruction")?,
            opcodes::CALLVAR => {
                let argument_count =
                    parse_callvar(b).context("parsing CallVariable Instruction")?;
                Self::CallVariable { argument_count }
            },
            opcodes::EXTENDED => {
                parse_extended(reader, b).context("parsing Extended Instruction")?
            },
            _ => bail!("Invalid Instruction Opcode {opcode} (0x{opcode:02X})"),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut opcode: u8 = self.opcode();
        if builder.bytecode_version() < 15 {
            opcode = opcodes::new_to_old(opcode);
        }

        match self {
            &Self::Convert { from, to } => {
                build_double_type(builder, opcode, from, to);
            },
            &Self::Multiply { multiplicand, multiplier } => {
                build_double_type(builder, opcode, multiplier, multiplicand);
            },
            &Self::Divide { dividend, divisor } => {
                build_double_type(builder, opcode, divisor, dividend);
            },
            &Self::Remainder { dividend, divisor } => {
                build_double_type(builder, opcode, divisor, dividend);
            },
            &Self::Modulus { dividend, divisor } => {
                build_double_type(builder, opcode, divisor, dividend);
            },
            &Self::Add { augend, addend } => {
                build_double_type(builder, opcode, addend, augend);
            },
            &Self::Subtract { minuend, subtrahend } => {
                build_double_type(builder, opcode, subtrahend, minuend);
            },
            &Self::And { lhs, rhs } | &Self::Or { lhs, rhs } | &Self::Xor { lhs, rhs } => {
                build_double_type(builder, opcode, rhs, lhs);
            },
            &Self::Negate { data_type } | &Self::Not { data_type } => {
                build_single_type(builder, opcode, data_type);
            },
            &Self::ShiftLeft { value, shift_amount } => {
                build_double_type(builder, opcode, shift_amount, value);
            },
            &Self::ShiftRight { value, shift_amount } => {
                build_double_type(builder, opcode, shift_amount, value);
            },
            &Self::Compare { lhs, rhs, comparison_type } => {
                build_comparison(builder, opcode, rhs, lhs, comparison_type);
            },
            &Self::Pop {
                variable_ref,
                variable_type,
                type1,
                type2,
            } => {
                let instr_pos: usize = builder.len();
                builder.write_i16(build_instance_type(variable_type));
                builder.write_u8(u8::from(type1) | u8::from(type2) << 4);
                builder.write_u8(opcode);
                //let variable: &GMVariable = variable.variable.resolve(&builder.gm_data.variables)?;
                write_variable_occurrence(
                    builder,
                    variable_ref.index,
                    instr_pos,
                    /*variable.name.index,*/
                    0xDEAD_C0DE,
                    variable_type,
                )?;
            },
            &Self::PopSwap { is_array } => {
                build_popswap(builder, opcode, is_array);
            },
            &Self::Duplicate { data_type, size } => {
                build_duplicate(builder, opcode, data_type, size);
            },
            &Self::DuplicateSwap { data_type, size1, size2 } => {
                build_dupswap(builder, opcode, data_type, size1, size2);
            },
            Self::Return => {
                build_single_type(builder, opcode, GMDataType::Variable);
            },
            Self::Exit => build_single_type(builder, opcode, GMDataType::Int32),
            &Self::PopDiscard { data_type } => {
                build_single_type(builder, opcode, data_type);
            },
            &Self::Branch { jump_offset } => {
                build_branch(builder, opcode, jump_offset);
            },
            &Self::BranchIf { jump_offset } => {
                build_branch(builder, opcode, jump_offset);
            },
            &Self::BranchUnless { jump_offset } => {
                build_branch(builder, opcode, jump_offset);
            },
            &Self::PushWithContext { jump_offset } => {
                build_branch(builder, opcode, jump_offset);
            },
            &Self::PopWithContext { jump_offset } => {
                build_branch(builder, opcode, jump_offset);
            },
            Self::PopWithContextExit => build_popenv_exit(builder, opcode),
            Self::Push { value } => build_push(builder, opcode, &value)?,
            &Self::PushLocal { variable_ref, variable_type } => {
                build_pushvar(builder, opcode, variable_ref, variable_type)?;
            },
            &Self::PushGlobal { variable_ref, variable_type } => {
                build_pushvar(builder, opcode, variable_ref, variable_type)?;
            },
            &Self::PushBuiltin { variable_ref, variable_type } => {
                build_pushvar(builder, opcode, variable_ref, variable_type)?;
            },
            &Self::PushImmediate { integer } => {
                build_pushim(builder, opcode, integer);
            },
            &Self::Call { function, argument_count } => {
                build_call(builder, opcode, function, argument_count)?;
            },
            &Self::CallVariable { argument_count } => {
                build_callvar(builder, opcode, argument_count);
            },
            Self::CheckArrayIndex => {
                build_extended16(builder, opcodes::extended::CHKINDEX);
            },
            Self::PushArrayFinal => {
                build_extended16(builder, opcodes::extended::PUSHAF);
            },
            Self::PopArrayFinal => {
                build_extended16(builder, opcodes::extended::POPAF);
            },
            Self::PushArrayContainer => {
                build_extended16(builder, opcodes::extended::PUSHAC);
            },
            Self::SetArrayOwner => {
                build_extended16(builder, opcodes::extended::SETOWNER);
            },
            Self::HasStaticInitialized => {
                build_extended16(builder, opcodes::extended::ISSTATICOK);
            },
            Self::SetStaticInitialized => {
                build_extended16(builder, opcodes::extended::SETSTATIC);
            },
            Self::SaveArrayReference => {
                build_extended16(builder, opcodes::extended::SAVEAREF);
            },
            Self::RestoreArrayReference => {
                build_extended16(builder, opcodes::extended::RESTOREAREF);
            },
            Self::IsNullishValue => {
                build_extended16(builder, opcodes::extended::ISNULLISH);
            },
            Self::PushReference { asset_reference } => {
                build_pushref(builder, &asset_reference)?;
            },
        }
        Ok(())
    }
}

fn assert_type(expected: GMDataType, actual: GMDataType) -> Result<()> {
    assert_data_type("Instruction", expected, actual)
}

fn assert_zero_b0(b: [u8; 3]) -> Result<()> {
    assert_int("Instruction byte #0", 0, b[0])
}

fn assert_zero_b1(b: [u8; 3]) -> Result<()> {
    assert_int("Instruction byte #1", 0, b[1])
}

fn assert_zero_type2(b: [u8; 3]) -> Result<()> {
    assert_int("Instruction data type 2 (in byte #2)", 0, b[2] >> 4)
}

fn get_type1(b: [u8; 3]) -> Result<GMDataType> {
    num_enum_from(b[2] & 0xF)
}

fn get_type2(b: [u8; 3]) -> Result<GMDataType> {
    num_enum_from(b[2] >> 4)
}

/// This will not work for big endian (probably)
fn get_u24(b: [u8; 3]) -> u32 {
    let b0 = u32::from(b[0]);
    let b1 = u32::from(b[1]);
    let b2 = u32::from(b[2]);
    b0 | (b1 << 8) | (b2 << 16)
}

/// This will not work for big endian (probably)
fn get_u16(b: [u8; 3]) -> u16 {
    let b0 = u16::from(b[0]);
    let b1 = u16::from(b[1]);
    b0 | (b1 << 8)
}

fn parse_single_type(b: [u8; 3]) -> Result<GMDataType> {
    assert_zero_b0(b)?;
    assert_zero_b1(b)?;
    let data_type = get_type1(b)?;
    assert_zero_type2(b)?;
    Ok(data_type)
}

fn parse_double_type(b: [u8; 3]) -> Result<[GMDataType; 2]> {
    assert_zero_b0(b)?;
    assert_zero_b1(b)?;
    let right = get_type1(b)?;
    let left = get_type2(b)?;
    Ok([right, left])
}

fn parse_comparison(b: [u8; 3]) -> Result<GMInstruction> {
    assert_zero_b0(b)?;
    let comparison_type: GMComparisonType = num_enum_from(b[1])?;
    let rhs = get_type1(b)?;
    let lhs = get_type2(b)?;
    Ok(GMInstruction::Compare { lhs, rhs, comparison_type })
}

fn parse_pop(b: [u8; 3], reader: &mut DataReader) -> Result<GMInstruction> {
    let raw_instance_type = get_u16(b) as i16;
    let type1: GMDataType = num_enum_from(b[2] & 0xF)?;
    let type2: GMDataType = num_enum_from(b[2] >> 4)?;

    if type1 == GMDataType::Int16 {
        // PopSwap instruction
        assert_type(GMDataType::Variable, type2)?;

        let is_array = match raw_instance_type {
            5 => false,
            6 => true,
            n => bail!(
                "Expected 5 or 6 for \"instance type\" (aka SwapExtra) of PopSwap Instruction, got {n}"
            ),
        };
        return Ok(GMInstruction::PopSwap { is_array });
    }

    let (variable_ref, variable_type) = read_variable(reader, raw_instance_type)?;
    Ok(GMInstruction::Pop {
        variable_ref,
        variable_type,
        type1,
        type2,
    })
}

fn parse_duplicate(b: [u8; 3]) -> Result<GMInstruction> {
    let size: u8 = b[0];
    let mut size2: u8 = b[1];
    let data_type = get_type1(b)?;
    assert_zero_type2(b)?;

    if size2 == 0 {
        return Ok(GMInstruction::Duplicate { data_type, size });
    }

    // Duplicate Swap Instruction
    size2 = (size2 & 0x7F) >> 3;
    Ok(GMInstruction::DuplicateSwap { data_type, size1: size, size2 })
}

fn parse_branch(b: [u8; 3], reader: &DataReader) -> i32 {
    let mut value: u32 = get_u24(b);
    if reader.general_info.bytecode_version > 14 && (value & 0x40_0000) != 0 {
        value |= 0x80_0000;
    }
    if value & 0x80_0000 != 0 {
        (value | 0xFF00_0000) as i32
    } else {
        value as i32
    }
}

fn parse_push(b: [u8; 3], reader: &mut DataReader) -> Result<GMCodeValue> {
    let int16 = get_u16(b) as i16;
    let data_type = get_type1(b)?;
    assert_zero_type2(b)?;

    match data_type {
        GMDataType::Int16 => Ok(GMCodeValue::Int16(int16)),
        GMDataType::Int32 => {
            if let Some(&function_ref) = reader.function_occurrences.get(&reader.cur_pos) {
                reader.cur_pos += 4; // Skip next occurrence offset
                return Ok(GMCodeValue::Function { function_ref });
            }

            if let Some(&_variable) = reader.variable_occurrences.get(&reader.cur_pos) {
                reader.cur_pos += 4; // Skip next occurrence offset
                bail!(
                    "Found implicit Int32 variable reference at {}! Please report this message",
                    reader.cur_pos
                );
            }

            reader.read_i32().map(GMCodeValue::Int32)
        },
        GMDataType::Int64 => reader.read_i64().map(GMCodeValue::Int64),
        GMDataType::Double => reader.read_f64().map(GMCodeValue::Double),
        GMDataType::Boolean => reader.read_bool32().map(GMCodeValue::Boolean),
        GMDataType::String => {
            let index = reader.read_u32()? as usize;
            let len = reader.strings.len();
            let string = reader
                .strings
                .get(index)
                .ok_or_else(|| format!("String ID is out of range: {index} >= {len}"))?;
            Ok(GMCodeValue::String(string.clone()))
        },
        GMDataType::Variable => {
            let (variable_ref, variable_type) = read_variable(reader, int16)?;
            Ok(GMCodeValue::Variable { variable_ref, variable_type })
        },
    }
}

fn parse_push_var(
    b: [u8; 3],
    reader: &mut DataReader,
) -> Result<(GMRef<GMVariable>, GMVariableType)> {
    let raw_instance_type = get_u16(b) as i16;
    let data_type: GMDataType = get_type1(b)?;
    assert_zero_type2(b)?;
    assert_type(GMDataType::Variable, data_type)?;

    read_variable(reader, raw_instance_type)
}

fn parse_pushim(b: [u8; 3]) -> Result<i16> {
    let integer = get_u16(b) as i16;
    let data_type = get_type1(b)?;
    assert_zero_type2(b)?;
    assert_type(GMDataType::Int16, data_type)?;

    Ok(integer)
}

fn parse_call(b: [u8; 3], reader: &mut DataReader) -> Result<GMInstruction> {
    let argument_count: u16 = get_u16(b);
    let data_type: GMDataType = get_type1(b)?;
    assert_zero_type2(b)?;
    assert_type(GMDataType::Int32, data_type)?;

    let function: GMRef<GMFunction> = *reader
        .function_occurrences
        .get(&(reader.cur_pos))
        .ok_or_else(|| {
            format!(
                "Could not find any function with absolute occurrence position {} in map with length {} while parsing Call Instruction",
                reader.cur_pos,
                reader.function_occurrences.len(),
            )
        })?;
    reader.cur_pos += 4; // Skip next occurrence offset

    Ok(GMInstruction::Call { function, argument_count })
}

fn parse_callvar(b: [u8; 3]) -> Result<u16> {
    let argument_count: u16 = get_u16(b);
    let data_type: GMDataType = get_type1(b)?;
    assert_zero_type2(b)?;
    assert_type(GMDataType::Variable, data_type)?;

    Ok(argument_count)
}

fn parse_extended(reader: &mut DataReader, b: [u8; 3]) -> Result<GMInstruction> {
    use GMDataType::{Int16, Int32};
    use opcodes::extended::*;

    let kind = get_u16(b) as i16;
    let data_type: GMDataType = num_enum_from(b[2] & 0xF)?;
    assert_zero_type2(b)?;

    let instruction = match (data_type, kind) {
        (Int16, CHKINDEX) => GMInstruction::CheckArrayIndex,
        (Int16, PUSHAF) => GMInstruction::PushArrayFinal,
        (Int16, POPAF) => GMInstruction::PopArrayFinal,
        (Int16, PUSHAC) => GMInstruction::PushArrayContainer,
        (Int16, SETOWNER) => GMInstruction::SetArrayOwner,
        (Int16, ISSTATICOK) => GMInstruction::HasStaticInitialized,
        (Int16, SETSTATIC) => GMInstruction::SetStaticInitialized,
        (Int16, SAVEAREF) => GMInstruction::SaveArrayReference,
        (Int16, RESTOREAREF) => GMInstruction::RestoreArrayReference,
        (Int16, ISNULLISH) => GMInstruction::IsNullishValue,
        (Int32, PUSHREF) => {
            let asset_reference = GMAssetReference::deserialize(reader)
                .context("parsing PushReference Extended Instruction")?;
            GMInstruction::PushReference { asset_reference }
        },
        _ => bail!("Invalid Extended Instruction with data type {data_type:?} and kind {kind}"),
    };

    Ok(instruction)
}

fn build_single_type(builder: &mut DataBuilder, opcode: u8, data_type: GMDataType) {
    builder.write_u16(0);
    builder.write_u8(data_type.into());
    builder.write_u8(opcode);
}

fn build_double_type(builder: &mut DataBuilder, opcode: u8, type1: GMDataType, type2: GMDataType) {
    builder.write_u16(0);
    builder.write_u8(u8::from(type1) | u8::from(type2) << 4);
    builder.write_u8(opcode);
}

fn build_comparison(
    builder: &mut DataBuilder,
    mut opcode: u8,
    type1: GMDataType,
    type2: GMDataType,
    comparison_type: GMComparisonType,
) {
    let mut comparison_type = u8::from(comparison_type);
    if builder.bytecode_version() < 15 {
        opcode = 0x10 + comparison_type;
        comparison_type = 0;
    }
    builder.write_u8(0);
    builder.write_u8(comparison_type);
    builder.write_u8(u8::from(type1) | u8::from(type2) << 4);
    builder.write_u8(opcode);
}

fn build_popswap(builder: &mut DataBuilder, opcode: u8, array: bool) {
    builder.write_i16(if array { 6 } else { 5 });
    builder.write_u8(u8::from(GMDataType::Int32) | u8::from(GMDataType::Variable) << 4);
    builder.write_u8(opcode);
}

fn build_duplicate(builder: &mut DataBuilder, opcode: u8, data_type: GMDataType, size: u8) {
    builder.write_u8(size);
    builder.write_u8(0);
    builder.write_u8(data_type.into());
    builder.write_u8(opcode);
}

fn build_dupswap(
    builder: &mut DataBuilder,
    opcode: u8,
    data_type: GMDataType,
    size1: u8,
    size2: u8,
) {
    builder.write_u8(size1);
    builder.write_u8((size2 << 3) | 0x80);
    builder.write_u8(data_type.into());
    builder.write_u8(opcode);
}

fn build_branch(builder: &mut DataBuilder, opcode: u8, jump_offset: i32) {
    let mut value = (jump_offset as u32) & 0x00FF_FFFF;
    if builder.bytecode_version() > 14 && (value & 0x80_0000) != 0 {
        value &= !0x80_0000;
        value |= 0x40_0000;
    }
    builder.write_u8((value & 0xFF) as u8);
    builder.write_u8(((value >> 8) & 0xFF) as u8);
    builder.write_u8(((value >> 16) & 0xFF) as u8);
    builder.write_u8(opcode);
}

fn build_popenv_exit(builder: &mut DataBuilder, opcode: u8) {
    builder.write_u8(0x00);
    builder.write_u8(0x00);
    builder.write_u8(0xF0);
    builder.write_u8(opcode);
}

fn build_push(builder: &mut DataBuilder, opcode: u8, value: &GMCodeValue) -> Result<()> {
    let instr_pos: usize = builder.len();
    builder.write_i16(match value {
        &GMCodeValue::Int16(int16) => int16,
        &GMCodeValue::Variable { variable_type, .. } => build_instance_type(variable_type),
        _ => 0,
    });

    builder.write_u8(value.data_type().into());
    builder.write_u8(opcode);

    match value {
        GMCodeValue::Int16(_) => {}, // Nothing because it was already written inside the instruction
        &GMCodeValue::Int32(int32) => builder.write_i32(int32),
        &GMCodeValue::Int64(int64) => builder.write_i64(int64),
        &GMCodeValue::Double(double) => builder.write_f64(double),
        &GMCodeValue::Boolean(boolean) => builder.write_bool32(boolean),
        GMCodeValue::String(string) => {
            builder.write_gm_string_id(string.clone());
        },
        &GMCodeValue::Variable { variable_ref, variable_type } => {
            write_variable_occurrence(
                builder,
                variable_ref.index,
                instr_pos,
                0x6767_6767,
                variable_type,
            )?;
        },
        &GMCodeValue::Function { function_ref } => {
            write_function_occurrence(
                builder,
                function_ref.index,
                instr_pos,
                /*function.name.index*/ 0x6767_6767,
            )?;
        },
    }
    Ok(())
}

fn build_pushvar(
    builder: &mut DataBuilder,
    opcode: u8,
    variable_ref: GMRef<GMVariable>,
    variable_type: GMVariableType,
) -> Result<()> {
    let instr_pos = builder.len();
    builder.write_i16(build_instance_type(variable_type));
    builder.write_u8(GMDataType::Variable.into());
    builder.write_u8(opcode);

    write_variable_occurrence(
        builder,
        variable_ref.index,
        instr_pos,
        0xDEAD_C0DE,
        variable_type,
    )?;
    Ok(())
}

fn build_pushim(builder: &mut DataBuilder, opcode: u8, integer: i16) {
    builder.write_i16(integer);
    builder.write_u8(GMDataType::Int16.into());
    builder.write_u8(opcode);
}

fn build_call(
    builder: &mut DataBuilder,
    opcode: u8,
    function: GMRef<GMFunction>,
    argument_count: u16,
) -> Result<()> {
    let instr_pos: usize = builder.len();
    builder.write_u16(argument_count);
    builder.write_u8(GMDataType::Int32.into());
    builder.write_u8(opcode);

    write_function_occurrence(
        builder,
        function.index,
        instr_pos,
        /*function.name.index*/ 0xDEAD_C0DE,
    )?;
    Ok(())
}

fn build_callvar(builder: &mut DataBuilder, opcode: u8, argument_count: u16) {
    builder.write_u16(argument_count);
    builder.write_u8(GMDataType::Variable.into());
    builder.write_u8(opcode);
}

fn build_extended16(builder: &mut DataBuilder, extended_kind: i16) {
    builder.write_i16(extended_kind);
    builder.write_u8(GMDataType::Int16.into());
    builder.write_u8(opcodes::EXTENDED);
}

fn build_pushref(builder: &mut DataBuilder, asset_reference: &GMAssetReference) -> Result<()> {
    builder.write_i16(opcodes::extended::PUSHREF);
    builder.write_u8(GMDataType::Int32.into());
    builder.write_u8(opcodes::EXTENDED);
    asset_reference.serialize(builder)
}

impl GMElement for GMAssetReference {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if let Some(func) = reader.function_occurrences.get(&reader.cur_pos) {
            reader.cur_pos += 4; // Consume next occurrence offset
            return Ok(Self::Function(*func));
        }

        let raw = reader.read_u32()?;
        let index: u32 = (raw & 0xFF_FFFF) as u32;
        let asset_type: u8 = (raw >> 24) as u8;

        Ok(match asset_type {
            0 => Self::Object(GMRef::new(index)),
            1 => Self::Sprite(GMRef::new(index)),
            2 => Self::Sound(GMRef::new(index)),
            3 => Self::Room(GMRef::new(index)),
            4 => Self::Background(GMRef::new(index)),
            5 => Self::Path(GMRef::new(index)),
            6 => Self::Script(GMRef::new(index)),
            7 => Self::Font(GMRef::new(index)),
            8 => Self::Timeline(GMRef::new(index)),
            9 => Self::Shader(GMRef::new(index)),
            10 => Self::Sequence(GMRef::new(index)),
            11 => Self::AnimCurve(GMRef::new(index)),
            12 => Self::ParticleSystem(GMRef::new(index)),
            13 => Self::RoomInstance(index as i32),
            _ => bail!("Invalid asset type {asset_type}"),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let (index, asset_type) = match self {
            Self::Object(gm_ref) => (gm_ref.index, 0),
            Self::Sprite(gm_ref) => (gm_ref.index, 1),
            Self::Sound(gm_ref) => (gm_ref.index, 2),
            Self::Room(gm_ref) => (gm_ref.index, 3),
            Self::Background(gm_ref) => (gm_ref.index, 4),
            Self::Path(gm_ref) => (gm_ref.index, 5),
            Self::Script(gm_ref) => (gm_ref.index, 6),
            Self::Font(gm_ref) => (gm_ref.index, 7),
            Self::Timeline(gm_ref) => (gm_ref.index, 8),
            Self::Shader(gm_ref) => (gm_ref.index, 9),
            Self::Sequence(gm_ref) => (gm_ref.index, 10),
            Self::AnimCurve(gm_ref) => (gm_ref.index, 11),
            Self::ParticleSystem(gm_ref) => (gm_ref.index, 12),
            Self::RoomInstance(id) => (*id as u32, 13),
            Self::Function(func_ref) => {
                //let function: &GMFunction =
                func_ref.resolve(&builder.gm_data.functions)?;
                write_function_occurrence(
                    builder,
                    func_ref.index,
                    builder.len(),
                    /*function.name.index*/ 0x6767_6767,
                )?;
                return Ok(());
            },
        };

        let raw: u32 = (asset_type << 24) | index & 0xFF_FFFF;
        builder.write_u32(raw);
        Ok(())
    }
}

fn read_variable(
    reader: &mut DataReader,
    raw_instance_type: i16,
) -> Result<(GMRef<GMVariable>, GMVariableType)> {
    let occurrence_position: u32 = reader.cur_pos;
    let raw_value = reader.read_u32()?;
    let variable_type = (raw_value >> 24) & 0xF8;

    let variable_type = match variable_type {
        0x00 => GMVariableType::Array,
        0x80 => GMVariableType::StackTopChain,
        0xA0 => parse_instance_type(raw_instance_type)?,
        0xE0 => GMVariableType::Instance(raw_instance_type),
        0x10 => GMVariableType::ArrayPushAF,
        0x90 => GMVariableType::ArrayPopAF,
        _ => bail!("Invalid Variable Type 0x{variable_type:02X}"),
    };

    let variable: GMRef<GMVariable> = *reader.variable_occurrences.get(&occurrence_position).ok_or_else(|| {
        format!(
            "Could not find variable with occurrence position {} in hashmap with length {} while parsing code value",
            occurrence_position,
            reader.variable_occurrences.len(),
        )
    })?;

    Ok((variable, variable_type))
}

pub fn parse_instance_type(raw_value: i16) -> Result<GMVariableType> {
    Ok(match raw_value {
        -1 => GMVariableType::Self_,
        -2 => GMVariableType::Other,
        -3 => GMVariableType::All,
        -4 => GMVariableType::None,
        -5 => GMVariableType::Global,
        -6 => GMVariableType::Builtin,
        -7 => GMVariableType::Local,
        -9 => GMVariableType::StackTopInstance,
        -15 => GMVariableType::Argument,
        -16 => GMVariableType::Static,
        n if n > 0 => GMVariableType::GameObject(GMRef::new(n as u32)),
        _ => bail!("Invalid instance type {raw_value} (0x{raw_value:04X})"),
    })
}

#[must_use]
pub(crate) const fn build_instance_type(variable_type: GMVariableType) -> i16 {
    match variable_type {
        GMVariableType::Self_ => -1,
        GMVariableType::GameObject(game_object_ref) => game_object_ref.index as i16,
        GMVariableType::Instance(instance_id) => instance_id,
        GMVariableType::Other => -2,
        GMVariableType::All => -3,
        GMVariableType::None => -4,
        GMVariableType::Global => -5,
        GMVariableType::Builtin => -6,
        GMVariableType::Local => -7,
        GMVariableType::StackTopInstance => -9,
        GMVariableType::Argument => -15,
        GMVariableType::Static => -16,
        GMVariableType::StackTopChain
        | GMVariableType::Array
        | GMVariableType::ArrayPushAF
        | GMVariableType::ArrayPopAF => 0,
    }
}

#[must_use]
pub(crate) const fn build_variable_type(variable_type: GMVariableType) -> u8 {
    match variable_type {
        GMVariableType::GameObject(_) => 0xA0,
        GMVariableType::Self_ => 0xA0,
        GMVariableType::Other => 0xA0,
        GMVariableType::All => 0xA0,
        GMVariableType::None => 0xA0,
        GMVariableType::Global => 0xA0,
        GMVariableType::Builtin => 0xA0,
        GMVariableType::Local => 0xA0,
        GMVariableType::Argument => 0xA0,
        GMVariableType::Static => 0xA0,
        GMVariableType::StackTopInstance => 0xA0,
        GMVariableType::StackTopChain => 0x80,
        GMVariableType::Array => 0x00,
        GMVariableType::Instance(_) => 0xE0,
        GMVariableType::ArrayPushAF => 0x10,
        GMVariableType::ArrayPopAF => 0x90,
    }
}

fn write_variable_occurrence(
    builder: &mut DataBuilder,
    gm_index: u32,
    occurrence_pos: usize,
    name_string_id: u32,
    variable_type: GMVariableType,
) -> Result<()> {
    let len: usize = builder.variable_occurrences.len();
    let occurrences: &mut Vec<(usize, GMVariableType)> = builder
        .variable_occurrences
        .get_mut(gm_index as usize)
        .ok_or_else(|| {
            format!("Invalid Variable GMRef while writing occurrence: {gm_index} >= {len}")
        })?;

    if let Some(&(last_occurrence_pos, old_variable_type)) = occurrences.last() {
        // Replace last occurrence with next occurrence offset
        let occurrence_offset: i32 = occurrence_pos as i32 - last_occurrence_pos as i32;
        let occurrence_offset_full: i32 = occurrence_offset & 0x07FF_FFFF
            | (i32::from(build_variable_type(old_variable_type) & 0xF8) << 24);
        builder.overwrite_i32(occurrence_offset_full, last_occurrence_pos + 4)?;
    }

    // Write name string id for this occurrence. this is correct if it is the last occurrence.
    // Otherwise, it will be overwritten later by the code above.
    // Hopefully, writing the name string id instead of -1 for unused variables will be fine.
    builder.write_u32(
        name_string_id & 0x07FF_FFFF | (u32::from(build_variable_type(variable_type) & 0xF8) << 24),
    );

    // Fuckass borrow checker
    builder
        .variable_occurrences
        .get_mut(gm_index as usize)
        .unwrap()
        .push((occurrence_pos, variable_type));
    Ok(())
}

fn write_function_occurrence(
    builder: &mut DataBuilder,
    gm_index: u32,
    occurrence_pos: usize,
    name_string_id: u32,
) -> Result<()> {
    let len: usize = builder.function_occurrences.len();
    let occurrences: &mut Vec<usize> = builder
        .function_occurrences
        .get_mut(gm_index as usize)
        .ok_or_else(|| {
            format!("Invalid Function GMRef while writing occurrence: {gm_index} >= {len}")
        })?;

    if let Some(&last_occurrence_pos) = occurrences.last() {
        // Replace last occurrence with next occurrence offset
        let occurrence_offset: i32 = occurrence_pos as i32 - last_occurrence_pos as i32;
        builder.overwrite_i32(occurrence_offset & 0x07FF_FFFF, last_occurrence_pos + 4)?;
    }

    // Write name string id for this occurrence. this is correct if it is the last occurrence.
    // Otherwise, it will be overwritten later by the code above.
    builder.write_u32(name_string_id & 0x07FF_FFFF);

    builder
        .function_occurrences
        .get_mut(gm_index as usize)
        .unwrap()
        .push(occurrence_pos);
    Ok(())
}

/// Check whether this data file was generated with `YYC` (`YoYoGames Compiler`).
/// Should that be the case, the `CODE`, `VARI` and `FUNC` chunks will be empty
/// (or not exist, depending on the bytecode version).
/// NOTE: YYC is untested. Issues may occur.
pub(crate) fn check_yyc(reader: &DataReader) -> Result<bool> {
    // If the CODE chunk doesn't exist; the data file was compiled with YYC.
    let Some(code) = reader.chunks.get("CODE") else {
        return Ok(true);
    };

    let vari = reader
        .chunks
        .get("VARI")
        .ok_or("Chunk CODE exists but VARI doesn't")?;

    let func = reader
        .chunks
        .get("FUNC")
        .ok_or("Chunk CODE and VARI exist but FUNC doesn't")?;

    // If the CODE chunk exists but is completely empty,
    // the data file was compiled with YYC before bytecode 17.
    if !code.is_empty() {
        return Ok(false);
    }

    if reader.general_info.bytecode_version > 16 {
        bail!("Empty, but existant CODE chunk before bytecode 17");
    }

    if !vari.is_empty() {
        bail!("Chunk CODE is empty but VARI is not");
    }

    if !func.is_empty() {
        bail!("Chunk CODE and VARI are empty but FUNC is not");
    }

    Ok(true)
}
