//! Everything related to GML (GameMaker language) bytecode.
//!
//! TODO(doc): explain more.
//! For now, just visit [`crate::wad`] sorry lol

pub mod analysis;
pub mod assembly;
pub mod instruction;
pub(crate) mod opcodes;

pub use crate::gml::instruction::Instruction;
use crate::prelude::*;
use std::ops::Range;

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

        let mut children: Vec<GMRef<Self>> = Vec::new();

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

        let mut children: Vec<GMRef<Self>> = Vec::new();

        for (idx, code_entry) in data.codes.iter().enumerate() {
            if code_entry.parent() == Some(code_ref) {
                children.push(GMRef::from(idx));
            }
        }

        children
    }

    /// Gets the total (cumulative) size of all instructions, in bytes.
    ///
    /// This function simply calls [`Instruction::size`] on each instruction and sums up the sizes.
    #[must_use]
    pub fn length(&self) -> u32 {
        instructions_size(&self.instructions)
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
    pub argument_count: u16,

    /// A flag set on certain code entries, which usually don't have locals attached to them.
    pub weird_local_flag: bool,

    /// Offset, **in bytes**, where code should begin executing from within the bytecode of this code entry.
    /// Should be 0 for root-level (parent) code entries, and nonzero for child code entries.
    pub execution_offset: u32,

    /// Parent entry of this code entry, if this is a child entry; [`None`] otherwise.
    pub parent: Option<GMRef<GMCode>>,
}

/// Gets the total (cumulative) size of all instructions, in bytes.
///
/// This function simply calls [`Instruction::size`] on each instruction and sums up the sizes.
#[must_use]
pub fn instructions_size(instructions: &[Instruction]) -> u32 {
    let mut size: u32 = 0;
    for instruction in instructions {
        size += instruction.size();
    }
    size
}

/// TODO: test this
fn splice_instructions(
    haystack: &mut Vec<Instruction>,
    range: Range<u32>,
    replace_with: &[Instruction],
    needs_result: bool,
) -> Result<Option<Vec<Instruction>>> {
    let start = range.start as usize;
    let end = range.end as usize;
    let len = haystack.len();

    if start >= len {
        bail!("Start index {start} out of bounds for vector with instruction count {len}");
    }

    if start > end {
        bail!("Start index {start} is greater than the end index {end}");
    }

    let insertion_size = instructions_size(replace_with);
    let removal_size = instructions_size(&haystack[start..end]);
    let fixed_offset = (insertion_size - removal_size) as i32 / 4;

    let first_half_size = instructions_size(&haystack[..start]) as i32 / 4;

    let mut cur_pos: u32 = 0;
    for (i, instr) in haystack.iter_mut().enumerate() {
        // (this technically ignores stuff that is neither half 1 nor 2 if range is nonzero but it
        // shouldnt make a different i think)
        cur_pos += instr.size4();
        let Some(offset) = instr.jump_offset_mut() else {
            continue;
        };

        let branch_target_pos = cur_pos as i32 + *offset;
        let origin_is_first_half = i < start;
        let target_is_first_half = branch_target_pos < first_half_size;

        // if branching withing their half, everything is fine.
        if origin_is_first_half == target_is_first_half {
            continue;
        }

        // branch crosses boundary; fix the jump offsets.
        if origin_is_first_half {
            *offset += fixed_offset;
        } else {
            *offset -= fixed_offset;
        }
    }

    // now perform the actual splice
    let iter = haystack.splice(start..end, replace_with.iter().cloned());
    if needs_result {
        Ok(Some(iter.collect()))
    } else {
        Ok(None)
    }
}

pub fn insert_instructions(
    haystack: &mut Vec<Instruction>,
    index: u32,
    insertion: &[Instruction],
) -> Result<()> {
    splice_instructions(haystack, index..index, insertion, false).with_context(|| {
        format!(
            "inserting {} instructions at index {} into vector with {} instructions",
            insertion.len(),
            index,
            haystack.len(),
        )
    })?;
    Ok(())
}

pub fn insert_instruction(
    haystack: &mut Vec<Instruction>,
    index: u32,
    insertion: &Instruction,
) -> Result<()> {
    insert_instructions(haystack, index, std::slice::from_ref(insertion))
        .with_context(|| format!("inserting single instruction {insertion:?}"))
}

#[allow(clippy::missing_panics_doc)]
pub fn remove_instructions(
    haystack: &mut Vec<Instruction>,
    range: Range<u32>,
) -> Result<Vec<Instruction>> {
    let removal_len = range.len();
    let index = range.start;
    let old_instrs = splice_instructions(haystack, range, &[], true).with_context(|| {
        format!(
            "removing {} instructions at index {} of vector with {} instructions",
            removal_len,
            index,
            haystack.len(),
        )
    })?;
    Ok(old_instrs.unwrap())
}

pub fn remove_instruction(haystack: &mut Vec<Instruction>, index: u32) -> Result<Vec<Instruction>> {
    remove_instructions(haystack, index..index + 1)
}
