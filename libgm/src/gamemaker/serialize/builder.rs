use crate::gamemaker::data::{Endianness, GMData};
use crate::gamemaker::elements::GMChunkElement;
use crate::gamemaker::elements::code::GMVariableType;
use crate::gamemaker::elements::strings::StringPlaceholder;
use crate::gamemaker::gm_version::GMVersionReq;
use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::util::fmt::typename;
use std::collections::HashMap;

// The Default value should never be read.
// This can only happen if there are zero existant chunks, though.
#[derive(Debug, Default)]
pub struct LastChunk {
    pub length_pos: usize,
    pub padding_start_pos: usize,
}

#[derive(Debug)]
pub struct DataBuilder<'a> {
    /// The [`GMData`] to serialize.
    pub gm_data: &'a GMData,

    /// The raw data being generated.
    pub raw_data: Vec<u8>,

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
        let approximated_size: usize = (gm_data.original_data_size as f64 * 1.05) as usize;
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

    /// The current length (aka. "position") of the internal buffer.
    pub const fn len(&self) -> usize {
        self.raw_data.len()
    }

    pub fn is_gm_version_at_least<V: Into<GMVersionReq>>(&self, version_req: V) -> bool {
        self.gm_data.general_info.version.is_version_at_least(version_req)
    }

    pub const fn bytecode_version(&self) -> u8 {
        self.gm_data.general_info.bytecode_version
    }

    /// Pads the internal buffer with zero bytes until its length is aligned to `alignment`.
    ///
    /// This adds zero bytes until `self.len()` is a multiple of `alignment`.
    pub fn align(&mut self, alignment: u32) {
        while self.len() % (alignment as usize) != 0 {
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

    /// Write a GameMaker boolean as a 32-bit integer.
    /// - If `true`, write `1_i32`.
    /// - If `false`, write `0_i32`.
    pub fn write_bool32(&mut self, boolean: bool) {
        let int: i32 = match boolean {
            true => 1,
            false => 0,
        };
        self.write_i32(int);
    }

    /// Write an actual character string.
    ///
    /// This should only be used for literal strings in the `STRG` chunk.
    /// For writing regular GameMaker string references, see [Self::write_gm_string].
    pub fn write_literal_string(&mut self, string: &str) {
        self.write_bytes(string.as_bytes());
    }

    /// Write a 4 character ASCII GameMaker chunk name.
    /// Accounts for endianness (chunk names in big endian are reversed).
    pub fn write_chunk_name(&mut self, name: &str) -> Result<()> {
        if name.len() != 4 {
            bail!(
                "Expected chunk name '{}' to be 4 characters long; but is actually {} characters long",
                name,
                name.len(),
            );
        }

        let mut bytes: [u8; 4] = name.as_bytes().try_into().with_context(|| {
            format!(
                "Expected chunk name '{}' to be 4 bytes long; but it's actually {} bytes long",
                name,
                name.as_bytes().len(),
            )
        })?;

        if self.gm_data.endianness == Endianness::Big {
            bytes.reverse();
        }

        self.write_bytes(&bytes);
        Ok(())
    }

    /// Overwrites a 4-byte unsigned integer (`usize` truncated to `u32`) at `position`.
    ///
    /// Useful for patching fixed-size numeric values like lengths or offsets after serialization.
    /// For writing regular pointer lists, see [Self::write_pointer_list].
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
    /// Useful for patching fixed-size numeric values like lengths or offsets after serialization.
    /// For writing regular pointer lists, see `[Self::write_pointer_list].
    pub fn overwrite_i32(&mut self, number: i32, position: usize) -> Result<()> {
        let bytes: [u8; 4] = match self.gm_data.endianness {
            Endianness::Little => number.to_le_bytes(),
            Endianness::Big => number.to_be_bytes(),
        };
        self.overwrite_bytes(&bytes, position)
    }

    /// Create a placeholder pointer at the current position in the chunk and remember
    /// its data position paired with the target GameMaker element's memory address.
    ///
    /// This will later be resolved by calling [`Self::resolve_pointer`]; replacing the
    /// pointer placeholder with the written data position of the target GameMaker element.
    /// ___
    /// This system exists because it is virtually impossible to predict which data position a GameMaker element will be written to.
    /// Circular references and writing order would make predicting these pointer resource positions even harder.
    /// ___
    /// This function should NOT be called for `GMRef`s; use their `DataBuilder::write_gm_x()` methods instead.
    pub fn write_pointer<T>(&mut self, element: &T) -> Result<()> {
        let memory_address: usize = element as *const _ as usize;
        let placeholder_position: u32 = self.len() as u32; // Gamemaker is 32bit anyway
        self.write_u32(0xDEAD_C0DE);
        self.pointer_placeholder_positions
            .push((placeholder_position, memory_address));
        Ok(())
    }

    /// Optionally writes a pointer to the given [`Option`] value.
    /// - If [`Some`], writes a pointer to the contained value using [`Self::write_pointer`].
    /// - If [`None`], writes a null pointer (0) using [`Self::write_i32`].
    pub fn write_pointer_opt<T>(&mut self, element: &Option<T>) -> Result<()> {
        if let Some(elem) = element {
            self.write_pointer(elem)?;
        } else {
            self.write_i32(0);
        }
        Ok(())
    }

    /// Store the written GameMaker element's data position paired with its memory address in the pointer resource pool.
    /// The element's position corresponds to the data builder's current position,
    /// since this method should get called when the element is serialized.
    pub fn resolve_pointer<T>(&mut self, element: &T) -> Result<()> {
        let memory_address: usize = element as *const T as usize;
        let resource_position: u32 = self.len() as u32;
        if let Some(old_resource_pos) = self
            .pointer_resource_positions
            .insert(memory_address, resource_position)
        {
            bail!(
                "Pointer placeholder for {} with memory address {} already resolved \
                to data position {}; tried to resolve again to data position {}",
                typename::<T>(),
                memory_address,
                old_resource_pos,
                resource_position,
            );
        }
        Ok(())
    }

    /// Writes a GameMaker data chunk.
    /// Skips the chunk if the element does not exist.
    ///
    /// Appends padding if required by the GameMaker version.
    /// This padding has to then be manually cut off for the last chunk in the data file.
    pub fn build_chunk<T: GMChunkElement>(&mut self, element: &T) -> Result<()> {
        let name: &str = T::NAME;
        if !element.exists() {
            return Ok(());
        }

        let stopwatch = Stopwatch::start();

        self.write_chunk_name(name).expect("Constant chunk name is invalid");
        self.write_u32(0xDEADC0DE); // Chunk length placeholder
        let start_pos: usize = self.len();
        let length_pos = start_pos - 4;

        element
            .serialize(self)
            .with_context(|| format!("serializing chunk '{name}'"))?;

        // Write padding in these versions
        let padding_start_pos = self.len();
        let ver = &self.gm_data.general_info.version;
        if ver.major >= 2 || (ver.major == 1 && ver.build >= 9999) {
            self.align(self.gm_data.chunk_padding);
        }

        // Since the padding should not get written for the last chunk,
        // set the `last_chunk` field to potentially remove the padding later.
        self.last_chunk = LastChunk { length_pos, padding_start_pos };

        // Resolve chunk length placeholder
        let chunk_length: usize = self.len() - start_pos;
        self.overwrite_usize(chunk_length, length_pos)
            .expect("Chunk length overwrite position out of bounds");

        log::trace!("Building chunk '{name}' took {stopwatch}");
        Ok(())
    }
}
