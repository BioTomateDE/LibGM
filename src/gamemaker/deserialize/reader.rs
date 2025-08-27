use std::collections::HashMap;
use crate::gamemaker::deserialize::chunk::GMChunk;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::general_info::GMGeneralInfo;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::variables::{GMVariable, GMVariables};
use crate::gamemaker::gm_version::GMVersionReq;
use crate::utility::format_bytes;


pub struct DataReader<'a> {
    /// The raw data buffer that is being parsed.
    data: &'a [u8],
    
    /// The current read position within the data buffer.
    /// Reading data will be read from this position; incrementing it.
    pub cur_pos: usize,
    
    /// How many bytes of padding are/should be at the end of every chunk.
    /// Only relevant in certain GameMaker versions.
    /// Defaults to 16, but will be set to 4 or 1 if detected.
    pub chunk_padding: usize,

    /// Indicates whether the data is formatted using big-endian byte order.    
    /// This applies only to certain target platforms that require big endian encoding (e.g. PS3 or Xbox 360).
    pub is_big_endian: bool,

    /// Map of all chunks specified by `FORM`; indexed by chunk name.
    /// Read chunks will be removed from this HashMap when calling [`DataReader::read_chunk_required`] or [`DataReader::read_chunk_optional`].
    /// May contain unknown chunks (if there is a GameMaker update, for example).
    pub chunks: HashMap<String, GMChunk>,

    /// Metadata about the currently parsed chunk of data.
    /// This includes the chunk's name, start position, and end position within the data buffer.
    /// When reading data, these bounds are checked to ensure the read operation stays within the chunk.    
    /// 
    /// **Safety Warning**: If the chunk's start or end position are set incorrectly, the program becomes memory unsafe.
    pub chunk: GMChunk,

    /// Contains garbage placeholders until the `GEN8` chunk is deserialized.
    /// Use [`DataReader::unstable_get_gm_version`] in [`crate::gamemaker::deserialize::chunk`]
    /// to get the GameMaker version before GEN8 is parsed.
    pub general_info: GMGeneralInfo,
    
    /// Will be set after chunk STRG is parsed (first chunk to parse).
    /// Contains all GameMaker strings, which are needed to resolve strings while deserialization.
    pub strings: GMStrings,

    /// Should only be set by [`crate::gamemaker::elements::strings`].
    pub string_occurrence_map: HashMap<usize, GMRef<String>>,

    /// Should only be set by [`crate::gamemaker::elements::texture_page_items`].
    /// This means that TPAG has to be parsed before any chunk with texture page item pointers.
    pub texture_page_item_occurrence_map: HashMap<usize, GMRef<GMTexturePageItem>>,

    /// Should only be set by [`crate::gamemaker::elements::variables`].
    /// This means that VARI has to be parsed before CODE.
    pub variable_occurrence_map: HashMap<usize, GMRef<GMVariable>>,

    /// Should only be set by [`crate::gamemaker::elements::functions`].
    /// This means that VARI has to be parsed before CODE.
    pub function_occurrence_map: HashMap<usize, GMRef<GMFunction>>,
}

impl<'a> DataReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            general_info: GMGeneralInfo::empty(),
            strings: GMStrings::empty(),
            chunks: HashMap::with_capacity(35),
            chunk: GMChunk {
                name: "FORM".to_string(),
                start_pos: 0,
                end_pos: data.len(),
                is_last_chunk: true,
            },
            data,
            cur_pos: 0,
            chunk_padding: 16,    // default padding value (if used) is 16
            string_occurrence_map: HashMap::new(),
            texture_page_item_occurrence_map: HashMap::new(),
            variable_occurrence_map: HashMap::new(),
            function_occurrence_map: HashMap::new(),
            is_big_endian: false,   // assume little endian; big endian is an edge case
        }
    }

    pub fn read_bytes_dyn(&mut self, count: usize) -> Result<&'a [u8], String> {
        // combined check to hopefully increase performance
        if !(self.chunk.start_pos <= self.cur_pos && self.cur_pos+count <= self.chunk.end_pos) {
            return if self.cur_pos < self.chunk.start_pos {
                Err(format!(
                    "out of lower bounds at position {} in chunk '{}' with start position {}",
                    self.cur_pos, self.chunk.name, self.chunk.start_pos,
                ))
            } else {
                Err(format!(
                    "out of upper bounds at position {} in chunk '{}': {} > {}",
                    self.cur_pos, self.chunk.name, self.cur_pos+count, self.chunk.end_pos,
                ))
            }
        }
        // SAFETY: If chunk.start_pos and chunk.end_pos are set correctly; this should never read memory out of bounds.
        let slice: &[u8] = unsafe { self.data.get_unchecked(self.cur_pos..self.cur_pos + count) };
        self.cur_pos += count;
        Ok(slice)
    }
    
    pub fn read_bytes_const<const N: usize>(&mut self) -> Result<&[u8; N], String> {
        let slice: &[u8] = self.read_bytes_dyn(N)?;
        // read_bytes_dyn is guaranteed to read N bytes so the unwrap never fails.
        Ok(unsafe { &*(slice.as_ptr() as *const [u8; N]) })
    }


    /// Read unsigned 32-bit integer and convert to usize (little endian).
    /// Meant for reading positions/pointers; uses total data length as failsafe.
    pub fn read_pointer(&mut self) -> Result<usize, String> {
        let failsafe_amount: usize = self.data.len();
        let number: usize = self.read_usize()?;
        if number >= failsafe_amount {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying to read usize \
                (pointer) integer: Number {} ({}) is larger than the total data length of {} ({})",
                self.chunk.name, self.cur_pos-4, number, format_bytes(number), failsafe_amount, format_bytes(failsafe_amount),
            ))
        }
        Ok(number)
    }

    /// Read a 32-bit integer and convert it to a bool.
    /// ___
    /// Returns `Err<String>` when the read number is neither 0 nor 1.
    pub fn read_bool32(&mut self) -> Result<bool, String> {
        let number: u32 = self.read_u32()?;
        match number {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!(
                "Read invalid boolean value in chunk '{0}' at position {1}: {2} (0x{2:08X})",
                self.chunk.name, self.cur_pos, number,
            ))
        }
    }

    /// Read a UTF-8 character string with the specified byte length.
    /// ___
    /// For reading standard GameMaker string references, see [`DataReader::read_gm_string`].
    pub fn read_literal_string(&mut self, length: usize) -> Result<String, String> {
        let bytes: &[u8] = self.read_bytes_dyn(length)
            .map_err(|e| format!("Trying to read literal string with length {length} {e}"))?;
        let string: String = String::from_utf8(bytes.to_vec()).map_err(|e| format!(
            "Could not parse literal string with length {} in chunk '{}' at position {}: {e}",
            length, self.chunk.name, self.cur_pos,
        ))?;
        Ok(string)
    }

    /// Gets the length of the chunk that is being currently parsed.
    pub fn get_chunk_length(&self) -> usize {
        self.chunk.end_pos - self.chunk.start_pos
    }
    
    pub fn get_data_length(&self) -> usize {
        self.data.len()
    }

    /// Read bytes until the reader position is divisible by the specified alignment.
    /// Ensures the read padding bytes are all zero.
    pub fn align(&mut self, alignment: usize) -> Result<(), String> {
        while self.cur_pos % alignment != 0 {
            let byte: u8 = self.read_u8()?;
            if byte != 0 {
                return Err(format!("Invalid padding byte while aligning to {alignment}: expected zero but got {byte}"))
            }
        }
        Ok(())
    }

    /// Ensures the reader is at the specified position.
    pub fn assert_pos(&self, position: usize, pointer_name: &str) -> Result<(), String> {
        if self.cur_pos != position {
            return Err(format!(
                "{} pointer misaligned: expected position {} but reader is actually at {} (diff: {})",
                pointer_name, position, self.cur_pos, position as i64 - self.cur_pos as i64,
            ))
        }
        Ok(())
    }

    /// Sets the reader position to the current chunk's start position plus the specified relative position.
    pub fn set_rel_cur_pos(&mut self, relative_position: usize) -> Result<(), String> {
        if self.chunk.start_pos + relative_position > self.chunk.end_pos {
            return Err(format!(
                "Tried to set relative reader position to {} in chunk '{}' with start position {} and end position {}; out of bounds",
                relative_position, self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
            ))
        }
        self.cur_pos = self.chunk.start_pos + relative_position;
        Ok(())
    }

    /// Gets the reader position relative to the current chunk's start position.
    pub fn get_rel_cur_pos(&self) -> usize {
        self.cur_pos - self.chunk.start_pos
    }

    /// If the GameMaker version requirement is met, deserializes the element and returns it.
    /// Otherwise, just returns [`None`].
    pub fn deserialize_if_gm_version<T: GMElement, V: Into<GMVersionReq>>(&mut self, ver_req: V) -> Result<Option<T>, String> {
        if self.general_info.is_version_at_least(ver_req) {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }
    
    /// If the Bytecode version requirement is met, deserializes the element and returns it.
    /// Otherwise, just returns [`None`].
    pub fn deserialize_if_bytecode_version<T: GMElement>(&mut self, ver_req: u8) -> Result<Option<T>, String> {
        if self.general_info.bytecode_version >= ver_req {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }
}

