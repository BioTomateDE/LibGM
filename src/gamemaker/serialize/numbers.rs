use crate::gamemaker::serialize::DataBuilder;

impl DataBuilder<'_> {
    pub fn write_u64(&mut self, number: u64) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_i64(&mut self, number: i64) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_u32(&mut self, number: u32) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_i32(&mut self, number: i32) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_u16(&mut self, number: u16) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_i16(&mut self, number: i16) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_u8(&mut self, number: u8) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_i8(&mut self, number: i8) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_f64(&mut self, number: f64) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_f32(&mut self, number: f32) {
        self.write_bytes(&if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        })
    }
    
    pub fn write_usize(&mut self, number: usize) -> Result<(), String> {
        let number: u32 = number.try_into().map_err(|_| format!(
            "Number {number} (0x{number:016X}) does not fit into 32 bits while writing usize integer",
        ))?;
        self.write_u32(number);
        Ok(())
    }

    pub fn write_i24(&mut self, number: i32) {
        let masked: u32 = (number as u32) & 0x00FF_FFFF;
        let bytes: [u8; 4] = if self.gm_data.is_big_endian {
            masked.to_be_bytes()
        } else {
            masked.to_le_bytes()
        };
        self.raw_data.extend_from_slice(&bytes[..3]);
    }
}

