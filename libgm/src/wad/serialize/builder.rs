use std::collections::HashMap;

use crate::{
    gml::instruction::VariableType,
    prelude::*,
    wad::{
        data::{Endianness, GMData},
        elements::string::StringPlaceholder,
        serialize::pointers::Pointer,
        version::GMVersionReq,
    },
};

// The Default value should never be read.
// This can only happen if there are zero existent chunks, though.
#[derive(Debug, Clone, Default)]
pub struct LastChunk {
    pub length_pos: u32,
    pub padding_start_pos: u32,
}

#[derive(Debug)]
pub struct DataBuilder<'a> {
    /// The [`GMData`] to serialize.
    pub gm_data: &'a GMData,

    /// The raw data being generated.
    pub raw_data: Vec<u8>,

    /// Pairs data positions of pointer placeholders with the memory address of the GameMaker element they're pointing to.
    pub(super) pointer_placeholder_positions: Vec<(u32, Pointer)>,

    /// Maps memory addresses of GameMaker elements to their resolved data position.
    pub(super) pointer_resource_positions: HashMap<Pointer, u32>,

    /// Tracks where each function is used throughout the game data.
    /// Will be populated when code is built.
    /// - Outer Vec: Indexed by Function index from `gm_data.functions.functions`
    /// - Inner Vec: List of written positions for each occurrence
    pub function_occurrences: Vec<Vec<u32>>,

    /// Tracks where each variable is used throughout the game data.
    /// Will be populated when code is built.
    /// - Outer Vec: Indexed by Variable index from `gm_data.variables.variables`
    /// - Inner Vec: List of `(written_position, variable_type)` tuples for each occurrence
    pub variable_occurrences: Vec<Vec<(u32, VariableType)>>,

    pub string_placeholders: Vec<StringPlaceholder>,

    pub last_chunk: LastChunk,
}

impl<'a> DataBuilder<'a> {
    pub fn new(gm_data: &'a GMData) -> Self {
        let approximated_size: usize = (f64::from(gm_data.meta.original_data_size) * 1.05) as usize;

        Self {
            gm_data,
            raw_data: Vec::with_capacity(approximated_size),
            pointer_placeholder_positions: Vec::new(),
            pointer_resource_positions: HashMap::new(),
            function_occurrences: vec![Vec::new(); gm_data.functions.len()],
            variable_occurrences: vec![Vec::new(); gm_data.variables.len()],
            string_placeholders: Vec::new(),
            last_chunk: LastChunk::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.raw_data
    }

    /// The current length (aka. "position") of the internal buffer.
    #[inline]
    pub const fn len(&self) -> u32 {
        self.raw_data.len() as u32
    }

    #[inline]
    #[must_use]
    pub fn is_version_at_least(&self, req: impl Into<GMVersionReq>) -> bool {
        self.gm_data.general_info.version.is_version_at_least(req)
    }

    #[inline]
    #[must_use]
    pub const fn wad_version(&self) -> u8 {
        self.gm_data.general_info.wad_version
    }

    /// Pads the internal buffer with zero bytes until its length is aligned to `alignment`.
    ///
    /// This adds zero bytes until `self.len()` is a multiple of `alignment`.
    #[inline]
    pub fn align(&mut self, alignment: u32) {
        while !self.len().is_multiple_of(alignment) {
            self.write_u8(0);
        }
    }

    /// Appends the given bytes to the internal data buffer.
    #[inline]
    pub fn write_bytes(&mut self, data: &[u8]) {
        // It's really stupid how this function is a one-liner whereas
        // reading bytes needs like 50 lines to be handled correctly...
        self.raw_data.extend_from_slice(data);
    }

    /// Overwrites bytes at the given position in the internal data buffer.
    ///
    /// Useful for patching data like lengths or offsets after serialization.
    fn overwrite_bytes(&mut self, bytes: &[u8], position: u32) -> Result<()> {
        let start = position as usize;
        let end = start + bytes.len();
        if let Some(mut_slice) = self.raw_data.get_mut(start..end) {
            mut_slice.copy_from_slice(bytes);
            Ok(())
        } else {
            Err(err!(
                "Could not overwrite {} bytes at position {} in data with length {}; out of bounds",
                bytes.len(),
                position,
                self.raw_data.len(),
            ))
        }
    }

    /// Overwrites a 32-bit unsigned integer at the specified written data `position`.
    ///
    /// Useful for patching fixed-size numeric values like lengths or offsets after serialization.
    /// For writing regular pointer lists normally, see [`Self::write_pointer_list`].
    pub fn overwrite_u32(&mut self, number: u32, position: u32) -> Result<()> {
        let bytes: [u8; 4] = match self.gm_data.meta.endianness {
            Endianness::Little => number.to_le_bytes(),
            Endianness::Big => number.to_be_bytes(),
        };
        self.overwrite_bytes(&bytes, position).with_context(|| {
            format!("overwriting 32-bit number {number} at target data position {position}")
        })
    }

    /// Overwrites a 32-bit number at the specified data position with the current data position (length).
    /// The data position is calculated via `pointer_list_pos + (4 * element_index)`
    /// which is suitable for overwriting pointer in a pointer list.
    ///
    /// For overwriting other numbers, see [`Self::overwrite_u32`].
    /// For writing regular pointer lists normally, see [`Self::write_pointer_list`].
    #[inline]
    pub fn overwrite_pointer_with_cur_pos(
        &mut self,
        pointer_list_pos: u32,
        element_index: usize,
    ) -> Result<()> {
        let number: u32 = self.len();
        let position: u32 = pointer_list_pos + 4 * element_index as u32;
        self.overwrite_u32(number, position).with_context(|| {
            format!(
                "overwriting pointer for element index {element_index} of \
                pointer list with start position {pointer_list_pos}"
            )
        })
    }

    /// Write a GameMaker boolean as a 32-bit integer.
    /// - If `true`, write `1_i32`.
    /// - If `false`, write `0_i32`.
    #[inline]
    pub fn write_bool32(&mut self, boolean: bool) {
        self.write_i32(boolean.into());
    }

    /// Write an actual character string.
    ///
    /// This should only be used for literal strings in the `STRG` chunk.
    /// For writing regular GameMaker string references, see [`Self::write_gm_string`].
    #[inline]
    pub fn write_literal_string(&mut self, string: &str) {
        self.write_bytes(string.as_bytes());
    }
}
