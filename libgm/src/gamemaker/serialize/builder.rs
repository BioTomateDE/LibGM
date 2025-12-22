use std::collections::HashMap;

use crate::{
    gamemaker::{
        data::{Endianness, GMData},
        elements::string::StringPlaceholder,
        gm_version::GMVersionReq,
    },
    gml::instruction::GMVariableType,
    prelude::*,
};

// The Default value should never be read.
// This can only happen if there are zero existant chunks, though.
#[derive(Debug, Clone, Default)]
pub struct LastChunk {
    pub length_pos: usize,
    pub padding_start_pos: usize,
}

#[derive(Debug)]
pub struct DataBuilder<'a> {
    /// The [`GMData`] to serialize.
    pub gm_data: &'a GMData,

    /// The raw data being generated.
    raw_data: Vec<u8>,

    /// Pairs data positions of pointer placeholders with the memory address of the GameMaker element they're pointing to
    pub(super) pointer_placeholder_positions: Vec<(u32, usize)>,

    /// Maps memory addresses of GameMaker elements to their resolved data position
    pub(super) pointer_resource_positions: HashMap<usize, u32>,

    /// Tracks where each function is used throughout the game data.
    /// Will be populated when code is built.
    /// - Outer Vec: Indexed by Function index from `gm_data.functions.functions`
    /// - Inner Vec: List of written positions for each occurrence
    pub function_occurrences: Vec<Vec<usize>>,

    /// Tracks where each variable is used throughout the game data.
    /// Will be populated when code is built.
    /// - Outer Vec: Indexed by Variable index from `gm_data.variables.variables`
    /// - Inner Vec: List of `(written_position, variable_type)` tuples for each occurrence
    pub variable_occurrences: Vec<Vec<(usize, GMVariableType)>>,

    pub string_placeholders: Vec<StringPlaceholder>,

    pub last_chunk: LastChunk,
}

impl<'a> DataBuilder<'a> {
    pub fn new(gm_data: &'a GMData) -> Self {
        let approximated_size: usize = (f64::from(gm_data.original_data_size) * 1.05) as usize;

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

    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.raw_data
    }

    /// The current length (aka. "position") of the internal buffer.
    pub const fn len(&self) -> usize {
        self.raw_data.len()
    }

    #[must_use]
    pub fn is_gm_version_at_least<V: Into<GMVersionReq>>(&self, version_req: V) -> bool {
        self.gm_data
            .general_info
            .version
            .is_version_at_least(version_req)
    }

    #[must_use]
    pub const fn wad_version(&self) -> u8 {
        self.gm_data.general_info.wad_version
    }

    /// Pads the internal buffer with zero bytes until its length is aligned to `alignment`.
    ///
    /// This adds zero bytes until `self.len()` is a multiple of `alignment`.
    pub fn align(&mut self, alignment: u32) {
        while !self.len().is_multiple_of(alignment as usize) {
            self.write_u8(0);
        }
    }

    /// Appends the given bytes to the internal data buffer.
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.raw_data.extend_from_slice(data);
    }

    /// Overwrites bytes at the given position in the internal data buffer.
    ///
    /// Useful for patching data like lengths or offsets after serialization.
    fn overwrite_bytes(&mut self, bytes: &[u8], position: usize) -> Result<()> {
        if let Some(mut_slice) = self.raw_data.get_mut(position..position + bytes.len()) {
            mut_slice.copy_from_slice(bytes);
            Ok(())
        } else {
            bail!(
                "Could not overwrite {} bytes at position {} in data with length {}; out of bounds",
                bytes.len(),
                position,
                self.raw_data.len(),
            );
        }
    }

    pub fn truncate_bytes(&mut self, count: usize) {
        self.raw_data.truncate(count);
    }

    /// Write a GameMaker boolean as a 32-bit integer.
    /// - If `true`, write `1_i32`.
    /// - If `false`, write `0_i32`.
    pub fn write_bool32(&mut self, boolean: bool) {
        self.write_i32(boolean.into());
    }

    /// Write an actual character string.
    ///
    /// This should only be used for literal strings in the `STRG` chunk.
    /// For writing regular GameMaker string references, see [`Self::write_gm_string`].
    pub fn write_literal_string(&mut self, string: &str) {
        self.write_bytes(string.as_bytes());
    }

    /// Overwrites a 4-byte unsigned integer (`usize` truncated to `u32`) at `position`.
    ///
    /// Useful for patching fixed-size numeric values like lengths or offsets after serialization.
    /// For writing regular pointer lists, see [`Self::write_pointer_list`].
    pub fn overwrite_usize(&mut self, number: usize, position: usize) -> Result<()> {
        let number: u32 = number as u32;
        let bytes: [u8; 4] = match self.gm_data.endianness {
            Endianness::Little => number.to_le_bytes(),
            Endianness::Big => number.to_be_bytes(),
        };
        self.overwrite_bytes(&bytes, position)
    }

    /// Overwrites a 4-byte signed integer (`i32`) at `position`.
    ///
    /// Useful for patching fixed-size numeric values
    /// like lengths or offsets after serialization.
    /// For writing regular pointer lists, see [`Self::write_pointer_list`].
    pub fn overwrite_i32(&mut self, number: i32, position: usize) -> Result<()> {
        let bytes: [u8; 4] = match self.gm_data.endianness {
            Endianness::Little => number.to_le_bytes(),
            Endianness::Big => number.to_be_bytes(),
        };
        self.overwrite_bytes(&bytes, position)
    }
}
