use crate::deserialize::strings::UTStrings;

#[derive(Clone)]
pub struct UTChunk {
    pub name: String,       // 4 letter name of chunk
    pub abs_pos: usize,     // absolute position/index in data.win file
    pub data: Vec<u8>,      // raw data
    pub data_len: usize,    // length of data for performance
    pub file_index: usize,  // gets incremented by .read_{} methods when parsing chunk
}

impl UTChunk {
    // fn apply_changes(&self, mut changes: Vec<DataChange>) {
    //     changes.sort_by(|a, b| b.index.cmp(&a.index));
    //     for change in changes {
    //         println!(
    //             "[DataChange @ {}] Index: {} | Len: {} | Delete: {}",
    //             self.name,
    //             change.index,
    //             change.content.len(),
    //             change.delete
    //         );
    //         change.apply(self.data.clone());
    //     }
    // }

    // can be refactored (i didn't know about <num>::from_le_bytes)

    pub fn read_u64(&mut self) -> Result<u64, String> {
        // Read unsigned 64-bit integer (little endian)
        if self.file_index + 8 > self.data_len {
            return Err(format!(
                "Trying to read u64 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 8,
                self.data_len
            ));
        }

        let mut number: u64 = 0;
        for i in 0..8 {
            number |= u64::from(self.data[self.file_index]) << (i << 3);
            self.file_index += 1;
        }
        Ok(number)
    }

    pub fn read_i64(&mut self) -> Result<i64, String> {
        // Read signed 64-bit integer (little endian)
        if self.file_index + 8 > self.data_len {
            return Err(format!(
                "Trying to read i64 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 8,
                self.data_len
            ));
        }

        let mut number: i64 = 0;
        for i in 0..8 {
            number |= i64::from(self.data[self.file_index]) << (i << 3);
            self.file_index += 1;
        }
        Ok(number)
    }

    pub fn read_u32(&mut self) -> Result<u32, String> {
        // Read unsigned 32-bit integer (little endian)
        if self.file_index + 4 > self.data_len {
            return Err(format!(
                "Trying to read u32 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 4,
                self.data_len
            ));
        }

        let mut number: u32 = 0;
        for i in 0..4 {
            number |= u32::from(self.data[self.file_index]) << (i << 3);
            self.file_index += 1;
        }
        Ok(number)
    }

    pub fn read_i32(&mut self) -> Result<i32, String> {
        // Read signed 32-bit integer (little endian)
        if self.file_index + 4 > self.data_len {
            return Err(format!(
                "Trying to read i32 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 4,
                self.data_len
            ));
        }

        let mut number: i32 = 0;
        for i in 0..4 {
            number |= i32::from(self.data[self.file_index]) << (i << 3);
            self.file_index += 1;
        }
        Ok(number)
    }
    pub fn read_u16(&mut self) -> Result<u16, String> {
        // Read unsigned 16-bit integer (little endian)
        if self.file_index + 2 > self.data_len {
            return Err(format!(
                "Trying to read u16 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 2,
                self.data_len
            ));
        }

        let mut number: u16 = 0;
        for i in 0..2 {
            number |= u16::from(self.data[self.file_index]) << (i << 3);
            self.file_index += 1;
        }
        Ok(number)
    }
    pub fn read_i16(&mut self) -> Result<i16, String>  {
        // Read signed 16-bit integer (little endian)
        if self.file_index + 2 > self.data_len {
            return Err(format!(
                "Trying to read i16 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 2,
                self.data_len
            ));
        }

        let mut number: i16 = 0;
        for i in 0..2 {
            number |= i16::from(self.data[self.file_index]) << (i << 3);
            self.file_index += 1;
        }
        Ok(number)
    }

    pub fn read_u8(&mut self) -> Result<u8, String> {
        // Read unsigned 8-bit integer (little endian)
        if self.file_index + 1 > self.data_len {
            return Err(format!(
                "Trying to read u8 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 1,
                self.data_len
            ));
        }

        let number: u8 = u8::from(self.data[self.file_index]);
        self.file_index += 1;
        Ok(number)
    }

    pub fn read_i8(&mut self) -> Result<i8, String> {
        // Read signed 8-bit integer (little endian)
        if self.file_index + 1 > self.data_len {
            return Err(format!(
                "Trying to read u8 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 1,
                self.data_len
            ));
        }

        let number: i8 = self.data[self.file_index] as i8;
        self.file_index += 1;
        Ok(number)
    }

    pub fn read_usize(&mut self) -> Result<usize, String> {
        // Read unsigned 32-bit integer and convert to usize (little endian)
        static FAILSAFE_AMOUNT: usize = 100_000_000;
        let number: u32 = self.read_u32()?;
        let number: usize = number as usize;

        if number < FAILSAFE_AMOUNT {
            Ok(number)
        } else {
            Err(format!(
                "Failsafe triggered in chunk '{}' at position {} trying \
                to read usize integer: Number {} is larger than failsafe amount {}",
                self.name,
                self.file_index - 4,
                number,
                FAILSAFE_AMOUNT
            ))
        }
    }

    pub fn read_f32(&mut self) -> Result<f32, String> {
        // Read a single-precision floating point number (little endian)
        if self.file_index + 4 > self.data_len {
            return Err(format!(
                "Trying to read f32 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 4,
                self.data_len
            ));
        }

        let raw: [u8; 4] = self.data[self.file_index .. self.file_index + 4].try_into().unwrap();
        let number: f32 = f32::from_le_bytes(raw);
        self.file_index += 4;
        Ok(number)
    }


    pub fn read_literal_string(&mut self, length: usize) -> Result<String, String> {
        // Read literal ascii/utf8 string with specified length
        if self.file_index + length > self.data_len {
            return Err(format!(
                "Trying to read literal string with length {} out of bounds \
                in chunk '{}' at position {}: {} > {}.",
                length,
                self.name,
                self.file_index,
                self.file_index + length,
                self.data_len
            ));
        }

        let string: Vec<u8> = self.data[self.file_index..self.file_index + length].to_owned();
        self.file_index += length;

        let string = match String::from_utf8(string) {
            Ok(string) => string,
            Err(error) => {
                return Err(format!(
                    "Could not parse literal string with length {} in chunk '{}' at position {}: {}",
                    length,
                    self.name,
                    self.file_index - length,
                    error
                ));
            }
        };
        Ok(string)
    }

    pub fn read_chunk_name(&mut self) -> Result<String, String> {
        // Read chunk name (4 ascii characters)
        if self.file_index + 4 > self.data_len {
            return Err(format!(
                "Trying to read chunk name out of bounds at position {}: {} > {}.",
                self.file_index,
                self.file_index + 4,
                self.data_len
            ));
        }

        match self.read_literal_string(4) {
            Ok(string) => Ok(string),
            Err(error) => Err(format!("Could not parse chunk name at position {}: {}", self.file_index, error))
        }
    }

    pub fn read_ut_string(&mut self, ut_strings: &UTStrings) -> Result<String, String> {
        let string_abs_pos: usize = self.read_usize()?;

        match ut_strings.get_string_by_pos(string_abs_pos) {
            Some(string) => Ok(string.clone()),
            None => Err(format!(
                "Could not read reference string with absolute position {} in chunk '{}' at \
                position {} because it doesn't exist in the string map (length {})",
                string_abs_pos,
                self.name,
                self.file_index - 4,
                ut_strings.len(),
            ))
        }
    }
}
