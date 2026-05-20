// SPDX-License-Identifier: GPL-3.0-only
//! Functions to assemble and disassemble GML code.

mod assembler;
mod disassembler;

pub use assembler::assemble_instruction;
pub use assembler::assemble_instructions;
pub use disassembler::disassemble_code;
pub use disassembler::disassemble_instruction;
pub use disassembler::disassemble_instructions;
