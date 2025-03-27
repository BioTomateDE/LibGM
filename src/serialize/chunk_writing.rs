

#[derive(Debug, Clone)]
pub struct ChunkBuilder {
    pub raw_data: Vec<u8>,
    pub chunk_name: &'static str,
}


impl ChunkBuilder {
    pub fn write_u64(&mut self, number: u64) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_i64(&mut self, number: i64) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_u32(&mut self, number: u32) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_i32(&mut self, number: i32) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_u16(&mut self, number: u16) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_i16(&mut self, number: i16) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_u8(&mut self, number: u8) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_i8(&mut self, number: i8) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_usize(&mut self, number: usize) -> Result<(), String> {
        for byte in (number as u32).to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_f32(&mut self, number: f32) -> Result<(), String> {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_bool(&mut self, boolean: bool) -> Result<(), String> {
        let number: u8 = if boolean {1} else {0};
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_string(&mut self, string: &str) -> Result<(), String> {
        // write an ascii string to the data
        for (i, char) in string.chars().enumerate() {
            let byte: u8 = match char.try_into() {
                Ok(byte) => byte,
                Err(_) => return Err(format!("Char Typecasting error while writing string \"{string}\" (i: {i}) to chunk (len: {})", self.len())),
            };
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn overwrite_data(&mut self, data: &[u8], position: usize) -> Result<(), String> {
        if position + data.len() >= self.len() {
            return Err(format!(
                "Could not overwrite {} bytes at position {} in data with length {} while building chunk.",
                data.len(),
                position,
                self.len()
            ))
        };

        for (i, byte) in data.iter().enumerate() {
            self.raw_data[position + i] = *byte;
        }

        Ok(())
    }
    pub fn len(&self) -> usize {
        self.raw_data.len()
    }
}

