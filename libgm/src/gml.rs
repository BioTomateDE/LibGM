//! Everything related to GML (GameMaker language) bytecode.
//!
//! TODO(doc): explain more.
//! For now, just visit [`crate::gamemaker`] sorry lol

pub mod analysis;
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

impl GMCode {
    /// The parent code entry of this code entry, if it has one.
    ///
    /// This will always be [`None`] for WAD < 15.
    #[must_use]
    pub const fn parent(&self) -> Option<GMRef<Self>> {
        match &self.modern_data {
            Some(data) => data.parent,
            None => None,
        }
    }

    /// Whether this code entry is a root entry, meaning it has no parent code entries.
    ///
    /// This will always be `true` for WAD < 15.
    #[must_use]
    pub const fn is_root(&self) -> bool {
        self.parent().is_none()
    }

    /// The offset, **in bytes**, where code should begin
    /// executing from within the bytecode of this code entry.
    ///
    /// This will always be zero for root code entries and before WAD 15.
    #[must_use]
    pub const fn execution_offset(&self) -> u32 {
        match &self.modern_data {
            Some(data) => data.offset,
            None => 0,
        }
    }
}

/// Extra data for code entries in WAD Version 15 and higher.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModernData {
    /// The amount of local variables this code entry has.
    /// TODO(break):  rename to local_count
    pub locals_count: u16,

    /// The amount of arguments this code entry accepts.
    pub arguments_count: u16,

    /// A flag set on certain code entries, which usually don't have locals attached to them.
    pub weird_local_flag: bool,

    /// Offset, **in bytes**, where code should begin executing from within the bytecode of this code entry.
    /// Should be 0 for root-level (parent) code entries, and nonzero for child code entries.
    /// TODO(break): rename to execution_offset
    pub offset: u32,

    /// Parent entry of this code entry, if this is a child entry; [`None`] otherwise.
    pub parent: Option<GMRef<GMCode>>,
}
