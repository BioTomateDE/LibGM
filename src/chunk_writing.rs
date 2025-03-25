

#[derive(Debug, Clone)]
pub struct DataBuilder {
    pub raw_data: Vec<u8>,
}


impl DataBuilder {
    fn write_number<T: num_traits::ops::bytes::ToBytes>(&mut self, number: T) -> Result<(), String> {
        let bytes = number.to_le_bytes();
        for byte in bytes {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_u64(&mut self, number: u64) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_i64(&mut self, number: i64) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_u32(&mut self, number: u32) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_i32(&mut self, number: i32) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_u16(&mut self, number: u16) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_i16(&mut self, number: i16) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_u8(&mut self, number: u8) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_i8(&mut self, number: i8) -> Result<(), String> {
        self.write_number(number)
    }
    pub fn write_usize(&mut self, number: usize) -> Result<(), String> {
        self.write_number(number as u32)
    }
    pub fn write_bool(&mut self, boolean: bool) -> Result<(), String> {
        let number: u8 = if boolean {1} else {0};
        self.write_number(number)
    }
    pub fn write_string(&mut self, string: &str) -> Result<(), String> {
        // write an ascii string to the data
        for (i, char) in string.chars().enumerate() {
            let byte: u8 = match char.try_into() {
                Ok(byte) => byte,
                Err(_) => return Err(format!("Char Typecasting error while writing string \"{string}\" (i: {i}) to data (len: {})", self.raw_data.len())),
                // Err(_) => return Err(format!("Non-Ascii character at position {i} in string \"{string}\" (data len: {}).", self.raw_data.len())),
            };
            self.raw_data.push(byte);
        }
        Ok(())
    }
}

