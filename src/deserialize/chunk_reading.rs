use crate::debug_utils::{format_bytes, likely, typename};
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

impl GMChunk<'_> {
    pub fn read_u64(&mut self) -> Result<u64, String> {
        let bytes = self.data
            .get(self.cur_pos..self.cur_pos + 8)
            .ok_or_else(|| format!(
                "Trying to read u64 out of bounds in chunk '{}' at position {}: {} > {}",
                self.name, self.cur_pos, self.cur_pos + 8, self.data.len(),
            ))?;
        self.cur_pos += 8;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_i64(&mut self) -> Result<i64, String> {
        let bytes = self.data
            .get(self.cur_pos..self.cur_pos + 8)
            .ok_or_else(|| format!(
                "Trying to read i64 out of bounds in chunk '{}' at position {}: {} > {}",
                self.name, self.cur_pos, self.cur_pos + 8, self.data.len(),
            ))?;
        self.cur_pos += 8;
        Ok(i64::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_u32(&mut self) -> Result<u32, String> {
        let bytes = self.data
            .get(self.cur_pos..self.cur_pos + 4)
            .ok_or_else(|| format!(
                "Trying to read u32 out of bounds in chunk '{}' at position {}: {} > {}",
                self.name, self.cur_pos, self.cur_pos + 4, self.data.len(),
            ))?;
        self.cur_pos += 4;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_i32(&mut self) -> Result<i32, String> {
        let bytes = self.data
            .get(self.cur_pos..self.cur_pos + 4)
            .ok_or_else(|| format!(
                "Trying to read i32 out of bounds in chunk '{}' at position {}: {} > {}",
                self.name, self.cur_pos, self.cur_pos + 4, self.data.len(),
            ))?;
        self.cur_pos += 4;
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_u16(&mut self) -> Result<u16, String> {
        let bytes = self.data
            .get(self.cur_pos..self.cur_pos + 2)
            .ok_or_else(|| format!(
                "Trying to read u16 out of bounds in chunk '{}' at position {}: {} > {}",
                self.name, self.cur_pos, self.cur_pos + 2, self.data.len(),
            ))?;
        self.cur_pos += 2;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_i16(&mut self) -> Result<i16, String> {
        let bytes = self.data
            .get(self.cur_pos..self.cur_pos + 2)
            .ok_or_else(|| format!(
                "Trying to read i16 out of bounds in chunk '{}' at position {}: {} > {}",
                self.name, self.cur_pos, self.cur_pos + 2, self.data.len(),
            ))?;
        self.cur_pos += 2;
        Ok(i16::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_u8(&mut self) -> Result<u8, String> {
        let byte = *self.data
            .get(self.cur_pos)
            .ok_or_else(|| format!(
                "Trying to read u8 out of bounds in chunk '{}' at position {}",
                self.name, self.cur_pos,
            ))?;
        self.cur_pos += 1;
        Ok(byte)
    }
    
    pub fn read_i8(&mut self) -> Result<i8, String> {
        let byte = *self.data
            .get(self.cur_pos)
            .ok_or_else(|| format!(
                "Trying to read i8 out of bounds in chunk '{}' at position {}",
                self.name, self.cur_pos,
            ))?;
        self.cur_pos += 1;
        Ok(byte as i8)
    }

    fn read_usize(&mut self) -> Result<usize, String> {
        let number: u32 = self.read_u32()?;
        let number: usize = number as usize;
        Ok(number)
    }

    /// Read unsigned 32-bit integer and convert to usize (little endian).
    /// Meant for reading positions/pointers; uses total data length as failsafe.
    pub fn read_usize_pos(&mut self) -> Result<usize, String> {
        let failsafe_amount: usize = self.total_data_len;
        let number: usize = self.read_usize()?;

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
        let number: usize = self.read_usize()?;

        if likely(number < FAILSAFE_AMOUNT) {
            return Ok(number)
        }

        Err(format!(
            "Failsafe triggered in chunk '{}' at position {} while trying \
            to read usize (count) integer: Number {} is larger than the failsafe count of {}",
            self.name, self.cur_pos-4, number, FAILSAFE_AMOUNT,
        ))
    }

    pub fn read_usize_big_endian(&mut self, enable_failsafe: bool) -> Result<usize, String> {
        // Read unsigned 32-bit integer and convert to usize (big endian)
        static FAILSAFE_AMOUNT: usize = 200_000_000;

        let bytes: [u8; 4] = self.data.get(self.cur_pos.. self.cur_pos + 4)
            .ok_or_else(|| format!(
                "Trying to read big endian usize integer (u32) \
                out of bounds in chunk '{}' at position {}: {} > {}",
                self.name, self.cur_pos, self.cur_pos + 4, self.data.len(),
            ))?
            .try_into().unwrap();
        self.cur_pos += 4;

        let number: u32 = u32::from_be_bytes(bytes);
        let number: usize = number as usize;

        if !enable_failsafe || likely(number < FAILSAFE_AMOUNT) {
            return Ok(number);
        }
        Err(format!(
            "Failsafe triggered in chunk '{}' at position {} trying \
            to read big endian usize integer: Number {} is larger than failsafe amount {}",
            self.name, self.cur_pos - 4, number, FAILSAFE_AMOUNT,
        ))
    }

    pub fn read_f64(&mut self) -> Result<f64, String> {
        // Read a double-precision floating point number (little endian)
        let bytes: [u8; 8] = self.data.get(self.cur_pos .. self.cur_pos+8).ok_or_else(|| format!(
            "Trying to read f64 out of bounds in chunk '{}' at position {}: {} > {}",
            self.name, self.cur_pos, self.cur_pos + 8, self.data.len(),
        ))?.try_into().unwrap();

        let number: f64 = f64::from_le_bytes(bytes);
        self.cur_pos += 8;
        Ok(number)
    }

    pub fn read_f32(&mut self) -> Result<f32, String> {
        // Read a single-precision floating point number (little endian)
        let bytes: [u8; 4] = self.data.get(self.cur_pos .. self.cur_pos+4).ok_or_else(|| format!(
            "Trying to read f32 out of bounds in chunk '{}' at position {}: {} > {}",
            self.name, self.cur_pos, self.cur_pos + 4, self.data.len(),
        ))?.try_into().unwrap();

        let number: f32 = f32::from_le_bytes(bytes);
        self.cur_pos += 4;
        Ok(number)
    }
    
    pub fn read_bool32(&mut self) -> Result<bool, String> {
        // Read a 32-bit integer and convert it to a bool.
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
        // Read literal ascii/utf8 string with specified length
        let bytes: Vec<u8> = match self.data.get(self.cur_pos..self.cur_pos + length) {
            Some(bytes) => bytes.to_owned(),
            None => return Err(format!(
                "Trying to read literal string with length {} out of bounds \
                in chunk '{}' at position {}: {} > {}",
                length, self.name, self.cur_pos, self.cur_pos + length, self.data.len(),
            )),
        };
        self.cur_pos += length;

        let string: String = String::from_utf8(bytes).map_err(|e| format!(
            "Could not parse literal string with length {} in chunk '{}' at position {}: {e}",
            length, self.name, self.cur_pos - length,
        ))?;
        Ok(string)
    }

    pub fn read_chunk_name(&mut self) -> Result<String, String> {
        // Read chunk name (4 ascii characters)
        if self.cur_pos + 4 > self.data.len() {
            return Err(format!(
                "Trying to read chunk name out of bounds at position {}: {} > {}",
                self.cur_pos, self.cur_pos + 4, self.data.len(),
            ));
        }

        self.read_literal_string(4)
            .map_err(|e| if self.abs_pos == 0 && self.cur_pos == 4 {
                "Invalid data.win file (because it doesn't start with 'FORM')".to_string()
            } else { 
                format!("Could not parse chunk name at position {}: {e}", self.cur_pos)
            })
    }

    pub fn read_gm_string(&mut self, gm_strings: &GMStrings) -> Result<GMRef<String>, String> {
        let string_abs_pos: usize = self.read_usize_pos()?;
        // if gm_strings.abs_pos_to_reference.get(&string_abs_pos).is_none() {
        //     log::error!("this is only here for easy breakpoints; comment out this if statement otherwise")
        // }
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
        match gm_strings.abs_pos_to_reference.get(&string_abs_pos) {
            None => Ok(None),
            Some(string_ref) => Ok(Some(string_ref.clone())),
        }
    }

    /// read pointer to pointer list (only used in rooms)
    pub fn read_pointer_list(&mut self) -> Result<Vec<usize>, String> {
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
}


