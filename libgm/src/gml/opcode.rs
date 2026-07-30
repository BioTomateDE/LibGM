// SPDX-License-Identifier: GPL-3.0-only
//! These opcodes are for WAD 15+.
//! They are different in previous versions, see [`old_to_new`] and [`new_to_old`].
//!
//! They follow the actual names used in GMAC.
use crate::gml::instruction::BinaryOp;
use crate::gml::instruction::Instruction;

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
pub const SET: u8 = 0x15; // aka CMP
pub const POP: u8 = 0x45;
pub const DUP: u8 = 0x86;
pub const RET: u8 = 0x9C;
pub const EXIT: u8 = 0x9D;
pub const POPZ: u8 = 0x9E;
pub const B: u8 = 0xB6; // aka BR or JMP
pub const BT: u8 = 0xB7;
pub const BF: u8 = 0xB8;
pub const PUSHENV: u8 = 0xBA;
pub const POPENV: u8 = 0xBB;
pub const PUSH: u8 = 0xC0;
pub const PUSHL: u8 = 0xC1; // aka PUSHLOC
pub const PUSHG: u8 = 0xC2; // aka PUSHGLB
pub const PUSHV: u8 = 0xC3; // aka PUSHBLTN
pub const PUSHI: u8 = 0x84; // aka PUSHIM
pub const CALL: u8 = 0xD9;
pub const CALLV: u8 = 0x99; // aka CALLVAR
pub const BREAK: u8 = 0xFF; // aka EXTENDED; probably originally intended as a breakpoint

impl Instruction {
    /// The normalized opcode of the instruction.
    ///
    /// Normalized means that it will always return the modern WAD15+ opcode,
    /// even if the game is in WAD < 15.
    ///
    /// Checking this implementation can be useful to get an overview of which
    /// [`Instruction`] enum variants use which opcode.
    #[must_use]
    pub const fn opcode(&self) -> u8 {
        match self {
            Self::Binary { op, .. } => opcode_binary(*op),
            Self::Convert { .. } => CONV,
            Self::Negate { .. } => NEG,
            Self::Not { .. } => NOT,
            Self::Compare { .. } => SET,
            Self::Pop { .. } | Self::PopSwap { .. } => POP,
            Self::Duplicate { .. } | Self::DuplicateSwap { .. } => DUP,
            Self::Return { .. } => RET,
            Self::Exit { .. } => EXIT,
            Self::PopDiscard { .. } => POPZ,
            Self::Branch { .. } => B,
            Self::BranchIf { .. } => BT,
            Self::BranchUnless { .. } => BF,
            Self::PushWithContext { .. } => PUSHENV,
            Self::PopWithContext { .. } | Self::PopWithContextExit { .. } => POPENV,
            Self::Push { .. } => PUSH,
            Self::PushLocal { .. } => PUSHL,
            Self::PushGlobal { .. } => PUSHG,
            Self::PushBuiltin { .. } => PUSHV,
            Self::PushImmediate { .. } => PUSHI,
            Self::Call { .. } => CALL,
            Self::CallVariable { .. } => CALLV,
            Self::CheckArrayIndex
            | Self::PushArrayFinal
            | Self::PopArrayFinal
            | Self::PushArrayContainer
            | Self::SetArrayOwner
            | Self::HasStaticInitialized
            | Self::SetStaticInitialized
            | Self::SaveArrayReference
            | Self::RestoreArrayReference
            | Self::IsNullishValue
            | Self::PushReference { .. } => BREAK,
        }
    }

    /// Get the old (WAD < 15) opcode of the instruction.
    #[must_use]
    pub const fn opcode_old(&self) -> u8 {
        new_to_old(self.opcode())
    }
}

const fn opcode_binary(binary_op: BinaryOp) -> u8 {
    match binary_op {
        BinaryOp::Add => ADD,
        BinaryOp::Subtract => SUB,
        BinaryOp::Multiply => MUL,
        BinaryOp::Divide => DIV,
        BinaryOp::Remainder => REM,
        BinaryOp::Modulus => MOD,
        BinaryOp::And => AND,
        BinaryOp::Or => OR,
        BinaryOp::Xor => XOR,
        BinaryOp::ShiftLeft => SHL,
        BinaryOp::ShiftRight => SHR,
    }
}

/// Convert old WAD14 opcodes to new WAD15+ opcodes
#[must_use]
pub const fn old_to_new(opcode: u8) -> u8 {
    match opcode {
        // Convert is shifted by 4
        0x03 => CONV,

        // All mathematical operations are shifted by 4
        0x04..0x11 => opcode + 4,

        // Comparison Instructions used to be different opcodes (instead of comparsion type)
        0x11..0x17 => SET,

        // Pop and Dup are shifted by 4
        0x41 => POP,
        0x82 => DUP,

        // Branch Instructions are shifted by -1
        0xB7 => B,
        0xB8 => BT,
        0xB9 => BF,
        0xBB => PUSHENV,
        0xBC => POPENV,

        // Return and Exit are shifted by -1
        0x9D => RET,
        0x9E => EXIT,

        // Popz is shifted by -1
        0x9F => POPZ,

        // Call is shifted by -1
        0xDA => 0xD9,

        _ => opcode,
    }
}

/// Convert new WAD15+ opcodes to old WAD14 opcodes
#[must_use]
pub const fn new_to_old(opcode: u8) -> u8 {
    match opcode {
        // Convert is shifted by 4
        CONV => CONV - 4,

        // All mathematical operations are shifted by 4
        MUL..=SHR => opcode - 4,

        // Comparison Type should be handled by `build_comparison`
        SET => 0,

        // Pop and Dup are shifted by 4
        POP => 0x41,
        DUP => 0x82,

        // These specialized Push Instructions didn't exist back then
        PUSHI | PUSHG | PUSHL | PUSHV => PUSH,

        // Return and Exit are shifted by -1
        RET => 0x9D,
        EXIT => 0x9E,

        // Popz is shifted by -1
        POPZ => 0x9F,

        // Branch Instructions are shifted by -1
        B..=POPENV => opcode + 1,

        // Call is shifted by -1
        CALL => 0xDA,

        _ => opcode,
    }
}

pub mod extended {
    pub const CHKINDEX: i16 = -1;
    pub const PUSHAF: i16 = -2;
    pub const POPAF: i16 = -3;
    pub const PUSHAC: i16 = -4;
    pub const SETOWNER: i16 = -5;
    pub const ISSTATICOK: i16 = -6;
    pub const SETSTATIC: i16 = -7;
    pub const SAVEAREF: i16 = -8;
    pub const RESTOREAREF: i16 = -9;
    pub const ISNULLISH: i16 = -10;
    pub const PUSHREF: i16 = -11;
}

impl Instruction {
    /// The "Extended Opcode" for this instruction, if this is an "extended"
    /// (aka "break") instruction.
    #[must_use]
    pub const fn extended_kind(&self) -> Option<i16> {
        use super::opcode::extended;
        match self {
            Self::CheckArrayIndex => Some(extended::CHKINDEX),
            Self::PushArrayFinal => Some(extended::PUSHAF),
            Self::PopArrayFinal => Some(extended::POPAF),
            Self::PushArrayContainer => Some(extended::PUSHAC),
            Self::SetArrayOwner => Some(extended::SETOWNER),
            Self::HasStaticInitialized => Some(extended::ISSTATICOK),
            Self::SetStaticInitialized => Some(extended::SETSTATIC),
            Self::SaveArrayReference => Some(extended::SAVEAREF),
            Self::RestoreArrayReference => Some(extended::RESTOREAREF),
            Self::IsNullishValue => Some(extended::ISNULLISH),
            Self::PushReference { .. } => Some(extended::PUSHREF),
            _ => None,
        }
    }
}
