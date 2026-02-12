use std::collections::HashMap;

use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, element_stub, function::GMFunction, variable::GMVariable},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::{
        GMCode, ModernData,
        instruction::{
            AssetReference, CodeVariable, ComparisonType, DataType, InstanceType, Instruction,
            PushValue, VariableType,
        },
        opcodes,
    },
    prelude::*,
    util::init::{num_enum_from, vec_with_capacity},
};

#[named_list_chunk("CODE")]
pub struct GMCodes {
    pub codes: Vec<GMCode>,
    pub exists: bool,
}

element_stub!(GMCode);

impl GMElement for GMCodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        // This can happen with YYC/
        if reader.chunk.is_empty() {
            return Ok(Self { codes: vec![], exists: false });
        }

        let pointers: Vec<u32> = reader.read_simple_list()?;
        let count: usize = pointers.len();

        let Some(&first_pos) = pointers.first() else {
            return Ok(Self { codes: vec![], exists: true });
        };
        reader.cur_pos = first_pos;

        let mut codes: Vec<GMCode> = vec_with_capacity(count as u32)?;
        let mut instructions_ranges: Vec<(u32, u32)> = Vec::with_capacity(count);
        let mut codes_by_pos: HashMap<u32, GMRef<GMCode>> = HashMap::new();
        let mut last_code_entry_pos = reader.cur_pos;

        for pointer in pointers {
            reader.assert_pos(pointer, "Code")?;
            let name: String = reader.read_gm_string()?;
            let code_length = reader.read_u32()?;

            let instructions_start_pos;
            let instructions_end_pos;
            let modern_data: Option<ModernData>;

            if reader.general_info.wad_version <= 14 {
                instructions_start_pos = reader.cur_pos; // Instructions are placed immediately after code metadata; how convenient!
                reader.cur_pos += code_length; // Skip over them; they will get parsed in the next loops
                instructions_end_pos = reader.cur_pos;
                modern_data = None;
            } else {
                let locals_count = reader.read_u16()?;
                let arguments_count_raw = reader.read_u16()?;
                let arguments_count: u16 = arguments_count_raw & 0x7FFF;
                let weird_local_flag: bool = arguments_count_raw & 0x8000 != 0;

                let position = reader.cur_pos;
                let instructions_start_offset = reader.read_i32()?;
                instructions_start_pos = position
                    .checked_add_signed(instructions_start_offset)
                    .ok_or("Instruction start position overflowed")?;

                let offset = reader.read_u32()?;

                instructions_end_pos = instructions_start_pos
                    .checked_add(code_length)
                    .ok_or("Instruction end position overflowed")?;

                let data = ModernData {
                    locals_count,
                    arguments_count,
                    weird_local_flag,
                    offset,
                    parent: None,
                };
                modern_data = Some(data);
            }

            codes.push(GMCode { name, instructions: vec![], modern_data });

            instructions_ranges.push((instructions_start_pos, instructions_end_pos));
            last_code_entry_pos = reader.cur_pos;
        }

        for (i, (start, end)) in instructions_ranges.into_iter().enumerate() {
            let code: &mut GMCode = &mut codes[i];
            let length = end - start;

            // If WAD15+ and the instructions pointer is known, then it's a child code entry
            if length > 0
                && let Some(parent_code) = codes_by_pos.get(&start)
                && let Some(data) = &mut code.modern_data
            {
                data.parent = Some(*parent_code);
                continue;
            }

            reader.cur_pos = start;
            // Estimated Size: https://discord.com/channels/566861759210586112/568625491876118528/1424403240258371615
            code.instructions = vec_with_capacity(length / 5)?;

            if length > 0 {
                // Update information to mark this entry as the root (if we have at least 1 instruction)
                codes_by_pos.insert(start, i.into());
            }

            while reader.cur_pos < end {
                let instruction = Instruction::deserialize(reader)
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

        reader.cur_pos = last_code_entry_pos;
        // Set pos to the supposed chunk end (since instructions are stored separately in WAD15+)

        Ok(Self { codes, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.codes.len())?;
        let pointer_list_pos: usize = builder.len();
        for _ in 0..self.codes.len() {
            builder.write_u32(0xDEAD_C0DE);
        }

        // WAD <= 14 my beloved
        if builder.wad_version() <= 14 {
            for (i, code) in self.codes.iter().enumerate() {
                builder.overwrite_usize(builder.len(), pointer_list_pos + 4 * i)?;
                builder.write_gm_string(&code.name);
                let length_placeholder_pos: usize = builder.len();
                builder.write_u32(0xDEAD_C0DE);
                let start: usize = builder.len();

                // In WAD <= 14, instructions are written immediately
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

        // In WAD 15+, the codes' instructions are written before the codes metadata
        let mut instructions_ranges: Vec<(usize, usize)> = Vec::with_capacity(self.codes.len());

        for (i, code) in self.codes.iter().enumerate() {
            if code.modern_data.as_ref().unwrap().parent.is_some() {
                // If this is a child code entry, don't write instructions; just repeat last pointer
                let prev_range = instructions_ranges
                    .last()
                    .ok_or("First code entry is a child code entry")?;
                instructions_ranges.push(*prev_range);
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
            let data: &ModernData = code.modern_data.as_ref().ok_or_else(|| {
                format!(
                    "Code WAD15+ data not set in WAD version {}",
                    builder.wad_version()
                )
            })?;

            builder.write_gm_string(&code.name);
            builder.write_usize(length)?;
            builder.write_u16(data.locals_count);
            builder
                .write_u16(data.arguments_count | if data.weird_local_flag { 0x8000 } else { 0 });
            let instructions_start_offset: i32 = start as i32 - builder.len() as i32;
            builder.write_i32(instructions_start_offset);
            builder.write_u32(data.offset);
        }

        Ok(())
    }
}

impl GMElement for Instruction {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let word = reader.read_u32()?;
        let mut opcode = ((word & 0xFF00_0000) >> 24) as u8;
        let b2 = ((word & 0x00FF_0000) >> 16) as u8;
        let b1 = ((word & 0x0000_FF00) >> 8) as u8;
        let b0 = (word & 0x0000_00FF) as u8;
        let mut b = [b0, b1, b2];

        if reader.general_info.wad_version < 15 {
            if matches!(opcode, 0x10..=0x16) {
                // This is needed to preserve the comparison type for pre WAD 15
                reader.assert_zero_b1(b)?;
                b[1] = opcode - 0x10;
            }
            opcode = opcodes::old_to_new(opcode);
        }

        // log::debug!("{} // {:02X} {:02X} {:02X} {:02X}", reader.cur_pos-4, b[0], b[1], b[2], opcode);

        Ok(match opcode {
            opcodes::CONV => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Convert Instruction")?;
                Self::Convert { from: types[0], to: types[1] }
            },
            opcodes::MUL => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Multiply Instruction")?;
                Self::Multiply {
                    multiplicand: types[1],
                    multiplier: types[0],
                }
            },
            opcodes::DIV => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Divide Instruction")?;
                Self::Divide { dividend: types[1], divisor: types[0] }
            },
            opcodes::REM => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Remainder Instruction")?;
                Self::Remainder { dividend: types[1], divisor: types[0] }
            },
            opcodes::MOD => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Modulus Instruction")?;
                Self::Modulus { dividend: types[1], divisor: types[0] }
            },
            opcodes::ADD => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Add Instruction")?;
                Self::Add { augend: types[1], addend: types[0] }
            },
            opcodes::SUB => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Subtract Instruction")?;
                Self::Subtract { minuend: types[1], subtrahend: types[0] }
            },
            opcodes::AND => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing And Instruction")?;
                Self::And { lhs: types[1], rhs: types[0] }
            },
            opcodes::OR => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Or Instruction")?;
                Self::Or { lhs: types[1], rhs: types[0] }
            },
            opcodes::XOR => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing Xor Instruction")?;
                Self::Xor { lhs: types[1], rhs: types[0] }
            },
            opcodes::NEG => {
                let data_type = reader
                    .parse_single_type(b)
                    .context("parsing Negate Instruction")?;
                Self::Negate { data_type }
            },
            opcodes::NOT => {
                let data_type = reader
                    .parse_single_type(b)
                    .context("parsing Not Instruction")?;
                Self::Not { data_type }
            },
            opcodes::SHL => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing ShiftLeft instruction")?;
                Self::ShiftLeft { value: types[1], shift_amount: types[0] }
            },
            opcodes::SHR => {
                let types = reader
                    .parse_double_type(b)
                    .context("parsing ShiftRight Instruction")?;
                Self::ShiftRight { value: types[1], shift_amount: types[0] }
            },
            opcodes::CMP => reader
                .parse_comparison(b)
                .context("parsing Comparison Instruction")?,
            opcodes::POP => reader.parse_pop(b).context("parsing Pop Instruction")?,
            opcodes::DUP => reader
                .parse_duplicate(b)
                .context("parsing Duplicate Instruction")?,
            opcodes::RET => {
                let ctx = "parsing Return Instruction";
                let data_type = reader.parse_single_type(b).context(ctx)?;
                reader
                    .assert_type(DataType::Variable, data_type)
                    .context(ctx)?;
                Self::Return
            },
            opcodes::EXIT => {
                let ctx = "parsing Exit Instruction";
                let data_type = reader.parse_single_type(b).context(ctx)?;
                reader
                    .assert_type(DataType::Int32, data_type)
                    .context(ctx)?;
                Self::Exit
            },
            opcodes::POPZ => {
                let data_type = reader
                    .parse_single_type(b)
                    .context("parsing PopDiscard Instruction")?;
                Self::PopDiscard { data_type }
            },
            opcodes::JMP => Self::Branch { jump_offset: reader.parse_branch(b) },
            opcodes::JT => Self::BranchIf { jump_offset: reader.parse_branch(b) },
            opcodes::JF => Self::BranchUnless { jump_offset: reader.parse_branch(b) },
            opcodes::PUSHENV => Self::PushWithContext { jump_offset: reader.parse_branch(b) },
            opcodes::POPENV if b == [0x00, 0x00, 0xF0] => Self::PopWithContextExit,
            opcodes::POPENV => Self::PopWithContext { jump_offset: reader.parse_branch(b) },
            opcodes::PUSH => {
                let value = reader.parse_push(b).context("parsing Push Instruction")?;
                Self::Push { value }
            },
            opcodes::PUSHLOC => {
                let variable = reader
                    .parse_push_var(b)
                    .context("parsing PushLocal Instruction")?;
                Self::PushLocal { variable }
            },
            opcodes::PUSHGLB => {
                let variable = reader
                    .parse_push_var(b)
                    .context("parsing PushGlobal Instruction")?;
                Self::PushGlobal { variable }
            },
            opcodes::PUSHBLTN => {
                let variable = reader
                    .parse_push_var(b)
                    .context("parsing PushBuiltin Instruction")?;
                Self::PushBuiltin { variable }
            },
            opcodes::PUSHIM => {
                let integer = reader
                    .parse_pushim(b)
                    .context("parsing PushImmediate Instruction")?;
                Self::PushImmediate { integer }
            },
            opcodes::CALL => reader.parse_call(b).context("parsing Call Instruction")?,
            opcodes::CALLVAR => {
                let argument_count = reader
                    .parse_callvar(b)
                    .context("parsing CallVariable Instruction")?;
                Self::CallVariable { argument_count }
            },
            opcodes::EXTENDED => reader
                .parse_extended(b)
                .context("parsing Extended Instruction")?,
            _ => bail!("Invalid Instruction Opcode {opcode} (0x{opcode:02X})"),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut opcode: u8 = self.opcode();
        if builder.wad_version() < 15 {
            opcode = opcodes::new_to_old(opcode);
        }

        match self {
            &Self::Negate { data_type }
            | &Self::Not { data_type }
            | &Self::PopDiscard { data_type } => {
                build_single_type(builder, opcode, data_type);
            },

            &Self::Convert { from: type1, to: type2 }
            | &Self::Multiply { multiplicand: type2, multiplier: type1 }
            | &Self::Divide { dividend: type2, divisor: type1 }
            | &Self::Remainder { dividend: type2, divisor: type1 }
            | &Self::Modulus { dividend: type2, divisor: type1 }
            | &Self::Add { augend: type2, addend: type1 }
            | &Self::Subtract { minuend: type2, subtrahend: type1 }
            | &Self::And { lhs: type2, rhs: type1 }
            | &Self::Or { lhs: type2, rhs: type1 }
            | &Self::Xor { lhs: type2, rhs: type1 }
            | &Self::ShiftLeft { value: type2, shift_amount: type1 }
            | &Self::ShiftRight { value: type2, shift_amount: type1 } => {
                build_double_type(builder, opcode, type1, type2);
            },

            &Self::Compare { lhs, rhs, comparison_type } => {
                build_comparison(builder, opcode, rhs, lhs, comparison_type);
            },
            Self::Pop { variable, type1, type2 } => {
                build_pop(builder, opcode, variable, *type1, *type2)?;
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
                build_single_type(builder, opcode, DataType::Variable);
            },
            Self::Exit => build_single_type(builder, opcode, DataType::Int32),

            &Self::Branch { jump_offset }
            | &Self::BranchIf { jump_offset }
            | &Self::BranchUnless { jump_offset }
            | &Self::PushWithContext { jump_offset }
            | &Self::PopWithContext { jump_offset } => {
                build_branch(builder, opcode, jump_offset);
            },
            Self::PopWithContextExit => build_popenv_exit(builder, opcode),
            Self::Push { value } => build_push(builder, opcode, value)?,
            Self::PushLocal { variable }
            | Self::PushGlobal { variable }
            | Self::PushBuiltin { variable } => {
                build_pushvar(builder, opcode, variable)?;
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
                build_pushref(builder, *asset_reference)?;
            },
        }
        Ok(())
    }
}

fn get_type1(b: [u8; 3]) -> Result<DataType> {
    num_enum_from(b[2] & 0xF)
}

fn get_type2(b: [u8; 3]) -> Result<DataType> {
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

impl DataReader<'_> {
    fn assert_type(&self, actual: DataType, expected: DataType) -> Result<()> {
        self.assert_data_type(actual, expected, "Instruction")
    }

    fn assert_zero_b0(&self, b: [u8; 3]) -> Result<()> {
        self.assert_int(b[0], 0, "Instruction byte #0")
    }

    fn assert_zero_b1(&self, b: [u8; 3]) -> Result<()> {
        self.assert_int(b[1], 0, "Instruction byte #1")
    }

    fn assert_zero_type2(&self, b: [u8; 3]) -> Result<()> {
        self.assert_int(b[2] >> 4, 0, "Instruction data type 2 (in byte #2)")
    }

    fn parse_single_type(&self, b: [u8; 3]) -> Result<DataType> {
        self.assert_zero_b0(b)?;
        self.assert_zero_b1(b)?;
        let data_type = get_type1(b)?;
        self.assert_zero_type2(b)?;
        Ok(data_type)
    }

    fn parse_double_type(&self, b: [u8; 3]) -> Result<[DataType; 2]> {
        self.assert_zero_b0(b)?;
        self.assert_zero_b1(b)?;
        let right = get_type1(b)?;
        let left = get_type2(b)?;
        Ok([right, left])
    }

    fn parse_comparison(&self, b: [u8; 3]) -> Result<Instruction> {
        self.assert_zero_b0(b)?;
        let comparison_type: ComparisonType = num_enum_from(b[1])?;
        let rhs = get_type1(b)?;
        let lhs = get_type2(b)?;
        Ok(Instruction::Compare { lhs, rhs, comparison_type })
    }

    fn parse_pop(&mut self, b: [u8; 3]) -> Result<Instruction> {
        let raw_instance_type = get_u16(b) as i16;
        let type1: DataType = get_type1(b)?;
        let type2: DataType = get_type2(b)?;

        if type1 == DataType::Int16 {
            // PopSwap instruction
            self.assert_type(DataType::Variable, type2)?;

            let is_array = match raw_instance_type {
                5 => false,
                6 => true,
                n => bail!(
                    "Expected 5 or 6 for \"instance type\" (aka SwapExtra) of PopSwap Instruction, got {n}"
                ),
            };
            return Ok(Instruction::PopSwap { is_array });
        }

        let variable: CodeVariable = read_variable(self, raw_instance_type)?;
        Ok(Instruction::Pop { variable, type1, type2 })
    }

    fn parse_duplicate(&self, b: [u8; 3]) -> Result<Instruction> {
        let size: u8 = b[0];
        let mut size2: u8 = b[1];
        let data_type = get_type1(b)?;
        self.assert_zero_type2(b)?;

        if size2 == 0 {
            return Ok(Instruction::Duplicate { data_type, size });
        }

        // Duplicate Swap Instruction
        size2 = (size2 & 0x7F) >> 3;
        Ok(Instruction::DuplicateSwap { data_type, size1: size, size2 })
    }

    fn parse_branch(&self, b: [u8; 3]) -> i32 {
        let mut value: u32 = get_u24(b);
        if self.general_info.wad_version > 14 && (value & 0x40_0000) != 0 {
            value |= 0x80_0000;
        }
        if value & 0x80_0000 != 0 {
            (value | 0xFF00_0000) as i32
        } else {
            value as i32
        }
    }

    fn parse_push(&mut self, b: [u8; 3]) -> Result<PushValue> {
        let int16 = get_u16(b) as i16;
        let data_type = get_type1(b)?;
        self.assert_zero_type2(b)?;

        match data_type {
            DataType::Int16 => Ok(PushValue::Int16(int16)),
            DataType::Int32 => {
                if let Some(&function) = self.function_occurrences.get(&self.cur_pos) {
                    self.cur_pos += 4; // Skip next occurrence offset
                    return Ok(PushValue::Function(function));
                }

                if let Some(&(variable, _)) = self.variable_occurrences.get(&self.cur_pos) {
                    self.cur_pos += 4; // Skip next occurrence offset
                    return Ok(PushValue::Variable(CodeVariable {
                        variable,
                        variable_type: VariableType::Normal,
                        instance_type: InstanceType::Self_,
                        is_int32: true,
                    }));
                }

                self.read_i32().map(PushValue::Int32)
            },
            DataType::Int64 => self.read_i64().map(PushValue::Int64),
            DataType::Double => self.read_f64().map(PushValue::Double),
            DataType::Boolean => self.read_bool32().map(PushValue::Boolean),
            DataType::String => {
                let index = self.read_u32()? as usize;
                let len = self.strings.len();
                let string = self
                    .strings
                    .get(index)
                    .ok_or_else(|| format!("String ID is out of range: {index} >= {len}"))?;
                Ok(PushValue::String(string.clone()))
            },
            DataType::Variable => read_variable(self, int16).map(PushValue::Variable),
        }
    }

    fn parse_push_var(&mut self, b: [u8; 3]) -> Result<CodeVariable> {
        let raw_instance_type = get_u16(b) as i16;
        let data_type: DataType = get_type1(b)?;
        self.assert_zero_type2(b)?;
        self.assert_type(DataType::Variable, data_type)?;

        read_variable(self, raw_instance_type)
    }

    fn parse_pushim(&self, b: [u8; 3]) -> Result<i16> {
        let integer = get_u16(b) as i16;
        let data_type = get_type1(b)?;
        self.assert_zero_type2(b)?;
        self.assert_type(DataType::Int16, data_type)?;

        Ok(integer)
    }

    fn parse_call(&mut self, b: [u8; 3]) -> Result<Instruction> {
        let argument_count: u16 = get_u16(b);
        let data_type: DataType = get_type1(b)?;
        self.assert_zero_type2(b)?;
        self.assert_type(DataType::Int32, data_type)?;

        let function: GMRef<GMFunction> = *self
            .function_occurrences
            .get(&(self.cur_pos))
            .ok_or_else(|| {
                format!(
                    "Could not find any function with absolute occurrence position {} in map with length {} while parsing Call Instruction",
                    self.cur_pos,
                    self.function_occurrences.len(),
                )
            })?;
        self.cur_pos += 4; // Skip next occurrence offset

        Ok(Instruction::Call { function, argument_count })
    }

    fn parse_callvar(&self, b: [u8; 3]) -> Result<u16> {
        let argument_count: u16 = get_u16(b);
        let data_type: DataType = get_type1(b)?;
        self.assert_zero_type2(b)?;
        self.assert_type(DataType::Variable, data_type)?;

        Ok(argument_count)
    }

    fn parse_extended(&mut self, b: [u8; 3]) -> Result<Instruction> {
        use DataType::{Int16, Int32};
        #[allow(clippy::wildcard_imports)]
        use opcodes::extended::*;

        let kind = get_u16(b) as i16;
        let data_type: DataType = num_enum_from(b[2] & 0xF)?;
        self.assert_zero_type2(b)?;

        let instruction = match (data_type, kind) {
            (Int16, CHKINDEX) => Instruction::CheckArrayIndex,
            (Int16, PUSHAF) => Instruction::PushArrayFinal,
            (Int16, POPAF) => Instruction::PopArrayFinal,
            (Int16, PUSHAC) => Instruction::PushArrayContainer,
            (Int16, SETOWNER) => Instruction::SetArrayOwner,
            (Int16, ISSTATICOK) => Instruction::HasStaticInitialized,
            (Int16, SETSTATIC) => Instruction::SetStaticInitialized,
            (Int16, SAVEAREF) => Instruction::SaveArrayReference,
            (Int16, RESTOREAREF) => Instruction::RestoreArrayReference,
            (Int16, ISNULLISH) => Instruction::IsNullishValue,
            (Int32, PUSHREF) => {
                let asset_reference = AssetReference::deserialize(self)
                    .context("parsing PushReference Extended Instruction")?;
                Instruction::PushReference { asset_reference }
            },
            _ => bail!("Invalid Extended Instruction with data type {data_type:?} and kind {kind}"),
        };

        Ok(instruction)
    }
}

fn build_single_type(builder: &mut DataBuilder, opcode: u8, data_type: DataType) {
    builder.write_u16(0);
    builder.write_u8(data_type.into());
    builder.write_u8(opcode);
}

fn build_double_type(builder: &mut DataBuilder, opcode: u8, type1: DataType, type2: DataType) {
    builder.write_u16(0);
    builder.write_u8(u8::from(type1) | u8::from(type2) << 4);
    builder.write_u8(opcode);
}

fn build_comparison(
    builder: &mut DataBuilder,
    mut opcode: u8,
    type1: DataType,
    type2: DataType,
    comparison_type: ComparisonType,
) {
    let mut comparison_type = u8::from(comparison_type);
    if builder.wad_version() < 15 {
        opcode = 0x10 + comparison_type;
        comparison_type = 0;
    }
    builder.write_u8(0);
    builder.write_u8(comparison_type);
    builder.write_u8(u8::from(type1) | u8::from(type2) << 4);
    builder.write_u8(opcode);
}

fn build_pop(
    builder: &mut DataBuilder,
    opcode: u8,
    variable: &CodeVariable,
    type1: DataType,
    type2: DataType,
) -> Result<()> {
    let instr_pos: usize = builder.len();
    builder.write_i16(variable.instance_type.build());
    builder.write_u8(u8::from(type1) | u8::from(type2) << 4);
    builder.write_u8(opcode);
    write_variable_occurrence(
        builder,
        variable.variable.index,
        instr_pos,
        variable.variable_type,
    )?;
    Ok(())
}

fn build_popswap(builder: &mut DataBuilder, opcode: u8, array: bool) {
    builder.write_i16(if array { 6 } else { 5 });
    builder.write_u8(u8::from(DataType::Int32) | u8::from(DataType::Variable) << 4);
    builder.write_u8(opcode);
}

fn build_duplicate(builder: &mut DataBuilder, opcode: u8, data_type: DataType, size: u8) {
    builder.write_u8(size);
    builder.write_u8(0);
    builder.write_u8(data_type.into());
    builder.write_u8(opcode);
}

fn build_dupswap(builder: &mut DataBuilder, opcode: u8, data_type: DataType, size1: u8, size2: u8) {
    builder.write_u8(size1);
    builder.write_u8((size2 << 3) | 0x80);
    builder.write_u8(data_type.into());
    builder.write_u8(opcode);
}

fn build_branch(builder: &mut DataBuilder, opcode: u8, jump_offset: i32) {
    let mut value = (jump_offset as u32) & 0x00FF_FFFF;
    if builder.wad_version() > 14 && (value & 0x80_0000) != 0 {
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

fn build_push(builder: &mut DataBuilder, opcode: u8, value: &PushValue) -> Result<()> {
    let instr_pos: usize = builder.len();
    builder.write_i16(match value {
        PushValue::Int16(int16) => *int16,
        PushValue::Variable(variable) => variable.instance_type.build(),
        _ => 0,
    });

    builder.write_u8(value.data_type().into());
    builder.write_u8(opcode);

    match value {
        PushValue::Int16(_) => {}, // Nothing because it was already written inside the instruction
        PushValue::Int32(int32) => builder.write_i32(*int32),
        PushValue::Int64(int64) => builder.write_i64(*int64),
        PushValue::Double(double) => builder.write_f64(*double),
        PushValue::Boolean(boolean) => builder.write_bool32(*boolean),
        PushValue::String(string) => {
            builder.write_gm_string_id(string.clone());
        },
        PushValue::Variable(code_variable) => {
            write_variable_occurrence(
                builder,
                code_variable.variable.index,
                instr_pos,
                code_variable.variable_type,
            )?;
        },
        PushValue::Function(func_ref) => {
            write_function_occurrence(builder, func_ref.index, instr_pos)?;
        },
    }
    Ok(())
}

fn build_pushvar(builder: &mut DataBuilder, opcode: u8, variable: &CodeVariable) -> Result<()> {
    let instr_pos = builder.len();
    builder.write_i16(variable.instance_type.build());
    builder.write_u8(DataType::Variable.into());
    builder.write_u8(opcode);

    write_variable_occurrence(
        builder,
        variable.variable.index,
        instr_pos,
        variable.variable_type,
    )?;
    Ok(())
}

fn build_pushim(builder: &mut DataBuilder, opcode: u8, integer: i16) {
    builder.write_i16(integer);
    builder.write_u8(DataType::Int16.into());
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
    builder.write_u8(DataType::Int32.into());
    builder.write_u8(opcode);

    write_function_occurrence(builder, function.index, instr_pos)?;
    Ok(())
}

fn build_callvar(builder: &mut DataBuilder, opcode: u8, argument_count: u16) {
    builder.write_u16(argument_count);
    builder.write_u8(DataType::Variable.into());
    builder.write_u8(opcode);
}

fn build_extended16(builder: &mut DataBuilder, extended_kind: i16) {
    builder.write_i16(extended_kind);
    builder.write_u8(DataType::Int16.into());
    builder.write_u8(opcodes::EXTENDED);
}

fn build_pushref(builder: &mut DataBuilder, asset_reference: AssetReference) -> Result<()> {
    builder.write_i16(opcodes::extended::PUSHREF);
    builder.write_u8(DataType::Int32.into());
    builder.write_u8(opcodes::EXTENDED);
    asset_reference.serialize(builder)
}

impl GMElement for AssetReference {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if let Some(func) = reader.function_occurrences.get(&reader.cur_pos) {
            reader.cur_pos += 4; // Consume next occurrence offset
            return Ok(Self::Function(*func));
        }

        let raw = reader.read_u32()?;
        if reader.general_info.is_version_at_least((2024, 4)) {
            Self::parse(raw)
        } else {
            Self::parse_old(raw)
        }
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if let Self::Function(func_ref) = self {
            write_function_occurrence(builder, func_ref.index, builder.len())?;
            return Ok(());
        }

        let raw = if builder.is_version_at_least((2024, 4)) {
            self.build()
        } else {
            self.build_old()
        };

        builder.write_u32(raw);
        Ok(())
    }
}

fn read_variable(reader: &mut DataReader, raw_instance_type: i16) -> Result<CodeVariable> {
    let occurrence_position: u32 = reader.cur_pos;
    let raw_value = reader.read_u32()?;

    let (variable, vari_instance_type): (GMRef<GMVariable>, InstanceType) = *reader
        .variable_occurrences
        .get(&occurrence_position)
        .ok_or_else(|| {
            format!("Could not find variable with occurrence position {occurrence_position}")
        })?;

    let variable_type = (raw_value >> 24) & 0xF8;
    let variable_type: VariableType =
        num_enum_from(variable_type as u8).context("parsing variable reference chain")?;

    let instance_type: InstanceType =
        if matches!(variable_type, VariableType::Normal | VariableType::Instance) {
            InstanceType::parse(raw_instance_type, variable_type)?
        } else {
            vari_instance_type
        };

    Ok(CodeVariable {
        variable,
        variable_type,
        instance_type,
        is_int32: false,
    })
}

fn write_variable_occurrence(
    builder: &mut DataBuilder,
    gm_index: u32,
    occurrence_pos: usize,
    variable_type: VariableType,
) -> Result<()> {
    let len: usize = builder.variable_occurrences.len();
    let occurrences: &mut Vec<(usize, VariableType)> = builder
        .variable_occurrences
        .get_mut(gm_index as usize)
        .ok_or_else(|| {
            format!("Invalid Variable GMRef while writing occurrence: {gm_index} >= {len}")
        })?;

    if let Some(&(last_occurrence_pos, old_variable_type)) = occurrences.last() {
        // Replace last occurrence with next occurrence offset
        let occurrence_offset: i32 = occurrence_pos as i32 - last_occurrence_pos as i32;
        let occurrence_offset_full: i32 =
            occurrence_offset & 0x07FF_FFFF | (i32::from(u8::from(old_variable_type) & 0xF8) << 24);
        builder.overwrite_i32(occurrence_offset_full, last_occurrence_pos + 4)?;
    }

    // See write_function_occurrence
    builder.write_u32(u32::from(u8::from(variable_type) & 0xF8) << 24);

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

    // Technically it should write the name string id here.
    // Since i no longer store string ids though, this is impossible.
    // It doesn't seem to be an issue though, this value is probably unused by the runner anyway.
    builder.write_u32(0);

    builder
        .function_occurrences
        .get_mut(gm_index as usize)
        .unwrap()
        .push(occurrence_pos);
    Ok(())
}

/// Check whether this data file was generated with `YYC` (`YoYoGames Compiler`).
/// Should that be the case, the `CODE`, `VARI` and `FUNC` chunks will be empty
/// (or not exist, depending on the WAD version).
/// NOTE: YYC is untested. Issues may occur.
pub(crate) fn check_yyc(reader: &DataReader) -> Result<bool> {
    // If the CODE chunk doesn't exist; the data file was compiled with YYC.
    let Some(code) = reader.chunks.get("CODE") else {
        if reader.chunks.contains("VARI") {
            bail!("Chunk VARI exists but CODE doesn't");
        }

        if reader.chunks.contains("FUNC") {
            bail!("Chunk FUNC exists but CODE and VARI don't");
        }

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
    // the data file was compiled with YYC before WAD 17.
    if !code.is_empty() {
        return Ok(false);
    }

    if reader.general_info.wad_version > 16 {
        log::warn!("Empty, but existent CODE chunk after WAD 16");
    }

    if !vari.is_empty() {
        bail!("Chunk CODE is empty but VARI is not");
    }

    if !func.is_empty() {
        bail!("Chunk CODE and VARI are empty but FUNC is not");
    }

    Ok(true)
}
