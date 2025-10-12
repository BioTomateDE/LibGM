use crate::gamemaker::data::Endianness;
use crate::gamemaker::deserialize::chunk::GMChunk;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::general_info::GMGeneralInfo;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::variables::GMVariable;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::GMVersionReq;
use crate::prelude::*;
use crate::{integrity_assert, integrity_check};
use std::collections::HashMap;

pub struct DataReader<'a> {
    /// The raw data buffer that is being parsed.
    data: &'a [u8],

    /// The current read position within the data buffer.
    /// Reading data will be read from this position; incrementing it.
    pub cur_pos: u32,

    /// How many bytes of padding are/should be at the end of every chunk.
    /// Only relevant in certain GameMaker versions.
    /// Defaults to 16, but will be set to 4 or 1 if detected.
    pub chunk_padding: u32,

    /// Indicates the data's byte endianness.
    /// In most cases (and assumed by default), this is set to little-endian.
    /// Big endian is an edge case for certain target platforms (e.g. PS3 or Xbox 360).
    pub endianness: Endianness,

    /// Map of all chunks specified by `FORM`; indexed by chunk name.
    /// Read chunks will be removed from this HashMap when calling [`DataReader::read_chunk_required`] or [`DataReader::read_chunk_optional`].
    /// May contain unknown chunks (if there is a GameMaker update, for example).
    pub chunks: HashMap<String, GMChunk>,

    /// Metadata about the currently parsed chunk of data.
    /// This includes the chunk's name, start position, and end position within the data buffer.
    /// When reading data, these bounds are checked to ensure the read operation stays within the chunk.    
    ///
    /// **Safety Warning**: If the chunk's start/end positions are set incorrectly, the program becomes memory unsafe.
    pub chunk: GMChunk,

    /// Contains garbage placeholders until the `GEN8` chunk is deserialized.
    /// Use [`DataReader::unstable_get_gm_version`] in [`crate::gamemaker::deserialize::chunk`]
    /// to get the GameMaker version before GEN8 is parsed.
    pub general_info: GMGeneralInfo,

    /// Will be set after chunk STRG is parsed (first chunk to parse).
    /// Contains all GameMaker strings, which are needed to resolve strings while deserialization.
    pub strings: GMStrings,

    /// Should only be set by [`crate::gamemaker::elements::strings`].
    /// This means that STRG has to be parsed before any other chunk.
    pub string_occurrence_map: HashMap<u32, GMRef<String>>,

    /// Should only be set by [`crate::gamemaker::elements::texture_page_items`].
    /// This means that TPAG has to be parsed before any chunk with texture page item pointers.
    pub texture_page_item_occurrence_map: HashMap<u32, GMRef<GMTexturePageItem>>,

    /// Should only be set by [`crate::gamemaker::elements::variables`].
    /// This means that VARI has to be parsed before CODE.
    pub variable_occurrence_map: HashMap<u32, GMRef<GMVariable>>,

    /// Should only be set by [`crate::gamemaker::elements::functions`].
    /// This means that FUNC has to be parsed before CODE.
    pub function_occurrence_map: HashMap<u32, GMRef<GMFunction>>,
}

impl<'a> DataReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            general_info: GMGeneralInfo::stub(),
            strings: GMStrings::stub(),
            chunks: HashMap::with_capacity(35),
            chunk: GMChunk {
                name: "FORM".to_string(),
                start_pos: 0,
                end_pos: data.len() as u32,
                is_last_chunk: true,
            },
            data,
            cur_pos: 0,
            chunk_padding: 16, // Default padding value (if used) is 16
            string_occurrence_map: HashMap::new(),
            texture_page_item_occurrence_map: HashMap::new(),
            variable_occurrence_map: HashMap::new(),
            function_occurrence_map: HashMap::new(),
            endianness: Endianness::Little, // Assume little endian; big endian is an edge case
        }
    }

    /// Read the specified number of bytes from the data file while advancing the data position.
    /// Returns an error when trying to read out of chunk bounds.
    pub fn read_bytes_dyn(&mut self, count: u32) -> Result<&'a [u8]> {
        if self.cur_pos < self.chunk.start_pos {
            bail!(
                "Trying to read {} bytes out of lower chunk bounds at position {} in chunk '{}' with start position {}",
                count,
                self.cur_pos,
                self.chunk.name,
                self.chunk.start_pos,
            );
        }

        if self.cur_pos + count > self.chunk.end_pos {
            bail!(
                "Trying to read {} bytes out of upper chunk bounds at position {} in chunk '{}' with end position {}",
                count,
                self.cur_pos,
                self.chunk.name,
                self.chunk.end_pos,
            );
        }

        // SAFETY: If chunk.start_pos and chunk.end_pos are set correctly; this should never read memory out of bounds.
        let start = self.cur_pos as usize;
        let end = (self.cur_pos + count) as usize;
        let slice: &[u8] = unsafe { self.data.get_unchecked(start..end) };
        self.cur_pos += count;
        Ok(slice)
    }

    /// Read a constant number of bytes from the data file while advancing the data position.
    /// Useful for reading slices with specified sizes like `[u8; 16]`.
    pub fn read_bytes_const<const N: usize>(&mut self) -> Result<&[u8; N]> {
        let slice: &[u8] = self.read_bytes_dyn(N as u32)?;
        // SAFETY: read_bytes_dyn is guaranteed to read exact N bytes.
        Ok(unsafe { &*(slice.as_ptr() as *const [u8; N]) })
    }

    /// Read a 32-bit integer and convert it to a bool.
    /// ___
    /// Returns an error when the read number is neither 0 nor 1.
    pub fn read_bool32(&mut self) -> Result<bool> {
        let number = self.read_u32()?;
        match number {
            0 => Ok(false),
            1 => Ok(true),
            _ => bail!(
                "Read invalid boolean value {} (0x{:08X}) in chunk '{}' at position {}",
                number,
                number,
                self.chunk.name,
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
        let string: String = String::from_utf8(bytes).with_context(|| {
            format!(
                "parsing literal UTF-8 string with length {} in chunk '{}' at position {}",
                length, self.chunk.name, self.cur_pos,
            )
        })?;
        Ok(string)
    }

    /// Gets the length of the chunk that is being currently parsed.
    pub fn get_chunk_length(&self) -> u32 {
        self.chunk.end_pos - self.chunk.start_pos
    }

    /// Read bytes until the reader position is divisible by the specified alignment.
    /// Ensures the read padding bytes are all zero.
    pub fn align(&mut self, alignment: u32) -> Result<()> {
        while self.cur_pos % alignment != 0 {
            let byte = self.read_u8()?;
            integrity_assert! {
                byte == 0,
                "Invalid padding byte while aligning to {alignment}: expected zero but got {byte} (0x{byte:02X})"
            }
        }
        Ok(())
    }

    /// Ensures the reader is at the specified position.
    pub fn assert_pos(&self, position: u32, pointer_name: &str) -> Result<()> {
        integrity_check! {
            if self.cur_pos != position {
                if position == 0 {
                    bail!(
                        "{} pointer is zero at position {}! Null pointers are not yet supported.",
                        pointer_name, self.cur_pos,
                    )
                }
                bail!(
                    "{} pointer misaligned: expected position {} but reader is actually at {} (diff: {})",
                    pointer_name, position, self.cur_pos, position as i64 - self.cur_pos as i64,
                )
            }
        }
        Ok(())
    }

    /// Sets the reader position to the current chunk's start position plus the specified relative position.
    pub fn set_rel_cur_pos(&mut self, relative_position: u32) -> Result<()> {
        if self.chunk.start_pos + relative_position > self.chunk.end_pos {
            bail!(
                "Tried to set relative reader position to {} in chunk '{}' with start position {} and end position {}; out of bounds",
                relative_position,
                self.chunk.name,
                self.chunk.start_pos,
                self.chunk.end_pos,
            )
        }
        self.cur_pos = self.chunk.start_pos + relative_position;
        Ok(())
    }

    /// Gets the reader position relative to the current chunk's start position.
    pub fn get_rel_cur_pos(&self) -> u32 {
        self.cur_pos - self.chunk.start_pos
    }

    /// If the GameMaker version requirement is met, deserializes the element and returns it.
    /// Otherwise, just returns [`None`].
    pub fn deserialize_if_gm_version<T: GMElement, V: Into<GMVersionReq>>(&mut self, ver_req: V) -> Result<Option<T>> {
        if self.general_info.is_version_at_least(ver_req) {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }

    /// If the Bytecode version requirement is met, deserializes the element and returns it.
    /// Otherwise, just returns [`None`].
    pub fn deserialize_if_bytecode_version<T: GMElement>(&mut self, ver_req: u8) -> Result<Option<T>> {
        if self.general_info.bytecode_version >= ver_req {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }
}
