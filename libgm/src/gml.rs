//! Everything related to GML (GameMaker language) bytecode.
//!
//! TODO(doc): explain more.
//! For now, just visit [`crate::gamemaker`] sorry lol

pub mod analysis;
pub mod assembly;
pub mod instruction;
pub(crate) mod opcodes;

pub use crate::gml::instruction::Instruction;
use crate::prelude::*;

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
    // TODO: make a helper function to insert / remove instructions that preserves branch offsets.

    /// Find child code entries of this code entry.
    ///
    /// This is always `false` before WAD 15, since child/parent code entries did not exist then.
    ///
    /// This function has to compare the names of code entries and is also failable.
    /// If you have access to a [`GMRef`] to this code entry instead, consider
    /// using [`Self::find_children`] instead.
    ///
    /// This has to iterate over all code entries in the data file,
    /// so it's a good idea to cache this if possible.
    pub fn find_children_by_name(&self, data: &GMData) -> Result<Vec<GMRef<Self>>> {
        if data.general_info.wad_version < 15 {
            return Ok(Vec::new());
        }

        let mut children: Vec<GMRef<GMCode>> = Vec::new();

        for (idx, code_entry) in data.codes.iter().enumerate() {
            let Some(parent) = code_entry.parent() else {
                continue;
            };
            let parent = data.codes.by_ref(parent)?;
            if self.name == parent.name {
                children.push(GMRef::from(idx));
            }
        }

        Ok(children)
    }

    /// Find child code entries of this code entry.
    ///
    /// This is always `false` before WAD 15, since child/parent code entries did not exist then.
    ///
    /// This function takes a `GMRef<GMCode>` instead of `&GMCode`.
    /// If you only have a [`GMCode`] available, you'll have to
    /// use [`Self::find_children_by_name`] instead.
    ///
    /// This has to iterate over all code entries in the data file,
    /// so it's a good idea to cache this if possible.
    #[must_use]
    pub fn find_children(code_ref: GMRef<Self>, data: &GMData) -> Vec<GMRef<Self>> {
        if data.general_info.wad_version < 15 {
            return Vec::new();
        }

        let mut children: Vec<GMRef<GMCode>> = Vec::new();

        for (idx, code_entry) in data.codes.iter().enumerate() {
            if code_entry.parent() == Some(code_ref) {
                children.push(GMRef::from(idx));
            }
        }

        children
    }

    /// Get the total size of all instructions in bytes.
    ///
    /// This is not equivalent to (a multiple of) the instruction count,
    /// so this function may be needed for determining instructions byte size.
    #[must_use]
    pub fn length(&self) -> u32 {
        let mut size: u32 = 0;
        for instruction in &self.instructions {
            size += instruction.size();
        }
        size
    }

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
            Some(data) => data.execution_offset,
            None => 0,
        }
    }
}

/// Extra data for code entries in WAD Version 15 and higher.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModernData {
    /// The amount of local variables this code entry has.
    pub local_count: u16,

    /// The amount of arguments this code entry accepts.
    pub arguments_count: u16,

    /// A flag set on certain code entries, which usually don't have locals attached to them.
    pub weird_local_flag: bool,

    /// Offset, **in bytes**, where code should begin executing from within the bytecode of this code entry.
    /// Should be 0 for root-level (parent) code entries, and nonzero for child code entries.
    pub execution_offset: u32,

    /// Parent entry of this code entry, if this is a child entry; [`None`] otherwise.
    pub parent: Option<GMRef<GMCode>>,
}
