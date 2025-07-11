use crate::gamemaker::deserialize::reader::DataReader;

impl DataReader<'_> {
    pub fn read_u64(&mut self) -> Result<u64, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read u64 {e}"))?;
        Ok(if is_be {
            u64::from_be_bytes(*bytes)
        } else {
            u64::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_i64(&mut self) -> Result<i64, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read i64 {e}"))?;
        Ok(if is_be {
            i64::from_be_bytes(*bytes)
        } else {
            i64::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_u32(&mut self) -> Result<u32, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read u32 {e}"))?;
        Ok(if is_be {
            u32::from_be_bytes(*bytes)
        } else {
            u32::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_i32(&mut self) -> Result<i32, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read i32 {e}"))?;
        Ok(if is_be {
            i32::from_be_bytes(*bytes)
        } else {
            i32::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_u16(&mut self) -> Result<u16, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read u16 {e}"))?;
        Ok(if is_be {
            u16::from_be_bytes(*bytes)
        } else {
            u16::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_i16(&mut self) -> Result<i16, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read i16 {e}"))?;
        Ok(if is_be {
            i16::from_be_bytes(*bytes)
        } else {
            i16::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_u8(&mut self) -> Result<u8, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read u8 {e}"))?;
        Ok(if is_be {
            u8::from_be_bytes(*bytes)
        } else {
            u8::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_i8(&mut self) -> Result<i8, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read i8 {e}"))?;
        Ok(if is_be {
            i8::from_be_bytes(*bytes)
        } else {
            i8::from_le_bytes(*bytes)
        })
    }

    pub fn read_f64(&mut self) -> Result<f64, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read f64 {e}"))?;
        Ok(if is_be {
            f64::from_be_bytes(*bytes)
        } else {
            f64::from_le_bytes(*bytes)
        })
    }
    
    pub fn read_f32(&mut self) -> Result<f32, String> {
        let is_be: bool = self.is_big_endian;
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read f32 {e}"))?;
        Ok(if is_be {
            f32::from_be_bytes(*bytes)
        } else {
            f32::from_le_bytes(*bytes)
        })
    }

    pub fn read_usize(&mut self) -> Result<usize, String> {
        let number: u32 = self.read_u32()?;
        Ok(number as usize)
    }
}

