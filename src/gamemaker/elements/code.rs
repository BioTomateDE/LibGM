use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::elements::variables::GMVariable;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::elements::animation_curves::GMAnimationCurve;
use crate::gamemaker::elements::backgrounds::GMBackground;
use crate::gamemaker::elements::fonts::GMFont;
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::elements::particles::GMParticleSystem;
use crate::gamemaker::elements::paths::GMPath;
use crate::gamemaker::elements::rooms::GMRoom;
use crate::gamemaker::elements::scripts::GMScript;
use crate::gamemaker::elements::sequence::GMSequence;
use crate::gamemaker::elements::shaders::GMShader;
use crate::gamemaker::elements::sounds::GMSound;
use crate::gamemaker::elements::sprites::GMSprite;
use crate::gamemaker::elements::timelines::GMTimeline;
use crate::gamemaker::serialize::DataBuilder;
use crate::utility::num_enum_from;

#[derive(Debug, Clone)]
pub struct GMCodes {
    pub codes: Vec<GMCode>,
    pub exists: bool,
}
impl GMChunkElement for GMCodes {
    fn empty() -> Self {
        Self { codes: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMCodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        if reader.get_chunk_length() == 0 {
            return Ok(Self { codes: vec![], exists: false })
        }
        
        let pointers: Vec<usize> = reader.read_simple_list()?;
        reader.cur_pos = match pointers.first() {
            Some(ptr) => *ptr,
            None => return Ok(Self { codes: vec![], exists: true })
        };
        let count: usize = pointers.len();
        let mut codes: Vec<GMCode> = Vec::with_capacity(count);
        let mut instructions_ranges: Vec<(usize, usize)> = Vec::with_capacity(count);
        let mut codes_by_pos: HashMap<usize, GMRef<GMCode>> = HashMap::new();
        let mut last_pos: usize = reader.cur_pos;
        
        for pointer in pointers {
            reader.assert_pos(pointer, "Code")?;
            let name: GMRef<String> = reader.read_gm_string()?;
            let code_length: usize = reader.read_usize()?;

            let instructions_start_pos: usize;
            let instructions_end_pos: usize;
            let bytecode15_info: Option<GMCodeBytecode15>;

            if reader.general_info.bytecode_version <= 14 {
                instructions_start_pos = reader.cur_pos;    // instructions are placed immediately after code metadata; how convenient!
                reader.cur_pos += code_length;  // skip over them; they will get parsed in the next loop
                instructions_end_pos = reader.cur_pos;
                bytecode15_info = None;
            } else {
                let locals_count: u16 = reader.read_u16()?;
                let arguments_count_raw: u16 = reader.read_u16()?;
                let arguments_count: u16 = arguments_count_raw & 0x7FFF;
                let weird_local_flag: bool = arguments_count_raw & 0x8000 != 0;

                let instructions_start_offset: i32 = reader.read_i32()?;
                instructions_start_pos = (instructions_start_offset + reader.cur_pos as i32 - 4) as usize;

                let offset: u32 = reader.read_u32()?;
                let b15_info = GMCodeBytecode15 { locals_count, arguments_count, weird_local_flag, offset, parent: None };
                instructions_end_pos = instructions_start_pos + code_length;
                bytecode15_info = Some(b15_info);
            };
            
            codes.push(GMCode { name, instructions: vec![], bytecode15_info });
            instructions_ranges.push((instructions_start_pos, instructions_end_pos));
            last_pos = reader.cur_pos;
        }
        
        for (i, (start, end)) in instructions_ranges.into_iter().enumerate() {
            let code: &mut GMCode = &mut codes[i];
            let length: usize = end - start;

            // if bytecode15+ and the instructions pointer is known, then it's a child code entry
            if length > 0 {
                if let Some(parent_code) = codes_by_pos.get(&start) {
                    if let Some(ref mut b15_info) = code.bytecode15_info {
                        b15_info.parent = Some(parent_code.clone());
                        continue;
                    }
                }
            }

            reader.cur_pos = start;
            code.instructions = Vec::with_capacity(length / 6);  // estimate; data is from deltarune 1.00

            while reader.cur_pos < end {
                let instruction = GMInstruction::deserialize(reader).map_err(|e| format!(
                    "{e}\n↳ for Instruction #{} (at absolute position {}) of Code entry \"{}\" with absolute start position {}",
                    code.instructions.len(), reader.cur_pos, reader.display_gm_str(code.name), start,
                ))?;
                code.instructions.push(instruction);
            }
            code.instructions.shrink_to_fit();

            if length > 0 {
                // Update information to mark this entry as the root (if we have at least 1 instruction)
                codes_by_pos.insert(start, GMRef::new(i as u32));
            }
        }
        
        reader.cur_pos = last_pos;  // has to be chunk end (since instructions are stored separately in b15+)
        Ok(GMCodes { codes, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }

        builder.write_usize(self.codes.len())?;
        let pointer_list_pos: usize = builder.len();
        for _ in 0..self.codes.len() {
            builder.write_u32(0xDEADC0DE);
        }
        
        // bytecode 14 my beloved
        if builder.bytecode_version() <= 14 {
            for (i, code) in self.codes.iter().enumerate() {
                builder.overwrite_usize(builder.len(), pointer_list_pos + 4*i)?;
                builder.write_gm_string(&code.name)?;
                let length_placeholder_pos: usize = builder.len();
                builder.write_u32(0xDEADC0DE);
                let start: usize = builder.len();

                // in bytecode 14, instructions are written immediately
                for (i, instruction) in code.instructions.iter().enumerate() {
                    instruction.serialize(builder).map_err(|e| format!(
                        "{e}\n↳ while building bytecode14 code #{i} with name \"{}\"",
                        builder.display_gm_str(&code.name),
                    ))?;
                }

                let code_length: usize = builder.len() - start;
                builder.overwrite_usize(code_length, length_placeholder_pos)?;
            }
            return Ok(())
        }
        
        // in bytecode 15, the codes' instructions are written before the codes metadata
        let mut instructions_ranges: Vec<(usize, usize)> = Vec::with_capacity(self.codes.len());

        for (i, code) in self.codes.iter().enumerate() {
            if code.bytecode15_info.as_ref().unwrap().parent.is_some() {
                // If this is a child code entry, don't write instructions; just repeat last pointer
                instructions_ranges.push(instructions_ranges.last().unwrap().clone());
                // ^ this unwrap will fail if the first entry is a child entry (which is invalid anyway)
                continue
            }
            
            let start: usize = builder.len();
            for instruction in &code.instructions {
                instruction.serialize(builder).map_err(|e| format!(
                    "{e}\n↳ while serializing bytecode15 code #{i} with name \"{}\"",
                    builder.display_gm_str(&code.name),
                ))?;
            }
            let end: usize = builder.len();
            instructions_ranges.push((start, end));
        }

        for (i, code) in self.codes.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_pos + 4*i)?;
            let (start, end) = instructions_ranges[i];
            let length: usize = end - start;
            let b15_info: &GMCodeBytecode15 = code.bytecode15_info.as_ref()
                .ok_or_else(|| format!("Code bytecode 15 data not set in Bytecode version {}", builder.bytecode_version()))?;
            
            builder.write_gm_string(&code.name)?;
            builder.write_usize(length)?;
            builder.write_u16(b15_info.locals_count);
            builder.write_u16(b15_info.arguments_count | if b15_info.weird_local_flag {0x8000} else {0});
            let instructions_start_offset: i32 = start as i32 - builder.len() as i32;
            builder.write_i32(instructions_start_offset);
            builder.write_u32(b15_info.offset);
        }
        
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCode {
    pub name: GMRef<String>,
    pub instructions: Vec<GMInstruction>,
    pub bytecode15_info: Option<GMCodeBytecode15>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeBytecode15 {
    /// The amount of local variables this code entry has.
    pub locals_count: u16,
    /// The amount of arguments this code entry accepts.
    pub arguments_count: u16,
    /// A flag set on certain code entries, which usually don't have locals attached to them.
    pub weird_local_flag: bool,
    /// Offset, **in bytes**, where code should begin executing from within the bytecode of this code entry.
    /// Should be 0 for root-level (parent) code entries, and nonzero for child code entries.
    pub offset: u32,
    /// Parent entry of this code entry, if this is a child entry; [`None`] otherwise.
    pub parent: Option<GMRef<GMCode>>,
}


// #[derive(Debug, Clone, PartialEq)]
// pub enum GMInstructionData {
//     Empty,
//     SingleType(GMSingleTypeInstruction),
//     Duplicate(GMDuplicateInstruction),
//     DuplicateSwap(GMDuplicateSwapInstruction),
//     CallVariable(GMCallVariableInstruction),
//     DoubleType(GMDoubleTypeInstruction),
//     Comparison(GMComparisonInstruction),
//     Goto(GMGotoInstruction),
//     Pop(GMPopInstruction),
//     PopSwap(GMPopSwapInstruction),
//     Push(GMPushInstruction),
//     Call(GMCallInstruction),
//     Extended16(GMExtendedInstruction16),
//     Extended32(GMExtendedInstruction32),
//     ExtendedFunc(GMExtendedInstructionFunction),
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct GMInstruction {
//     pub opcode: GMOpcode,
//     pub kind: GMInstructionData,
// }



mod kinds {
    pub const CONV: u8 = 0x07;
    pub const MUL: u8 = 0x08;
    pub const DIV: u8 = 0x09;
    pub const REM: u8 = 0x0A;
    pub const MOD: u8 = 0x0B;
    pub const ADD: u8 = 0x0C;
    pub const SUB: u8 = 0x0D;
    pub const AND: u8 = 0x0E;
    pub const OR: u8 = 0x0F;
    pub const XOR: u8 = 0x10;
    pub const NEG: u8 = 0x11;
    pub const NOT: u8 = 0x12;
    pub const SHL: u8 = 0x13;
    pub const SHR: u8 = 0x14;
    pub const CMP: u8 = 0x15;
    pub const POP: u8 = 0x45;
    pub const DUP: u8 = 0x86;
    pub const RET: u8 = 0x9C;
    pub const EXIT: u8 = 0x9D;
    pub const POPZ: u8 = 0x9E;
    pub const JMP: u8 = 0xB6;
    pub const JT: u8 = 0xB7;
    pub const JF: u8 = 0xB8;
    pub const PUSHENV: u8 = 0xBA;
    pub const POPENV: u8 = 0xBB;
    pub const PUSH: u8 = 0xC0;
    pub const PUSHLOC: u8 = 0xC1;
    pub const PUSHGLB: u8 = 0xC2;
    pub const PUSHBLTN: u8 = 0xC3;
    pub const PUSHIM: u8 = 0x84;
    pub const CALL: u8 = 0xD9;
    pub const CALLVAR: u8 = 0x99;
    pub const EXTENDED: u8 = 0xFF;
}



#[derive(Debug, Clone, PartialEq)]
pub enum GMInstruction {
    /// Converts the top of the stack from one type to another.
    Convert(GMDoubleTypeInstruction),

    /// Pops two values from the stack, multiplies them, and pushes the result.
    Multiply(GMDoubleTypeInstruction),

    /// Pops two values from the stack, divides them, and pushes the result.
    /// The second popped value is divided by the first popped value.
    Divide(GMDoubleTypeInstruction),

    /// Pops two values from the stack, performs a GML `div` operation (division with remainder), and pushes the result.
    /// The second popped value is divided (with remainder) by the first popped value.
    Remainder(GMDoubleTypeInstruction),

    /// Pops two values from the stack, performs a GML `mod` operation (`%`), and pushes the result.
    /// The second popped value is modulo'd against the first popped value.
    Modulus(GMDoubleTypeInstruction),

    /// Pops two values from the stack, adds them, and pushes the result.
    Add(GMDoubleTypeInstruction),

    /// Pops two values from the stack, **subtracts** them, and pushes the result.
    /// The second popped value is subtracted by the first popped value.
    Subtract(GMDoubleTypeInstruction),

    /// Pops two values from the stack, performs an **AND** operation, and pushes the result.
    /// This can be done bitwise or logically.
    And(GMDoubleTypeInstruction),

    /// Pops two values from the stack, performs an **OR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Or(GMDoubleTypeInstruction),

    /// Pops two values from the stack, performs an **XOR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Xor(GMDoubleTypeInstruction),

    /// Negates the top value of the stack (as in, multiplies it with negative one).
    Negate(GMSingleTypeInstruction),

    /// Performs a boolean or bitwise NOT operation on the top value of the stack (modifying it).
    Not(GMSingleTypeInstruction),

    /// Pops two values from the stack, performs a bitwise left shift operation (`<<`), and pushes the result.
    /// The second popped value is shifted left by the first popped value.
    ShiftLeft(GMDoubleTypeInstruction),

    /// Pops two values from the stack, performs a bitwise right shift operation (`>>`), and pushes the result.
    /// The second popped value is shifted right by the first popped value.
    ShiftRight(GMDoubleTypeInstruction),

    /// Pops two values from the stack, compares them using a [`GMComparisonType`], and pushes a boolean result.
    Compare(GMComparisonInstruction),

    /// Pops a value from the stack, and generally stores it in a variable, array, or otherwise.
    /// Has an alternate mode that can swap values around on the stack.
    Pop(GMPopInstruction),

    /// no idea how this works
    PopSwap(GMPopSwapInstruction),

    /// Duplicates values on the stack, or swaps them around ("dup swap" mode).
    /// Behavior depends on instruction parameters, both in data sizes and mode.
    Duplicate(GMDuplicateInstruction),

    /// no idea how this works
    DuplicateSwap(GMDuplicateSwapInstruction),

    /// Pops a value from the stack, and returns from the current function/script with that value as the return value.
    Return(GMSingleTypeInstruction),

    /// Returns from the current function/script/event with no return value.
    Exit(GMEmptyInstruction),

    /// Pops a value from the stack, and discards it.
    PopDiscard(GMSingleTypeInstruction),

    /// Branches (jumps) to another instruction in the code entry.
    Branch(GMGotoInstruction),

    /// Pops a boolean/int32 value from the stack. If true/nonzero, branches (jumps) to another instruction in the code entry.
    BranchIf(GMGotoInstruction),

    /// Pops a boolean/int32 value from the stack. If false/zero, branches (jumps) to another instruction in the code entry.
    BranchUnless(GMGotoInstruction),

    /// Pushes a `with` context, used for GML `with` statements, to the VM environment/self instance stack.
    PushWithContext(GMGotoInstruction),

    /// Pops/ends a `with` context, used for GML `with` statements, from the VM environment/self instance stack.
    /// This instruction will branch to its encoded address until no longer iterating instances, where the context will finally be gone for good.
    /// If a flag is encoded in this instruction, then this will always terminate the loop, and branch to the encoded address.
    PopWithContext(GMGotoInstruction),

    /// Pushes a constant value onto the stack. Can vary in size depending on value type.
    Push(GMPushInstruction),

    /// Pushes a value stored in a local variable onto the stack.
    PushLocal(GMPushInstruction),

    /// Pushes a value stored in a global variable onto the stack.
    PushGlobal(GMPushInstruction),

    /// Pushes a value stored in a GameMaker builtin variable onto the stack.
    PushBuiltin(GMPushInstruction),

    /// Pushes an immediate signed 32-bit integer value onto the stack, encoded as a signed 16-bit integer.
    PushImmediate(GMPushInstruction),

    /// Calls a GML script/function, using its ID. Arguments are prepared prior to this instruction, in reverse order.
    /// Argument count is encoded in this instruction. Arguments are popped off of the stack.
    Call(GMCallInstruction),

    /// Pops two values off of the stack, and then calls a GML script/function using those values, representing
    /// the "self" instance to be used when calling, as well as the reference to the function being called.
    /// Arguments are dealt with identically to "call".
    CallVariable(GMCallVariableInstruction),


    /// Verifies an array index is within proper bounds, typically for multidimensional arrays.
    CheckArrayIndex,

    /// Pops two values from the stack, those being an index and an array reference.
    /// Then, pushes the value stored at the passed-in array at the desired index.
    /// That is, this is used only with multidimensional arrays, for the final/last index operation.
    PushArrayFinal,

    /// Pops three values from the stack, those being an index, an array reference, and a value.
    /// Then, assigns the value to the array at the specified index.
    PopArrayFinal,

    /// Pops two values from the stack, those being an array reference and an index.
    /// Then, pushes a new array reference from the passed-in array at the desired index,
    /// with the expectation that it will be further indexed into.
    /// That is, this is used only with multidimensional arrays,
    /// for all index operations from the second through the second to last.
    PushArrayContainer,

    /// Sets a global variable in the VM (popped from stack), designated for
    /// tracking the now-deprecated array copy-on-write functionality in GML.
    /// The value used is specific to certain locations in scripts.
    /// When array copy-on-write functionality is disabled, this extended opcode is not used.
    SetArrayOwner,

    /// Pushes a boolean value to the stack, indicating whether static initialization
    /// has already occurred for this function (true), or otherwise false.
    HasStaticInitialized,

    /// Marks the current function to no longer be able to enter its own static initialization.
    /// This can either occur at the beginning or end of a static block,
    /// depending on whether "AllowReentrantStatic" is enabled by a game's developer
    /// (enabled by default before GameMaker 2024.11; disabled by default otherwise).
    SetStaticInitialized,

    /// Keeps track of an array reference temporarily. Used in multidimensional array compound assignment statements.
    /// Presumed to be used for garbage collection purposes.
    SaveArrayReference,

    /// Restores a previously-tracked array reference.
    /// Used in multidimensional array compound assignment statements.
    /// Presumed to be used for garbage collection purposes.
    RestoreArrayReference,

    /// Pops a value from the stack, and pushes a boolean result.
    /// The result is true if a "nullish" value, such as undefined or GML's pointer_null.
    IsNullishValue,

    /// Pushes an asset reference to the stack, encoded in an integer. Includes asset type and index.
    PushReference(GMAssetReference),
}

impl GMElement for GMInstruction {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let mut bytes: (u8, u8, u8) = (
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        );

        let mut opcode: u8 = reader.read_u8()?;
        if reader.general_info.bytecode_version < 15 {
            if matches!(opcode, 0x10..=0x16) {
                // this is needed to preserve the comparison type for pre bytecode 15
                bytes.1 = opcode - 0x10;
            }
            opcode = opcode_old_to_new(opcode);
        }

        // log::debug!("{} // {:02X} {:02X} {:02X} {:02X}", reader.cur_pos-4, bytes.0, bytes.1, bytes.2, opcode);
        
        Ok(match opcode {
            kinds::CONV => Self::Convert(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::MUL => Self::Multiply(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::DIV => Self::Divide(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::REM => Self::Remainder(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::MOD => Self::Modulus(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::ADD => Self::Add(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::SUB => Self::Subtract(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::AND => Self::And(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::OR => Self::Or(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::XOR => Self::Xor(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::NEG => Self::Negate(GMSingleTypeInstruction::parse(reader, bytes)?),
            kinds::NOT => Self::Not(GMSingleTypeInstruction::parse(reader, bytes)?),
            kinds::SHL => Self::ShiftLeft(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::SHR => Self::ShiftRight(GMDoubleTypeInstruction::parse(reader, bytes)?),
            kinds::CMP => Self::Compare(GMComparisonInstruction::parse(reader, bytes)?),
            kinds::POP if bytes.2 == 0x0F => Self::PopSwap(GMPopSwapInstruction::parse(reader, bytes)?),
            kinds::POP => Self::Pop(GMPopInstruction::parse(reader, bytes)?),
            kinds::DUP if bytes.1 == 0 => Self::Duplicate(GMDuplicateInstruction::parse(reader, bytes)?),
            kinds::DUP => Self::DuplicateSwap(GMDuplicateSwapInstruction::parse(reader, bytes)?),
            kinds::RET => Self::Return(GMSingleTypeInstruction::parse(reader, bytes)?),
            kinds::EXIT => Self::Exit(GMEmptyInstruction::parse(reader, bytes)?),
            kinds::POPZ => Self::PopDiscard(GMSingleTypeInstruction::parse(reader, bytes)?),
            kinds::JMP => Self::Branch(GMGotoInstruction::parse(reader, bytes)?),
            kinds::JT => Self::BranchIf(GMGotoInstruction::parse(reader, bytes)?),
            kinds::JF => Self::BranchUnless(GMGotoInstruction::parse(reader, bytes)?),
            kinds::PUSHENV => Self::PushWithContext(GMGotoInstruction::parse(reader, bytes)?),
            kinds::POPENV => Self::PopWithContext(GMGotoInstruction::parse(reader, bytes)?),
            kinds::PUSH => Self::Push(GMPushInstruction::parse(reader, bytes)?),
            kinds::PUSHLOC => Self::PushLocal(GMPushInstruction::parse(reader, bytes)?),
            kinds::PUSHGLB => Self::PushGlobal(GMPushInstruction::parse(reader, bytes)?),
            kinds::PUSHBLTN => Self::PushBuiltin(GMPushInstruction::parse(reader, bytes)?),
            kinds::PUSHIM => Self::PushImmediate(GMPushInstruction::parse(reader, bytes)?),
            kinds::CALL => Self::Call(GMCallInstruction::parse(reader, bytes)?),
            kinds::CALLVAR => Self::CallVariable(GMCallVariableInstruction::parse(reader, bytes)?),
            kinds::EXTENDED => {
                let data_type: GMDataType = num_enum_from(bytes.2 & 0xf)?;
                let kind: i16 = bytes.0 as i16 | ((bytes.1 as i16) << 8);
                match data_type {
                    GMDataType::Int16 => {
                        match kind {
                            -1 => Self::CheckArrayIndex,
                            -2 => Self::PushArrayFinal,
                            -3 => Self::PopArrayFinal,
                            -4 => Self::PushArrayContainer,
                            -5 => Self::SetArrayOwner,
                            -6 => Self::HasStaticInitialized,
                            -7 => Self::SetStaticInitialized,
                            -8 => Self::SaveArrayReference,
                            -9 => Self::RestoreArrayReference,
                            -10 => Self::IsNullishValue,
                            _ => return Err(format!("Invalid Int16 Extended instruction kind {kind}"))
                        }
                    }
                    GMDataType::Int32 => {
                        if kind != - 11 {
                            return Err(format!("Expected PushReference (-11) for Int32 Extended instruction, found {kind}"))
                        }
                        Self::PushReference(GMAssetReference::deserialize(reader)?)
                    }
                    _ => return Err(format!("Invalid data type for Extended instruction: {data_type:?}"))
                }
            }
            // kinds::EXTENDED if bytes.2 == 2 => {   // Int32 data type
            //     if reader.function_occurrence_map.contains_key(&reader.cur_pos) {
            //         Self::ExtendedFunc(GMExtendedInstructionFunc::parse(reader, bytes)?)
            //     } else {
            //         Self::Extended32(GMExtendedInstruction32::parse(reader, bytes)?)
            //     }
            // }
            // kinds::EXTENDED => Self::Extended16(GMExtendedInstruction16::parse(reader, bytes)?),
            _ => return Err(format!("Invalid opcode {opcode} (0x{opcode:02X})"))
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let mut opcode: u8 = opcode_from_instruction(self);
        if builder.bytecode_version() < 15 {
            opcode = opcode_new_to_old(opcode);
        }
        match self {
            Self::Convert(instr) => instr.build(builder, opcode)?,
            Self::Multiply(instr) => instr.build(builder, opcode)?,
            Self::Divide(instr) => instr.build(builder, opcode)?,
            Self::Remainder(instr) => instr.build(builder, opcode)?,
            Self::Modulus(instr) => instr.build(builder, opcode)?,
            Self::Add(instr) => instr.build(builder, opcode)?,
            Self::Subtract(instr) => instr.build(builder, opcode)?,
            Self::And(instr) => instr.build(builder, opcode)?,
            Self::Or(instr) => instr.build(builder, opcode)?,
            Self::Xor(instr) => instr.build(builder, opcode)?,
            Self::Negate(instr) => instr.build(builder, opcode)?,
            Self::Not(instr) => instr.build(builder, opcode)?,
            Self::ShiftLeft(instr) => instr.build(builder, opcode)?,
            Self::ShiftRight(instr) => instr.build(builder, opcode)?,
            Self::Compare(instr) => instr.build(builder, opcode)?,
            Self::Pop(instr) => instr.build(builder, opcode)?,
            Self::PopSwap(instr) => instr.build(builder, opcode)?,
            Self::Duplicate(instr) => instr.build(builder, opcode)?,
            Self::DuplicateSwap(instr) => instr.build(builder, opcode)?,
            Self::Return(instr) => instr.build(builder, opcode)?,
            Self::Exit(instr) => instr.build(builder, opcode)?,
            Self::PopDiscard(instr) => instr.build(builder, opcode)?,
            Self::Branch(instr) => instr.build(builder, opcode)?,
            Self::BranchIf(instr) => instr.build(builder, opcode)?,
            Self::BranchUnless(instr) => instr.build(builder, opcode)?,
            Self::PushWithContext(instr) => instr.build(builder, opcode)?,
            Self::PopWithContext(instr) => instr.build(builder, opcode)?,
            Self::Push(instr) => instr.build(builder, opcode)?,
            Self::PushLocal(instr) => instr.build(builder, opcode)?,
            Self::PushGlobal(instr) => instr.build(builder, opcode)?,
            Self::PushBuiltin(instr) => instr.build(builder, opcode)?,
            Self::PushImmediate(instr) => instr.build(builder, opcode)?,
            Self::Call(instr) => instr.build(builder, opcode)?,
            Self::CallVariable(instr) => instr.build(builder, opcode)?,
            Self::CheckArrayIndex => build_extended16(builder, -1),
            Self::PushArrayFinal => build_extended16(builder, -2),
            Self::PopArrayFinal => build_extended16(builder, -3),
            Self::PushArrayContainer => build_extended16(builder, -4),
            Self::SetArrayOwner => build_extended16(builder, -5),
            Self::HasStaticInitialized => build_extended16(builder, -6),
            Self::SetStaticInitialized  => build_extended16(builder, -7),
            Self::SaveArrayReference => build_extended16(builder, -8),
            Self::RestoreArrayReference => build_extended16(builder, -9),
            Self::IsNullishValue => build_extended16(builder, -10),
            Self::PushReference(asset_ref) => {
                builder.write_i16(-11);
                builder.write_u8(GMDataType::Int32.into());
                builder.write_u8(kinds::EXTENDED.into());
                asset_ref.serialize(builder)?
            },
        }
        Ok(())
    }
}


fn build_extended16(builder: &mut DataBuilder, kind: i16) {
    builder.write_i16(kind);
    builder.write_u8(GMDataType::Int16.into());
    builder.write_u8(kinds::EXTENDED.into());
}


trait InstructionData {
    fn parse(reader: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> where Self: Sized;
    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String>;
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMEmptyInstruction;
impl InstructionData for GMEmptyInstruction {
    fn parse(_: &mut DataReader, _: (u8, u8, u8)) -> Result<Self, String> {
        Ok(Self)
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_i24(0);
        builder.write_u8(opcode);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMSingleTypeInstruction {
    pub data_type: GMDataType,
}
impl InstructionData for GMSingleTypeInstruction {
    fn parse(_: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let data_type: GMDataType = num_enum_from(b.2 & 0xf)?;

        // Ensure basic conditions hold
        if b.0 != 0 {
            return Err(format!("Invalid padding {:02X} while parsing Single Type Instruction", b.0));
        }
        if b.2 >> 4 != 0 {
            return Err(format!("Second type should be zero but is {0} (0x{0:02X}) for Single Type Instruction", b.2 >> 4))
        }

        Ok(Self { data_type })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_u16(0);
        builder.write_u8(self.data_type.into());
        builder.write_u8(opcode);
        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMCallVariableInstruction {
    pub data_type: GMDataType,
    pub argument_count: u8,
}
impl InstructionData for GMCallVariableInstruction {
    fn parse(_: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let data_type: GMDataType = num_enum_from(b.2 & 0xf)?;
        Ok(Self { data_type, argument_count: b.1 })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_u8(0);
        builder.write_u8(self.argument_count);
        builder.write_u8(self.data_type.into());
        builder.write_u8(opcode);
        Ok(())
    }
}



#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDuplicateInstruction {
    pub data_type: GMDataType,
    pub size: u8,
}
impl InstructionData for GMDuplicateInstruction {
    fn parse(_: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let data_type: GMDataType = num_enum_from(b.2 & 0xf)?;
        Ok(Self { data_type, size: b.0 })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_u8(self.size);
        builder.write_u8(0);
        builder.write_u8(self.data_type.into());
        builder.write_u8(opcode);
        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDuplicateSwapInstruction {
    pub data_type: GMDataType,
    pub size1: u8,
    pub size2: u8,
}
impl InstructionData for GMDuplicateSwapInstruction {
    fn parse(_: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let data_type: GMDataType = num_enum_from(b.2 & 0xf)?;
        Ok(Self { data_type, size1: b.0, size2: b.1 })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_u8(self.size1);
        builder.write_u8(self.size2);
        builder.write_u8(self.data_type.into());
        builder.write_u8(opcode);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMPopSwapInstruction {
    pub size: u8,
}
impl InstructionData for GMPopSwapInstruction {
    fn parse(_: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        Ok(Self { size: b.0 })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_u8(self.size);
        builder.write_u8(0);
        builder.write_u8(GMDataType::Int16.into());
        builder.write_u8(opcode);
        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMGotoInstruction {
    /// Contains the offset of where to jump. 1 = 4 bytes. Can be negative.
    /// If [`None`], it is a "popenv exit magic" goto instruction.
    pub jump_offset: Option<i32>,
}
impl InstructionData for GMGotoInstruction {
    fn parse(reader: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        if reader.general_info.bytecode_version <= 14 {
            let jump_offset: i32 = b.0 as i32 | ((b.1 as u32) << 8) as i32 | ((b.2 as i32) << 16);
            if jump_offset == -1048576 {   // little endian [00 00 F0]
                return Ok(Self { jump_offset: None })
            }
            return Ok(Self { jump_offset: Some(jump_offset) })
        }

        let v: u32 = b.0 as u32 | ((b.1 as u32) << 8) | ((b.2 as u32) << 16);      // i hate bitshifting
        let popenv_exit_magic: bool = (v & 0x800000) != 0;
        if popenv_exit_magic && v != 0xF00000 {
            return Err("Popenv exit magic doesn't work while parsing Goto Instruction".to_string());
        }
        // "The rest is int23 signed value, so make sure" (<-- idk what this is supposed to mean)
        let mut jump_offset: u32 = v & 0x003FFFFF;
        if (v & 0x00C00000) != 0 {
            jump_offset |= 0xFFC00000;
        }
        if popenv_exit_magic {
            Ok(Self { jump_offset: None })
        } else {
            Ok(Self { jump_offset: Some(jump_offset as i32) })
        }
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        if let Some(jump_offset) = self.jump_offset {
            builder.write_i24(jump_offset & 0x7fffff);
        } else {
            // popenv exit magic
            builder.write_i24(0xF00000);
        }
        builder.write_u8(opcode);
        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDoubleTypeInstruction {
    pub right: GMDataType,
    pub left: GMDataType,
}
impl InstructionData for GMDoubleTypeInstruction {
    fn parse(_: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let right: GMDataType = num_enum_from(b.2 & 0xf)?;
        let left: GMDataType = num_enum_from(b.2 >> 4)?;
        if b.1 != 0 {    // might be incorrect; remove if issues
            return Err(format!("b1 should be zero but is {} for Double Type Instruction", b.1))
        }
        Ok(Self { right, left })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_u8(0);
        builder.write_u8(0);
        builder.write_u8(u8::from(self.right) | u8::from(self.left) << 4);
        builder.write_u8(opcode);
        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMComparisonInstruction {
    pub comparison_type: GMComparisonType,
    pub type1: GMDataType,
    pub type2: GMDataType,
}
impl InstructionData for GMComparisonInstruction {
    fn parse(_: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let comparison_type: GMComparisonType = num_enum_from(b.1)?;
        let type1: GMDataType = num_enum_from(b.2 & 0xf)?;
        let type2: GMDataType = num_enum_from(b.2 >> 4)?;
        Ok(Self { comparison_type, type1, type2 })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        builder.write_u8(0);
        builder.write_u8(self.comparison_type.into());
        builder.write_u8(u8::from(self.type1) | u8::from(self.type2) << 4);
        if builder.bytecode_version() < 15 {
            builder.write_u8(0x10 + u8::from(self.comparison_type))
        } else {
            builder.write_u8(opcode);
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMPopInstruction {
    pub type1: GMDataType,
    pub type2: GMDataType,
    pub destination: CodeVariable,
}
impl InstructionData for GMPopInstruction {
    fn parse(reader: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let type1: GMDataType = num_enum_from(b.2 & 0xf)?;
        let type2: GMDataType = num_enum_from(b.2 >> 4)?;
        let raw_instance_type: i16 = b.0 as i16 | ((b.1 as i16) << 8);
        let destination: CodeVariable = read_variable(reader, raw_instance_type)?;
        Ok(Self { type1, type2, destination })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        let instr_pos: usize = builder.len();
        builder.write_i16(build_instance_type(&self.destination.instance_type));
        builder.write_u8(u8::from(self.type1) | u8::from(self.type2) << 4);
        builder.write_u8(opcode);
        let variable: &GMVariable = self.destination.variable.resolve(&builder.gm_data.variables.variables)?;
        write_variable_occurrence(builder, self.destination.variable.index, instr_pos, variable.name.index, self.destination.variable_type)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMPushInstruction {
    pub value: GMCodeValue,
}
impl InstructionData for GMPushInstruction {
    fn parse(reader: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let data_type: GMDataType = num_enum_from(b.2)?;
        let raw_instance_type: i16 = (b.0 as i16) | ((b.1 as i16) << 8);
        let value: GMCodeValue = if data_type == GMDataType::Variable {
            GMCodeValue::Variable(read_variable(reader, raw_instance_type)?)
        } else {
            read_code_value(reader, data_type)?
        };
        Ok(Self { value })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        let instr_pos: usize = builder.len();
        builder.write_i16(match &self.value {
            GMCodeValue::Int16(int16) => *int16,      // should never happen tbh (popswap)
            GMCodeValue::Variable(variable) => build_instance_type(&variable.instance_type),
            _ => 0
        });

        let data_type: GMDataType = get_data_type_from_value(&self.value);
        builder.write_u8(data_type.into());
        builder.write_u8(opcode);

        match &self.value {
            GMCodeValue::Int16(_) => {}   // nothing because it was already written inside the instruction
            GMCodeValue::Int32(int32) => builder.write_i32(*int32),
            GMCodeValue::Int64(int64) => builder.write_i64(*int64),
            GMCodeValue::Double(double) => builder.write_f64(*double),
            GMCodeValue::Float(float) => builder.write_f32(*float),
            GMCodeValue::Boolean(boolean) => builder.write_bool32(*boolean),
            GMCodeValue::String(string_ref) => builder.write_u32(string_ref.index),
            GMCodeValue::Variable(code_variable) => {
                let variable: &GMVariable = code_variable.variable.resolve(&builder.gm_data.variables.variables)?;
                write_variable_occurrence(builder, code_variable.variable.index, instr_pos, variable.name.index, code_variable.variable_type)?;
            }
            GMCodeValue::Function(func_ref) => {
                let function: &GMFunction = func_ref.resolve(&builder.gm_data.functions.functions)?;
                write_function_occurrence(builder, func_ref.index, instr_pos, function.name.index)?;
            }
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCallInstruction {
    pub arguments_count: u8,
    pub data_type: GMDataType,
    pub function: GMRef<GMFunction>,
}
impl InstructionData for GMCallInstruction {
    fn parse(reader: &mut DataReader, b: (u8, u8, u8)) -> Result<Self, String> {
        let data_type: GMDataType = num_enum_from(b.2)?;
        let function: GMRef<GMFunction> = reader.function_occurrence_map.get(&reader.cur_pos).ok_or_else(|| format!(
            "Could not find any function with absolute occurrence position {} in map with length {} while parsing Call Instruction",
            reader.cur_pos, reader.function_occurrence_map.len(),
        ))?.clone();
        reader.cur_pos += 4;   // skip next occurrence offset

        Ok(GMCallInstruction { arguments_count: b.0, data_type, function })
    }

    fn build(&self, builder: &mut DataBuilder, opcode: u8) -> Result<(), String> {
        let instr_pos: usize = builder.len();
        builder.write_u8(self.arguments_count);
        builder.write_u8(0);
        builder.write_u8(self.data_type.into());
        builder.write_u8(opcode);

        let function: &GMFunction = self.function.resolve(&builder.gm_data.functions.functions)?;
        write_function_occurrence(builder, self.function.index, instr_pos, function.name.index)?;
        Ok(())
    }
}


fn opcode_from_instruction(instruction: &GMInstruction) -> u8 {
    match instruction {
        GMInstruction::Convert(_) => kinds::CONV,
        GMInstruction::Multiply(_) => kinds::MUL,
        GMInstruction::Divide(_) => kinds::DIV,
        GMInstruction::Remainder(_) => kinds::REM,
        GMInstruction::Modulus(_) => kinds::MOD,
        GMInstruction::Add(_) => kinds::ADD,
        GMInstruction::Subtract(_) => kinds::SUB,
        GMInstruction::And(_) => kinds::AND,
        GMInstruction::Or(_) => kinds::OR,
        GMInstruction::Xor(_) => kinds::XOR,
        GMInstruction::Negate(_) => kinds::NEG,
        GMInstruction::Not(_) => kinds::NOT,
        GMInstruction::ShiftLeft(_) => kinds::SHL,
        GMInstruction::ShiftRight(_) => kinds::SHR,
        GMInstruction::Compare(_) => kinds::CMP,
        GMInstruction::Pop(_) => kinds::POP,
        GMInstruction::PopSwap(_) => kinds::POP,
        GMInstruction::Duplicate(_) => kinds::DUP,
        GMInstruction::DuplicateSwap(_) => kinds::DUP,
        GMInstruction::Return(_) => kinds::RET,
        GMInstruction::Exit(_) => kinds::EXIT,
        GMInstruction::PopDiscard(_) => kinds::POPZ,
        GMInstruction::Branch(_) => kinds::JMP,
        GMInstruction::BranchIf(_) => kinds::JT,
        GMInstruction::BranchUnless(_) => kinds::JF,
        GMInstruction::PushWithContext(_) => kinds::PUSHENV,
        GMInstruction::PopWithContext(_) => kinds::POPENV,
        GMInstruction::Push(_) => kinds::PUSH,
        GMInstruction::PushLocal(_) => kinds::PUSHLOC,
        GMInstruction::PushGlobal(_) => kinds::PUSHGLB,
        GMInstruction::PushBuiltin(_) => kinds::PUSHBLTN,
        GMInstruction::PushImmediate(_) => kinds::PUSHIM,
        GMInstruction::Call(_) => kinds::CALL,
        GMInstruction::CallVariable(_) => kinds::CALLVAR,
        GMInstruction::CheckArrayIndex => kinds::EXTENDED,
        GMInstruction::PushArrayFinal => kinds::EXTENDED,
        GMInstruction::PopArrayFinal => kinds::EXTENDED,
        GMInstruction::PushArrayContainer => kinds::EXTENDED,
        GMInstruction::SetArrayOwner => kinds::EXTENDED,
        GMInstruction::HasStaticInitialized => kinds::EXTENDED,
        GMInstruction::SetStaticInitialized => kinds::EXTENDED,
        GMInstruction::SaveArrayReference => kinds::EXTENDED,
        GMInstruction::RestoreArrayReference => kinds::EXTENDED,
        GMInstruction::IsNullishValue => kinds::EXTENDED,
        GMInstruction::PushReference(_) => kinds::EXTENDED,
    }
}


fn opcode_old_to_new(opcode: u8) -> u8 {
    match opcode {
        0x03 => 0x07,
        0x04 => 0x08,
        0x05 => 0x09,
        0x06 => 0x0A,
        0x07 => 0x0B,
        0x08 => 0x0C,
        0x09 => 0x0D,
        0x0A => 0x0E,
        0x0B => 0x0F,
        0x0C => 0x10,
        0x0D => 0x11,
        0x0E => 0x12,
        0x0F => 0x13,
        0x10 => 0x14,
        0x11 | 0x12 | 0x13 | 0x14 | 0x16 => 0x15,
        0x41 => 0x45,
        0x82 => 0x86,
        0xB7 => 0xB6,
        0xB8 => 0xB7,
        0xB9 => 0xB8,
        0xBB => 0xBA,
        0x9D => 0x9C,
        0x9E => 0x9D,
        0x9F => 0x9E,
        0xBC => 0xBB,
        0xDA => 0xD9,
        _ => opcode
    }
}


fn opcode_new_to_old(opcode: u8) -> u8 {
    match opcode {
        0x07 => 0x03,
        0x08 => 0x04,
        0x09 => 0x05,
        0x0A => 0x06,
        0x0B => 0x07,
        0x0C => 0x08,
        0x0D => 0x09,
        0x0E => 0x0A,
        0x0F => 0x0B,
        0x10 => 0x0C,
        0x11 => 0x0D,
        0x12 => 0x0E,
        0x13 => 0x0F,
        0x14 => 0x10,
        0x15 => 0,  // should be handled by GMComparisonInstruction::build
        0x45 => 0x41,
        0x84 => 0xC0,
        0x86 => 0x82,
        0x9C => 0x9D,
        0x9D => 0x9E,
        0x9E => 0x9F,
        0xB6 => 0xB7,
        0xB7 => 0xB8,
        0xB8 => 0xB9,
        0xBA => 0xBB,
        0xBB => 0xBC,
        0xD9 => 0xDA,
        0xC1 => 0xC0,
        0xC2 => 0xC0,
        0xC3 => 0xC0,
        _ => opcode
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMAssetReference {
    Object(GMRef<GMGameObject>),
    Sprite(GMRef<GMSprite>),
    Sound(GMRef<GMSound>),
    Room(GMRef<GMRoom>),
    Background(GMRef<GMBackground>),
    Path(GMRef<GMPath>),
    Script(GMRef<GMScript>),
    Font(GMRef<GMFont>),
    Timeline(GMRef<GMTimeline>),
    Shader(GMRef<GMShader>),
    Sequence(GMRef<GMSequence>),
    AnimCurve(GMRef<GMAnimationCurve>),
    ParticleSystem(GMRef<GMParticleSystem>),
    RoomInstance(i32),
    /// Not actually in GameMaker; added by me
    Function(GMRef<GMFunction>),
}

impl GMElement for GMAssetReference {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        if let Some(func) = reader.function_occurrence_map.get(&reader.cur_pos) {
            reader.cur_pos += 4;   // consume next occurrence offset
            return Ok(Self::Function(*func))
        }

        let raw: i32 = reader.read_i32()?;
        let index: u32 = (raw & 0xFFFFFF) as u32;
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
            _ => return Err(format!("Invalid asset type {asset_type}"))
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let (index, asset_type) = match self {
            GMAssetReference::Object(gm_ref) => (gm_ref.index, 0),
            GMAssetReference::Sprite(gm_ref) => (gm_ref.index, 1),
            GMAssetReference::Sound(gm_ref) => (gm_ref.index, 2),
            GMAssetReference::Room(gm_ref) => (gm_ref.index, 3),
            GMAssetReference::Background(gm_ref) => (gm_ref.index, 4),
            GMAssetReference::Path(gm_ref) => (gm_ref.index, 5),
            GMAssetReference::Script(gm_ref) => (gm_ref.index, 6),
            GMAssetReference::Font(gm_ref) => (gm_ref.index, 7),
            GMAssetReference::Timeline(gm_ref) => (gm_ref.index, 8),
            GMAssetReference::Shader(gm_ref) => (gm_ref.index, 9),
            GMAssetReference::Sequence(gm_ref) => (gm_ref.index, 10),
            GMAssetReference::AnimCurve(gm_ref) => (gm_ref.index, 11),
            GMAssetReference::ParticleSystem(gm_ref) => (gm_ref.index, 12),
            GMAssetReference::RoomInstance(id) => (*id as u32, 13),
            GMAssetReference::Function(func_ref) => {
                let function: &GMFunction = func_ref.resolve(&builder.gm_data.functions.functions)?;
                write_function_occurrence(builder, func_ref.index, builder.len(), function.name.index)?;
                return Ok(())
            }
        };
        let raw: u32 = (asset_type << 24) | index & 0xFFFFFF;
        builder.write_u32(raw);
        Ok(())
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMDataType {
    Double,
    Float,
    Int32,
    Int64,
    Boolean,
    Variable,
    String,
    Int16 = 0x0f,
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMInstanceType {
    Undefined,
    
    /// Represents the current chunk instance.
    Self_(Option<GMRef<GMGameObject>>),
    
    /// Instance ID in the Room -100000; used when the Variable Type is [`GMVariableType::Instance`].
    /// This doesn't exist in UTMT.
    RoomInstance(i16),
    
    /// Represents the other context, which has multiple definitions based on the location used.
    Other,
    
    /// Represents all active object instances. Assignment operations can perform a loop.
    All,
    
    /// Represents no object/instance.
    None,
    
    /// Used for global variables.
    Global,
    
    /// Used for GML built-in variables.
    Builtin,
    
    /// Used for local variables; local to their code script.
    Local,
    
    /// Instance is stored in a Variable data type on the top of the stack.
    StackTop,
    
    /// Used for function argument variables in GMLv2 (GMS 2.3).
    Argument,
    
    /// Used for static variables.
    Static,
}

impl Display for GMInstanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GMInstanceType::Undefined => write!(f, "Undefined"),
            GMInstanceType::Self_(None) => write!(f, "Self"),
            GMInstanceType::Self_(Some(reference)) => write!(f, "Self<{}>", reference.index),
            GMInstanceType::RoomInstance(instance_id) => write!(f, "RoomInstanceID<{instance_id}>"),
            GMInstanceType::Other => write!(f, "Other"),
            GMInstanceType::All => write!(f, "All"),
            GMInstanceType::None => write!(f, "None"),
            GMInstanceType::Global => write!(f, "Global"),
            GMInstanceType::Builtin => write!(f, "Builtin"),
            GMInstanceType::Local => write!(f, "Local"),
            GMInstanceType::StackTop => write!(f, "StackTop"),
            GMInstanceType::Argument => write!(f, "Argument"),
            GMInstanceType::Static => write!(f, "Static"),
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMVariableType {
    /// Used for normal single-dimension array variables
    Array = 0x00,
    
    /// Used when referencing a variable on another variable, e.g. a chain referenc
    StackTop = 0x80,
    
    /// normal
    Normal = 0xA0,
    
    /// Used when referencing variables on room instance IDs, e.g. something like "inst_01ABCDEF.x" in GML
    Instance = 0xE0,
    
    /// GMS2.3+, multidimensional array with pushaf
    ArrayPushAF = 0x10,
    
    /// GMS2.3+, multidimensional array with pushaf or popaf
    ArrayPopAF = 0x90,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMComparisonType {
    /// "Less than" | `<`
    LessThan = 1,
    
    /// "Less than or equal to" | `<=`
    LessOrEqual = 2,
    
    /// "Equal to" | `==`
    Equal = 3,

    /// "Not equal to" | `!=`
    NotEqual = 4,

    /// "Greater than or equal to" | `>=`
    GreaterOrEqual = 5,

    /// "Greater than" | `>`
    GreaterThan = 6,
}


#[derive(Debug, Clone, PartialEq)]
pub struct CodeVariable {
    pub variable: GMRef<GMVariable>,
    pub variable_type: GMVariableType,
    pub instance_type: GMInstanceType,
    pub is_int32: bool,
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMCodeValue {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Double(f64),
    Float(f32),
    Boolean(bool),
    String(GMRef<String>),
    Variable(CodeVariable),
    /// Does not exist in UTMT. Added in order to support inline/anonymous functions.
    Function(GMRef<GMFunction>),
}


fn read_code_value(reader: &mut DataReader, data_type: GMDataType) -> Result<GMCodeValue, String> {
    match data_type {
        GMDataType::Double => reader.read_f64().map(GMCodeValue::Double),
        GMDataType::Float => reader.read_f32().map(GMCodeValue::Float),
        GMDataType::Int32 => {
            if let Some(&function) = reader.function_occurrence_map.get(&reader.cur_pos) {
                reader.cur_pos += 4;    // skip next occurrence offset
                return Ok(GMCodeValue::Function(function.clone()))
            }

            if let Some(&variable) = reader.variable_occurrence_map.get(&reader.cur_pos) {
                reader.cur_pos += 4;    // skip next occurrence offset
                return Ok(GMCodeValue::Variable(CodeVariable {
                    variable,
                    variable_type: GMVariableType::Normal,
                    instance_type: GMInstanceType::Undefined,
                    is_int32: true,
                }))
            }

            reader.read_i32().map(GMCodeValue::Int32)
        }
        GMDataType::Int64 => reader.read_i64().map(GMCodeValue::Int64),
        GMDataType::Boolean => reader.read_bool32().map(GMCodeValue::Boolean),
        GMDataType::String => reader.read_resource_by_id().map(GMCodeValue::String),
        GMDataType::Int16 => {
            // int16 in embedded in the instruction itself
            reader.cur_pos -= 4;
            let number: i16 = reader.read_i16()?;
            reader.cur_pos += 2;
            Ok(GMCodeValue::Int16(number))
        }
        other => Err(format!("Trying to read unsupported data type {other:?} while reading value in code at absolute position {}", reader.cur_pos)),
    }
}


fn read_variable(reader: &mut DataReader, raw_instance_type: i16) -> Result<CodeVariable, String> {
    let occurrence_position: usize = reader.cur_pos;
    let raw_value: i32 = reader.read_i32()?;

    let variable_type: i32 = (raw_value >> 24) & 0xF8;
    let variable_type: GMVariableType = num_enum_from(variable_type as u8)
        .map_err(|e| format!("{e} while parsing variable reference chain"))?;

    let instance_type: GMInstanceType = parse_instance_type(raw_instance_type, variable_type)?;

    let variable: GMRef<GMVariable> = reader.variable_occurrence_map.get(&occurrence_position)
        .ok_or_else(|| format!(
            "Could not find {} Variable with occurrence position {} in hashmap with length {} while parsing code value",
            instance_type, occurrence_position, reader.variable_occurrence_map.len(),
        ))?.clone();

    Ok(CodeVariable { variable, variable_type, instance_type, is_int32: false })
}


pub fn parse_instance_type(raw_value: i16, variable_type: GMVariableType) -> Result<GMInstanceType, String> {
    // If > 0; then game object id (or room instance id). If < 0, then variable instance type.
    if raw_value > 0 {
        return Ok(if variable_type == GMVariableType::Instance {
            GMInstanceType::RoomInstance(raw_value)
        } else {
            GMInstanceType::Self_(Some(GMRef::new(raw_value as u32)))
        });
    }

    let instance_type: GMInstanceType = match raw_value {
        0 => GMInstanceType::Undefined,
        -1 => GMInstanceType::Self_(None),
        -2 => GMInstanceType::Other,
        -3 => GMInstanceType::All,
        -4 => GMInstanceType::None,
        -5 => GMInstanceType::Global,
        -6 => GMInstanceType::Builtin,
        -7 => GMInstanceType::Local,
        -9 => GMInstanceType::StackTop,
        -15 => GMInstanceType::Argument,
        -16 => GMInstanceType::Static,
        _ => return Err(format!("Invalid instance type {raw_value} (0x{raw_value:04X})"))
    };

    Ok(instance_type)
}

pub fn build_instance_type(instance_type: &GMInstanceType) -> i16 {
    // If > 0; then game object id (or room instance id). If < 0, then variable instance type.
    match instance_type {
        GMInstanceType::Undefined => 0,
        GMInstanceType::Self_(None) => -1,
        GMInstanceType::Self_(Some(game_object_ref)) => game_object_ref.index as i16,
        GMInstanceType::RoomInstance(instance_id) => *instance_id,
        GMInstanceType::Other => -2,
        GMInstanceType::All => -3,
        GMInstanceType::None => -4,
        GMInstanceType::Global => -5,
        GMInstanceType::Builtin => -6,
        GMInstanceType::Local => -7,
        GMInstanceType::StackTop => -9,
        GMInstanceType::Argument => -15,
        GMInstanceType::Static => -16,
    }
}


fn write_variable_occurrence(
    builder: &mut DataBuilder,
    gm_index: u32,
    occurrence_position: usize,
    name_string_id: u32,
    variable_type: GMVariableType,
) -> Result<(), String> {
    let occurrence_map_len: usize = builder.variable_occurrences.len();   // prevent double borrow on error message
    let occurrences: &mut Vec<(usize, GMVariableType)> = builder.variable_occurrences.get_mut(gm_index as usize).ok_or_else(|| format!(
        "Trying to get inner variable occurrences vec out of bounds while writing occurrence: {} >= {}",
        gm_index, occurrence_map_len,
    ))?;

    if let Some((last_occurrence_pos, old_variable_type)) = occurrences.last().cloned() {
        // replace last occurrence (which is name string id) with next occurrence offset
        let occurrence_offset: i32 = occurrence_position as i32 - last_occurrence_pos as i32;
        let occurrence_offset_full: i32 = occurrence_offset & 0x07FFFFFF | (((u8::from(old_variable_type) & 0xF8) as i32) << 24);
        builder.overwrite_i32(occurrence_offset_full, last_occurrence_pos+4)?;
    }

    // write name string id for this occurrence. this is correct if it is the last occurrence.
    // otherwise, it will be overwritten later by the code above.
    // hopefully, writing the name string id instead of -1 for unused variables will be fine.
    builder.write_u32(name_string_id & 0x07FFFFFF | (((u8::from(variable_type) & 0xF8) as u32) << 24));

    // fuckass borrow checker
    builder.variable_occurrences.get_mut(gm_index as usize).unwrap().push((occurrence_position, variable_type));
    Ok(())
}


fn write_function_occurrence(builder: &mut DataBuilder, gm_index: u32, occurrence_position: usize, name_string_id: u32) -> Result<(), String> {
    let occurrence_map_len: usize = builder.function_occurrences.len();   // prevent double borrow on error message
    let occurrences: &mut Vec<usize> = builder.function_occurrences.get_mut(gm_index as usize).ok_or_else(|| format!(
        "Trying to get inner function occurrences vec out of bounds while writing occurrence: {} >= {}",
        gm_index, occurrence_map_len,
    ))?;

    if let Some(last_occurrence_pos) = occurrences.last().cloned() {
        // replace last occurrence (which is name string id) with next occurrence offset
        let occurrence_offset: i32 = occurrence_position as i32 - last_occurrence_pos as i32;
        builder.overwrite_i32(occurrence_offset & 0x07FFFFFF, last_occurrence_pos+4)?;
    }

    // write name string id for this occurrence. this is correct if it is the last occurrence.
    // otherwise, it will be overwritten later by the code above.
    builder.write_u32(name_string_id & 0x07FFFFFF);

    builder.function_occurrences.get_mut(gm_index as usize).unwrap().push(occurrence_position);
    Ok(())
}


pub fn get_data_type_from_value(code_value: &GMCodeValue) -> GMDataType {
    match code_value {
        GMCodeValue::Int16(_) => GMDataType::Int16,
        GMCodeValue::Int32(_) => GMDataType::Int32,
        GMCodeValue::Function(_) => GMDataType::Int32,  // functions are not a "real" gm type; they're always int32
        GMCodeValue::Variable(var) if var.is_int32 => GMDataType::Int32,    // idk when this happens
        GMCodeValue::Int64(_) => GMDataType::Int64,
        GMCodeValue::Float(_) => GMDataType::Float,
        GMCodeValue::Double(_) => GMDataType::Double,
        GMCodeValue::Boolean(_) => GMDataType::Boolean,
        GMCodeValue::String(_) => GMDataType::String,
        GMCodeValue::Variable(_) => GMDataType::Variable,
    }
}


pub fn get_instruction_size(instruction: &GMInstruction) -> u32 {
    match instruction {
        GMInstruction::Pop(_) => 2,
        GMInstruction::Push(instr) |
        GMInstruction::PushLocal(instr) |
        GMInstruction::PushGlobal(instr) |
        GMInstruction::PushBuiltin(instr) |
        GMInstruction::PushImmediate(instr) => match instr.value {
            GMCodeValue::Int16(_) => 1,
            GMCodeValue::Int64(_) => 3,
            GMCodeValue::Double(_) => 3,
            _ => 2,
        }
        GMInstruction::Call(_) => 2,
        GMInstruction::PushReference(_) => 2,
        _ => 1,
    }
}


/// Check whether this data file was generated with YYC (YoYoGames Compiler).
/// Should that be the case, the CODE, VARI and FUNC chunks will be empty (or not exist?).
/// NOTE: YYC is untested. Issues may occur.
pub fn check_yyc(reader: &DataReader) -> bool {
    let Some(chunk_code) = reader.chunks.get("CODE") else {return true};
    let Some(chunk_vari) = reader.chunks.get("VARI") else {return true};
    let Some(chunk_func) = reader.chunks.get("FUNC") else {return true};
    chunk_code.end_pos <= chunk_code.start_pos &&
        chunk_vari.end_pos <= chunk_vari.start_pos &&
        chunk_func.end_pos <= chunk_func.start_pos
}

