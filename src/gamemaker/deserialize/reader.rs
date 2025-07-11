use std::collections::HashMap;
use crate::gamemaker::deserialize::chunk::GMChunk;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::general_info::GMGeneralInfo;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::variables::GMVariable;
use crate::gamemaker::gm_version::GMVersionReq;
use crate::utility::format_bytes;

pub struct DataReader<'a> {
    data: &'a [u8],
    pub cur_pos: usize,
    pub chunk_padding: usize,

    /// Indicates whether the data is formatted using big-endian byte order.    
    /// This applies only to certain target platforms that require big-endian encoding (e.g. PS3 or Xbox 360).
    pub is_big_endian: bool,

    pub chunks: HashMap<String, GMChunk>,
    pub chunk: GMChunk,

    /// Should not be read until GEN8 chunk is parsed
    pub general_info: GMGeneralInfo,
    /// Should only be set by `gamemaker::string`
    pub strings: GMStrings,

    /// Should only be set by `gamemaker::strings::GMStrings`
    pub string_occurrence_map: HashMap<usize, GMRef<String>>,
    /// Should only be set by `gamemaker::texture_page_items::GMTexturePageItems`
    pub texture_page_item_occurrence_map: HashMap<usize, GMRef<GMTexturePageItem>>,
    /// Should only be set by `gamemaker::variables::GMVariables`
    pub variable_occurrence_map: HashMap<usize, GMRef<GMVariable>>,
    /// Should only be set by `gamemaker::functions::GMFunctions`
    pub function_occurrence_map: HashMap<usize, GMRef<GMFunction>>,
}

impl<'a> DataReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            general_info: GMGeneralInfo::empty(),
            strings: GMStrings::empty(),
            chunks: HashMap::with_capacity(24),
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
            is_big_endian: false,   // assume little endian
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
        // if chunk.start_pos and chunk.end_pos are set correctly; this should never read memory out of bounds.
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
    /// Automatically subtracts `chunks.abs_pos`; converting it to a relative chunk position.
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

    pub fn read_literal_string(&mut self, length: usize) -> Result<String, String> {
        let bytes: &[u8] = self.read_bytes_dyn(length)
            .map_err(|e| format!("Trying to read literal string with length {length} {e}"))?;
        let string: String = String::from_utf8(bytes.to_vec()).map_err(|e| format!(
            "Could not parse literal string with length {} in chunk '{}' at position {}: {e}",
            length, self.chunk.name, self.cur_pos,
        ))?;
        Ok(string)
    }

    pub fn get_chunk_length(&self) -> usize {
        self.chunk.end_pos - self.chunk.start_pos
    }

    pub fn align(&mut self, alignment: usize) -> Result<(), String> {
        while self.cur_pos & (alignment - 1) != 0 {
            if self.cur_pos > self.chunk.end_pos {
                return Err(format!("Trying to align reader out of chunk bounds at position {}", self.cur_pos))
            }
            self.read_u8()?;
        }
        Ok(())
    }

    pub fn assert_pos(&self, position: usize, pointer_name: &str) -> Result<(), String> {
        if self.cur_pos != position {
            return Err(format!(
                "{} pointer misaligned: expected position {} but reader is actually at {} (diff: {})",
                pointer_name, position, self.cur_pos, position as i64 - self.cur_pos as i64,
            ))
        }
        Ok(())
    }

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

    pub fn get_rel_cur_pos(&self) -> usize {
        self.cur_pos - self.chunk.start_pos
    }

    pub fn deserialize_if_gm_version<T: GMElement, V: Into<GMVersionReq>>(&mut self, ver_req: V) -> Result<Option<T>, String> {
        if self.general_info.is_version_at_least(ver_req) {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }

    pub fn deserialize_if_bytecode_version<T: GMElement>(&mut self, ver_req: u8) -> Result<Option<T>, String> {
        if self.general_info.bytecode_version >= ver_req {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }
}
