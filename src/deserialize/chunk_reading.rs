use crate::debug_utils::{format_bytes, likely, typename, unlikely};
use crate::deserialize::strings::GMStrings;

// GMRef is for parsing chunks:
// It has (fake) generic types to make it clearer
// which type it belongs to (`name: GMRef` vs `name: GMRef<String>`).
// It can be resolved to the data it references using the `.resolve()` method,
// which needs the list the elements are stored in.
// [See GMPointer to understand difference]
#[derive(Hash, PartialEq, Eq)]
pub struct GMRef<T> {
    pub index: usize,
    // marker needs to be here to ignore "unused generic T" error; doesn't store any data
    _marker: std::marker::PhantomData<T>,
}
impl<T> Clone for GMRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for GMRef<T> {}
impl<T> std::fmt::Debug for GMRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMRef<{}#{}>", typename::<T>(), self.index)
    }
}
impl<T> GMRef<T> {
    pub fn new(index: usize) -> GMRef<T> {
        Self {
            index,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<'a, T> GMRef<T> {
    pub fn resolve(&self, elements_by_index: &'a Vec<T>) -> Result<&'a T, String> {
        elements_by_index.get(self.index)
            .ok_or_else(|| format!(
                "Could not resolve {} reference with index {} in list with length {}",
                std::any::type_name::<T>(),
                self.index,
                elements_by_index.len(),
            ))
    }
}


#[derive(Debug, Clone)]
pub struct GMChunk<'a> {
    pub name: String,           // 4 letter name of chunk
    pub abs_pos: usize,         // absolute position/index in data.win file
    pub data: &'a [u8],         // raw data
    pub cur_pos: usize,         // gets incremented by .read_{} methods when parsing chunk
    pub total_data_len: usize,  // used for read_usize failsafe
}

impl<'a> GMChunk<'a> {
    pub fn read_bytes_dyn(&mut self, count: usize) -> Result<&'a [u8], String> {
        // if self.cur_pos+count > self.data.len() {
        //     log::error!("this is only here for easy breakpoints; comment out this if statement otherwise")
        // }
        let slice: &[u8] = self.data.get(self.cur_pos..self.cur_pos+count).ok_or_else(|| format!(
            "out of bounds at absolute position {} in chunk '{}': {} > {}",
            self.cur_pos+self.abs_pos, self.name, self.cur_pos+self.abs_pos+count, self.data.len(),
        ))?;
        self.cur_pos += count;
        Ok(slice)
    }
    pub fn read_bytes_const<const N: usize>(&mut self) -> Result<&'a [u8; N], String> {
        let slice: &[u8] = self.read_bytes_dyn(N)?;
        Ok(slice.try_into().unwrap())
    }

    pub fn read_u64(&mut self) -> Result<u64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read u64 {e}"))?;
        Ok(u64::from_le_bytes(*bytes))
    }
    pub fn read_i64(&mut self) -> Result<i64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read i64 {e}"))?;
        Ok(i64::from_le_bytes(*bytes))
    }
    pub fn read_u32(&mut self) -> Result<u32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read u32 {e}"))?;
        Ok(u32::from_le_bytes(*bytes))
    }
    pub fn read_i32(&mut self) -> Result<i32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read i32 {e}"))?;
        Ok(i32::from_le_bytes(*bytes))
    }
    pub fn read_u16(&mut self) -> Result<u16, String> {
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read u16 {e}"))?;
        Ok(u16::from_le_bytes(*bytes))
    }
    pub fn read_i16(&mut self) -> Result<i16, String> {
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read i16 {e}"))?;
        Ok(i16::from_le_bytes(*bytes))
    }
    pub fn read_u8(&mut self) -> Result<u8, String> {
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read u8 {e}"))?;
        Ok(u8::from_le_bytes(*bytes))
    }
    pub fn read_i8(&mut self) -> Result<i8, String> {
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read i8 {e}"))?;
        Ok(i8::from_le_bytes(*bytes))
    }

    pub fn read_f64(&mut self) -> Result<f64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read f64 {e}"))?;
        Ok(f64::from_le_bytes(*bytes))
    }
    pub fn read_f32(&mut self) -> Result<f32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read f32 {e}"))?;
        Ok(f32::from_le_bytes(*bytes))
    }

    fn read_usize_internal(&mut self) -> Result<usize, String> {
        let number: u32 = self.read_u32()?;
        Ok(number as usize)
    }
    /// Read unsigned 32-bit integer and convert to usize (little endian).
    /// Meant for reading positions/pointers; uses total data length as failsafe.
    pub fn read_usize_pos(&mut self) -> Result<usize, String> {
        let failsafe_amount: usize = self.total_data_len;
        let number: usize = self.read_usize_internal()?;
        if likely(number < failsafe_amount) {
            return Ok(number)
        }
        Err(format!(
            "Failsafe triggered in chunk '{}' at position {} while trying to read usize \
            (pointer) integer: Number {} ({}) is larger than the total data length of {} ({})",
            self.name, self.cur_pos-4, number, format_bytes(number), failsafe_amount, format_bytes(failsafe_amount),
        ))
    }
    /// Read unsigned 32-bit integer and convert to usize (little endian).
    /// Meant for reading (pointer list element) count; uses small constant number as failsafe.
    pub fn read_usize_count(&mut self) -> Result<usize, String> {
        const FAILSAFE_AMOUNT: usize = 100_000;    // increase limit is not enough
        let number: usize = self.read_usize_internal()?;
        if likely(number < FAILSAFE_AMOUNT) {
            return Ok(number)
        }
        Err(format!(
            "Failsafe triggered in chunk '{}' at position {} while trying \
            to read usize (count) integer: Number {} is larger than the failsafe count of {}",
            self.name, self.cur_pos-4, number, FAILSAFE_AMOUNT,
        ))
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
                self.name, self.cur_pos, number,
            ))
        }
    }

    pub fn read_literal_string(&mut self, length: usize) -> Result<String, String> {
        let bytes: &[u8] = self.read_bytes_dyn(length)
            .map_err(|e| format!("Trying to read literal string with length {length} {e}"))?;
        let string: String = String::from_utf8(bytes.to_vec()).map_err(|e| format!(
            "Could not parse literal string with length {} in chunk '{}' at position {}: {e}",
            length, self.name, self.cur_pos,
        ))?;
        Ok(string)
    }
    
    /// Read chunk name (4 ascii characters)
    pub fn read_chunk_name(&mut self) -> Result<String, String> {
        if unlikely(self.abs_pos != 0) {
            return Err(format!(
                "Reading a chunk name outside of the special \"all chunk\" isn't \
                allowed (chunk is called '{}' and has abs pos {})", self.name, self.abs_pos,
            ))
        }
        let string: String = self.read_literal_string(4)
            .map_err(|e| if self.abs_pos == 0 && self.cur_pos == 4 {
                "Invalid data.win file; data doesn't start with 'FORM' string".to_string()
            } else {
                format!("Could not parse chunk name at position {}: {e}", self.cur_pos)
            })?;
        if unlikely(string.len() != 4 || !string.is_ascii()) {  // can happen because of unicode
            return Err(format!("Chunk name string \"{string}\" has length {} (chunk names need to be 4 ascii chars long)", string.len()))
        }
        Ok(string)
    }

    pub fn read_gm_string(&mut self, gm_strings: &GMStrings) -> Result<GMRef<String>, String> {
        let string_abs_pos: usize = self.read_usize_pos()?;
        if gm_strings.abs_pos_to_reference.get(&string_abs_pos).is_none() {
            log::error!("this is only here for easy breakpoints; comment out this if statement otherwise")
        }
        let string_ref = gm_strings.abs_pos_to_reference.get(&string_abs_pos)
            .ok_or_else(|| format!(
                "Could not read reference string with absolute position {} in chunk '{}' at \
                absolute position {} because it doesn't exist in the string map (length: {})",
                string_abs_pos, self.name, self.abs_pos + self.cur_pos - 4, gm_strings.abs_pos_to_reference.len(),
            ))?;
        Ok(string_ref.clone())
    }

    /// Try to read a GM String Reference. If the value is zero, return None.
    pub fn read_gm_string_optional(&mut self, gm_strings: &GMStrings) -> Result<Option<GMRef<String>>, String> {
        let string_abs_pos: usize = self.read_usize_pos()?;
        let string_ref: Option<GMRef<String>> = gm_strings.abs_pos_to_reference.get(&string_abs_pos).cloned();
        Ok(string_ref)
    }

    /// read pointer to pointer list (only used in rooms)
    pub fn read_pointer_to_pointer_list(&mut self) -> Result<Vec<usize>, String> {
        let abs_pointers_start_pos: usize = self.read_usize_pos()?;
        let pointers_start_pos: usize = abs_pointers_start_pos.checked_sub(self.abs_pos).ok_or_else(|| format!(
            "Pointer to start of Pointer list underflowed at position {} in chunk '{}': {} - {} < 0",
            self.cur_pos - 4, self.name, abs_pointers_start_pos, self.abs_pos,
        ))?;

        let old_position: usize = self.cur_pos;
        self.cur_pos = pointers_start_pos;

        let pointer_count: usize = self.read_usize_count()?;
        let mut pointers: Vec<usize> = Vec::with_capacity(pointer_count);
        for _ in 0..pointer_count {
            let abs_pointer_pos: usize = self.read_usize_pos()?;
            let pointer: usize = abs_pointer_pos.checked_sub(self.abs_pos).ok_or_else(|| format!(
                "Element of Pointer list underflowed at position {} in chunk '{}': {} - {} < 0",
                self.cur_pos - 4, self.name, abs_pointer_pos, self.abs_pos,
            ))?;
            pointers.push(pointer);
        }

        self.cur_pos = old_position;
        Ok(pointers)
    }

    pub fn align(&mut self, alignment: usize) -> Result<(), String> {
        while (self.cur_pos + self.abs_pos) & (alignment - 1) != 0 {
            self.read_u8()?;
        }
        Ok(())
    }
}


