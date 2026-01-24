//! Everything related to GML (GameMaker language) bytecode.

pub mod assembly;
pub mod instruction;
pub(crate) mod opcodes;

pub use crate::gml::instruction::Instruction;

use crate::prelude::GMRef;

/// A code entry in a GameMaker data file.
#[derive(Debug, Clone, PartialEq)]
pub struct GMCode {
    /// The name of the code entry.
    pub name: String,

    /// A list of VM instructions this code entry has.
    pub instructions: Vec<Instruction>,

    /// Set in WAD 15+.
    pub modern_data: Option<ModernData>,
}

/// Extra data for code entries in WAD Version 15 and higher.
#[derive(Debug, Clone, PartialEq)]
pub struct ModernData {
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
