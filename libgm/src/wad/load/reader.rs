// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use crate::gml::instruction::VariableType;
use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::chunk::ChunkName;
use crate::wad::chunk::ChunkOrder;
use crate::wad::data::Endianness;
use crate::wad::elem::GMComplexElement;
use crate::wad::elem::GMElement;
use crate::wad::elem::function::Function;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::elem::variable::Variable;
use crate::wad::load::LoadingOptions;
use crate::wad::load::chunk::ChunkBounds;
use crate::wad::load::chunk::ChunkMap;
use crate::wad::reference::GMRef;
use crate::wad::version::GMVersion;

#[derive(Debug, Clone)]
pub struct DataReader<'a> {
    /// The raw data buffer belonging to the GameMaker data file which is
    /// currently being parsed.
    data: &'a [u8],

    /// The current read position within the data buffer.
    /// Reading data will be read from this position; incrementing it.
    pub cur_pos: u32,

    pub version: GMVersion,

    /// How many null bytes of padding should be at the end of every chunk
    /// (except the last one). Only relevant in certain GameMaker versions.
    /// Defaults to 16, but will be set to 4 or 1 if detected.
    pub chunk_padding: u32,

    /// Indicates the data file's byte endianness.
    /// In most cases (and assumed by default), this is set to little-endian.
    /// Big endian is an edge case for certain target platforms (e.g. PS3 or
    /// Xbox 360).
    pub endianness: Endianness,

    /// Map of all chunks specified by `FORM`; indexed by chunk name.
    /// Read chunks will be removed from this `HashMap` when calling
    /// [`DataReader::read_chunk`]. May contain unknown chunks (if there is
    /// a GameMaker update, for example).
    pub chunks: ChunkMap,

    pub chunk_order: ChunkOrder,

    /// Metadata about the currently parsed chunk of data.
    /// This includes the chunk's name, start position, and end position within
    /// the data buffer. When reading data, these bounds are checked to
    /// ensure the read operation stays within the chunk.
    pub chunk: ChunkBounds,

    /// The name of the last chunk in the data file.
    /// Is properly initialized after parsing `FORM`.
    pub last_chunk: ChunkName,

    /// Chunk `STRG`.
    /// Is properly initialized after parsing `FORM`.
    pub string_chunk: ChunkBounds,

    /// Contains parsing options (wow!).
    /// Properly initialized after parsing `FORM`.
    pub options: LoadingOptions,

    /// Should only be set by [`crate::wad::elem::string`].
    /// This means that `STRG` has to be parsed before any other chunk
    /// (any chunk with string pointers).
    pub string_occurrences: HashMap<u32, GMRef<String>>,

    /// Should only be set by [`crate::wad::elem::texture_page_item`].
    /// This means that `TPAG` has to be parsed before any chunk with texture
    /// page item pointers.
    pub texture_page_item_occurrences: HashMap<u32, GMRef<TexturePageItem>>,

    /// Should only be set by [`crate::wad::elem::variable`].
    /// This means that `VARI` has to be parsed before `CODE`.
    pub variable_occurrences: HashMap<u32, (GMRef<Variable>, VariableType)>,

    /// Should only be set by [`crate::wad::elem::function`].
    /// This means that `FUNC` has to be parsed before `CODE`.
    pub function_occurrences: HashMap<u32, GMRef<Function>>,
}

impl<'a> DataReader<'a> {
    /// Creates a new [`DataReader`] for the given raw GameMaker data.
    ///
    /// Most fields will be filled with a sentinel/stub value before
    /// being properly initialized while parsing FORM.
    pub fn new(data: &'a [u8]) -> Self {
        // Memory Safety Assertion. This should've been verified before, though.
        let end_pos: u32 = data.len().try_into().expect("Data length out of u32 bounds");

        Self {
            data,
            cur_pos: 0,
            version: GMVersion::Wad12, // stub
            // The default padding value is 16, if used.
            chunk_padding: 16,
            // Assume little endian; big endian is an edge case.
            endianness: Endianness::Little,
            chunk: ChunkBounds { start_pos: 0, end_pos },
            chunks: ChunkMap::new(),
            chunk_order: ChunkOrder::new_empty(),
            last_chunk: ChunkName::DAFL,          // stub
            string_chunk: ChunkBounds::default(), // stub
            options: LoadingOptions::default(),   // stub
            string_occurrences: HashMap::new(),
            texture_page_item_occurrences: HashMap::new(),
            variable_occurrences: HashMap::new(),
            function_occurrences: HashMap::new(),
        }
    }

    /// The size / byte length of the data file.
    pub const fn size(&self) -> u32 {
        self.data.len() as u32
    }

    /// Reads the specified number of bytes from the data file while advancing
    /// the data position. Returns an error when trying to read out of chunk
    /// bounds.
    ///
    /// This is the core data reading abstraction. All other methods build up on
    /// this.
    pub fn read_bytes_dyn(&mut self, count: u32) -> Result<&'a [u8]> {
        let start: u32 = self.cur_pos;
        let end: u32 = self.cur_pos.checked_add(count).ok_or("Trying to read out of u32 bounds")?;

        // Lower chunk bounds check
        if start < self.chunk.start_pos {
            bail!(
                "Trying to read {} bytes out of lower chunk bounds at position {} with start \
                 position {}",
                count,
                self.cur_pos,
                self.chunk.start_pos,
            );
        }

        // Upper chunk bounds check
        if end > self.chunk.end_pos {
            bail!(
                "Trying to read {} bytes out of upper chunk bounds at position {} with end \
                 position {}",
                count,
                self.cur_pos,
                self.chunk.end_pos,
            );
        }

        let start = start as usize;
        let end = end as usize;
        let slice: &[u8] = self.data.get(start..end).expect("chunk bounds not set correctly");
        self.cur_pos += count;
        Ok(slice)
    }

    /// Reads a constant number of bytes from the data file while advancing the
    /// data position. Useful for reading slices with specified sizes like
    /// `[u8; 16]`.
    pub fn read_bytes_const<const N: usize>(&mut self) -> Result<&[u8; N]> {
        const {
            assert!(N < u32::MAX as usize);
        }
        let slice: &[u8] = self.read_bytes_dyn(N as u32)?;
        let array: &[u8; N] = slice.try_into().expect("read_bytes_dyn always reads N bytes");
        Ok(array)
    }

    /// Reads a 32-bit integer and convert it to a boolean.
    /// ___
    /// Returns an error when the read number is neither 0 nor 1.
    pub fn read_bool32(&mut self) -> Result<bool> {
        let number = self.read_u32()?;
        match number {
            0 => Ok(false),
            1 => Ok(true),
            n => bail!("Invalid boolean value {n} (0x{n:08X}) at position {}", self.cur_pos - 4,),
        }
    }

    /// Reads a UTF-8 character string with the specified byte length.
    /// ___
    /// For reading standard GameMaker string references, see
    /// [`DataReader::read_gm_string`].
    pub fn read_literal_string(&mut self, length: u32) -> Result<String> {
        let bytes: Vec<u8> = self
            .read_bytes_dyn(length)
            .ctx(|| format!("reading literal string with length {length}"))?
            .to_vec();

        let string: String = String::from_utf8(bytes).ctx_any(|| {
            format!(
                "parsing literal UTF-8 string with length {} at position {}",
                length,
                self.cur_pos - length,
            )
        })?;

        Ok(string)
    }

    /// Reads bytes until the reader position is divisible by the specified
    /// alignment. Ensures the read padding bytes are all zero.
    pub fn align(&mut self, alignment: u32) -> Result<()> {
        let start_pos = self.cur_pos;
        let end_pos: u32 = self.cur_pos.next_multiple_of(alignment);
        let byte_count: u32 = end_pos - self.cur_pos;
        let context = || format!("aligning to a multiple of {alignment} bytes");
        let bytes: &[u8] = self.read_bytes_dyn(byte_count).ctx(context)?;
        if bytes.iter().all(|&b| b == 0) {
            return Ok(());
        }

        let error = format!("Found misaligned data at position {start_pos}");
        if self.options.verify_alignment {
            return Err(Error::new(error).with_context(context));
        }

        // lenient mode: cancel aligning; go back and try to read the data. will probably not work.
        log::warn!("{error}! Going back to read it.");
        self.cur_pos = start_pos;
        Ok(())
    }

    /// Sets the reader position to the current chunk's start position plus the
    /// specified relative position.
    pub fn set_rel_cur_pos(&mut self, relative_pos: u32) -> Result<()> {
        let start = self.chunk.start_pos;
        let end = self.chunk.end_pos;
        let pos = start.checked_add(relative_pos).ok_or_else(|| {
            format!(
                "Relative position {relative_pos} would
                overflow from start position {start}"
            )
        })?;

        if pos > end {
            bail!(
                "Position {pos} (start {start} + relative {relative_pos})
                exceeds chunk end position {end}"
            );
        }

        self.cur_pos = pos;
        Ok(())
    }

    /// Deserializes an element if the GameMaker version meets the requirement
    /// (`>=`).
    ///
    /// This is useful for handling format changes across different GameMaker
    /// versions where certain chunks or fields were added, removed, or
    /// modified.
    ///
    /// # Returns
    /// - `Ok(Some(T))` if the version requirement is met and deserialization
    ///   succeeds
    /// - `Ok(None)` if the version requirement is not met
    /// - `Err(_)` if the version requirement is met but deserialization fails
    pub fn deserialize_if_version<T: GMElement>(
        &mut self,
        ver_req: GMVersion,
    ) -> Result<Option<T>> {
        if self.version >= ver_req { Ok(Some(T::deserialize(self)?)) } else { Ok(None) }
    }

    #[inline]
    pub fn read_elem_ctx<T: GMComplexElement>(&mut self, ctx: T::ReadContext) -> Result<T> {
        let start_pos = self.cur_pos;
        T::deserialize_complex(self, ctx)
            .ctx(|| format!("deserializing {} at position {start_pos}", typename::<T>()))
    }

    #[inline]
    pub fn read_elem<T: GMElement>(&mut self) -> Result<T> {
        self.read_elem_ctx(())
    }
}
