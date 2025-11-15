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

//mod old {
//    //! The (different) opcodes before bytecode 15
//    pub const CONV: u8 = 0x03;
//}

/// Convert old bytecode14 opcodes to new bytecode15+ opcodes
#[must_use]
pub const fn old_to_new(opcode: u8) -> u8 {
    match opcode {
        // Convert is shifted by 4
        0x03 => CONV,

        // All mathematical operations are shifted by 4
        0x04..0x11 => opcode + 4,

        // Comparison Instructions used to be different opcodes (instead of comparsion type)
        0x11..0x17 => CMP,

        // Pop and Dup are shifted by 4
        0x41 => POP,
        0x82 => DUP,

        // Branch Instructions are shifted by -1
        0xB7 => JMP,
        0xB8 => JT,
        0xB9 => JF,
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

/// Convert new bytecode15+ opcodes to old bytecode14 opcodes
#[must_use]
pub const fn new_to_old(opcode: u8) -> u8 {
    match opcode {
        // Convert is shifted by 4
        CONV => CONV - 4,

        // All mathematical operations are shifted by 4
        MUL..=SHR => opcode - 4,

        // Comparison Type should be handled by GMComparisonInstruction::build
        CMP => 0,

        // Pop and Dup are shifted by 4
        POP => 0x41,
        DUP => 0x82,

        // These specialized Push Instructions didn't exist back then
        PUSHIM | PUSHGLB | PUSHLOC | PUSHBLTN => PUSH,

        // Return and Exit are shifted by -1
        RET => 0x9D,
        EXIT => 0x9E,

        // Popz is shifted by -1
        POPZ => 0x9F,

        // Branch Instructions are shifted by -1
        JMP..=POPENV => opcode + 1,

        // Call is shifted by -1
        CALL => 0xDA,

        _ => opcode,
    }
}
