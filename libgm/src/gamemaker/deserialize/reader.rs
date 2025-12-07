use std::collections::HashMap;

use crate::{
    gamemaker::{
        chunk::ChunkName,
        data::Endianness,
        deserialize::chunk::{ChunkBounds, Chunks},
        elements::{
            GMElement, functions::GMFunction, general_info::GMGeneralInfo,
            texture_page_items::GMTexturePageItem, variables::GMVariable,
        },
        gm_version::{GMVersion, GMVersionReq},
        reference::GMRef,
    },
    prelude::*,
    util::assert::assert_int,
};

#[derive(Debug)]
pub struct DataReader<'a> {
    /// The raw data buffer belonging to the GameMaker data file which is currently being parsed.
    data: &'a [u8],

    /// The current read position within the data buffer.
    /// Reading data will be read from this position; incrementing it.
    pub cur_pos: u32,

    /// The GameMaker version specified by GEN8.
    /// The "actual" version will be detected later and stored in `general_info.version`.
    pub specified_version: GMVersion,

    /// How many null bytes of padding should be at the end of every chunk (except the last one).
    /// Only relevant in certain GameMaker versions.
    /// Defaults to 16, but will be set to 4 or 1 if detected.
    pub chunk_padding: u32,

    /// Indicates the data file's byte endianness.
    /// In most cases (and assumed by default), this is set to little-endian.
    /// Big endian is an edge case for certain target platforms (e.g. PS3 or Xbox 360).
    pub endianness: Endianness,

    /// Map of all chunks specified by `FORM`; indexed by chunk name.
    /// Read chunks will be removed from this `HashMap` when calling [`DataReader::read_chunk_required`] or [`DataReader::read_chunk`].
    /// May contain unknown chunks (if there is a GameMaker update, for example).
    pub chunks: Chunks,

    /// Metadata about the currently parsed chunk of data.
    /// This includes the chunk's name, start position, and end position within the data buffer.
    /// When reading data, these bounds are checked to ensure the read operation stays within the chunk.    
    ///
    /// **Safety Warning**: If the chunk's start/end positions are set incorrectly, the program becomes memory unsafe.
    pub chunk: ChunkBounds,

    /// The name of the last chunk in the data file.
    /// Is properly initialized after parsing `FORM`.
    pub last_chunk: ChunkName,

    /// General info about this data file. Includes game name, GameMaker Version and Bytecode Version.
    /// Contains garbage placeholders until the `GEN8` chunk is deserialized.
    /// Use [`DataReader::unstable_get_gm_version`] to get the GameMaker version before `GEN8` is parsed.
    pub general_info: GMGeneralInfo,

    /// Will be set after chunk `STRG` is parsed (first chunk to parse).
    /// Contains all GameMaker strings by ID (aka index)
    /// Needed for String references in Push Instructions.
    pub strings: Vec<String>,

    /// Chunk `STRG`.
    /// Is properly initialized after parsing `FORM`.
    pub string_chunk: ChunkBounds,

    /// Should only be set by [`crate::gamemaker::elements::texture_page_items`].
    /// This means that `TPAG` has to be parsed before any chunk with texture page item pointers.
    pub texture_page_item_occurrences: HashMap<u32, GMRef<GMTexturePageItem>>,

    /// Should only be set by [`crate::gamemaker::elements::variables`].
    /// This means that `VARI` has to be parsed before `CODE`.
    pub variable_occurrences: HashMap<u32, GMRef<GMVariable>>,

    /// Should only be set by [`crate::gamemaker::elements::functions`].
    /// This means that `FUNC` has to be parsed before `CODE`.
    pub function_occurrences: HashMap<u32, GMRef<GMFunction>>,
}

impl<'a> DataReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        // Memory Safety Assertion. This should've been verfied before, though.
        let end_pos: u32 = data
            .len()
            .try_into()
            .expect("Data length out of u32 bounds");

        Self {
            data,
            cur_pos: 0,
            specified_version: GMVersion::stub(),
            // The default padding value is 16, if used.
            chunk_padding: 16,
            // Assume little endian; big endian is an edge case.
            endianness: Endianness::Little,
            chunk: ChunkBounds { start_pos: 0, end_pos },
            last_chunk: ChunkName::new("XXXX"),
            // Just a stub, will not be read until GEN8 is parsed.
            general_info: GMGeneralInfo::default(),
            strings: vec![],
            string_chunk: ChunkBounds::default(),
            chunks: Chunks::new(),
            texture_page_item_occurrences: HashMap::new(),
            variable_occurrences: HashMap::new(),
            function_occurrences: HashMap::new(),
        }
    }

    /// The size / byte length of the data file.
    pub const fn size(&self) -> u32 {
        self.data.len() as u32
    }

    /// Read the specified number of bytes from the data file while advancing the data position.
    /// Returns an error when trying to read out of chunk bounds.
    pub fn read_bytes_dyn(&mut self, count: u32) -> Result<&'a [u8]> {
        let start: u32 = self.cur_pos;
        let end: u32 = self
            .cur_pos
            .checked_add(count)
            .ok_or("Trying to read out of u32 bounds")?;

        // Lower chunk bounds check
        if start < self.chunk.start_pos {
            bail!(
                "Trying to read {} bytes out of lower chunk bounds at position {} with start position {}",
                count,
                self.cur_pos,
                self.chunk.start_pos,
            );
        }

        // Upper chunk bounds check
        if end > self.chunk.end_pos {
            bail!(
                "Trying to read {} bytes out of upper chunk bounds at position {} with end position {}",
                count,
                self.cur_pos,
                self.chunk.end_pos,
            );
        }

        #[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
        compile_error!(
            "Cannot safely convert u32 to usize on this platform (target pointer width not 32 or 64)"
        );

        // SAFETY: If chunk.end_pos is set correctly, this should never read memory out of bounds.

        let start = start as usize;
        let end = end as usize;
        let slice: &[u8] = unsafe { self.data.get_unchecked(start..end) };
        self.cur_pos += count;
        Ok(slice)
    }

    /// Read a constant number of bytes from the data file while advancing the data position.
    /// Useful for reading slices with specified sizes like `[u8; 16]`.
    ///
    /// **Safety Note:** `N` must be less than `u32::MAX`.
    ///
    /// (TODO: Implement const assertion when rust supports it.)
    pub fn read_bytes_const<const N: usize>(&mut self) -> Result<&[u8; N]> {
        let slice: &[u8] = self.read_bytes_dyn(N as u32)?;
        // SAFETY: read_bytes_dyn is guaranteed to read exact N bytes.
        // > EXCEPTION: This produces undefined behavior is if N > u32::MAX.
        Ok(unsafe { &*slice.as_ptr().cast::<[u8; N]>() })
    }

    /// Read a 32-bit integer and convert it to a bool.
    /// ___
    /// Returns an error when the read number is neither 0 nor 1.
    pub fn read_bool32(&mut self) -> Result<bool> {
        let number = self.read_u32()?;
        match number {
            0 => Ok(false),
            1 => Ok(true),
            n => bail!(
                "Read invalid boolean value {n} (0x{n:08X}) at position {}",
                self.cur_pos,
            ),
        }
    }

    /// Read a UTF-8 character string with the specified byte length.
    /// ___
    /// For reading standard GameMaker string references, see [`DataReader::read_gm_string`].
    pub fn read_literal_string(&mut self, length: u32) -> Result<String> {
        let bytes: Vec<u8> = self
            .read_bytes_dyn(length)
            .with_context(|| format!("reading literal string with length {length}"))?
            .to_vec();

        let string: String = String::from_utf8(bytes)
            .map_err(|e| e.to_string())
            .with_context(|| {
                format!(
                    "parsing literal UTF-8 string with length {} at position {}",
                    length,
                    self.cur_pos - length,
                )
            })?;

        Ok(string)
    }

    /// Gets the length of the chunk that is being currently parsed.
    pub const fn get_chunk_length(&self) -> u32 {
        self.chunk.end_pos - self.chunk.start_pos
    }

    /// Read bytes until the reader position is divisible by the specified alignment.
    /// Ensures the read padding bytes are all zero.
    pub fn align(&mut self, alignment: u32) -> Result<()> {
        while !self.cur_pos.is_multiple_of(alignment) {
            let byte = self.read_u8()?;
            assert_int("Padding Byte", 0, byte)
                .with_context(|| format!("aligning reader to {alignment}"))?;
        }
        Ok(())
    }

    /// Ensures the reader is at the specified position.
    pub fn assert_pos(&self, position: u32, pointer_name: &str) -> Result<()> {
        if self.cur_pos != position {
            if position == 0 {
                bail!(
                    "{} pointer is zero at position {}! Null pointers are not yet supported.",
                    pointer_name,
                    self.cur_pos,
                )
            }
            bail!(
                "{} pointer misaligned: expected position {} but reader is actually at {} (diff: {})",
                pointer_name,
                position,
                self.cur_pos,
                i64::from(position) - i64::from(self.cur_pos),
            )
        }
        Ok(())
    }

    /// Sets the reader position to the current chunk's start position plus the specified relative position.
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

    /// Deserializes an element if the GameMaker version meets the requirement (`>=`).
    ///
    /// This is useful for handling format changes across different GameMaker versions
    /// where certain chunks or fields were added, removed, or modified.
    ///
    /// # Returns
    /// - `Ok(Some(T))` if the version requirement is met and deserialization succeeds
    /// - `Ok(None)` if the version requirement is not met
    /// - `Err(_)` if the version requirement is met but deserialization fails
    pub fn deserialize_if_gm_version<T: GMElement, V: Into<GMVersionReq>>(
        &mut self,
        ver_req: V,
    ) -> Result<Option<T>> {
        if self.general_info.is_version_at_least(ver_req) {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }

    /// Deserializes an element if the bytecode version meets the requirement (`>=`).
    ///
    /// Bytecode version is separate from the GameMaker IDE version and tracks
    /// changes to the virtual machine instruction format.
    ///
    /// # Returns
    /// - `Ok(Some(T))` if the bytecode version requirement is met and deserialization succeeds
    /// - `Ok(None)` if the bytecode version requirement is not met
    /// - `Err(_)` if the bytecode version requirement is met but deserialization fails
    pub fn deserialize_if_bytecode_version<T: GMElement>(
        &mut self,
        ver_req: u8,
    ) -> Result<Option<T>> {
        if self.general_info.bytecode_version >= ver_req {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }
}
