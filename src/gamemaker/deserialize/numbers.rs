use crate::prelude::*;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::data::Endianness;


macro_rules! read_int_fn {
    ($method:ident, $dtype:ty) => {
        /// Read an integer from the data file while advancing the data position.
        /// Respects the endianness setting.
        pub fn $method(&mut self) -> Result<$dtype> {
            let bytes = *self.read_bytes_const()
                .map_err(|e| format!("Trying to read {} {e}", stringify!($dtype)))?;
            Ok(match self.endianness {
                Endianness::Little => <$dtype>::from_le_bytes(bytes),
                Endianness::Big => <$dtype>::from_be_bytes(bytes),
            })
        }
    }
}

impl DataReader<'_> {
    read_int_fn!(read_u64, u64);
    read_int_fn!(read_u32, u32);
    read_int_fn!(read_u16, u16);
    read_int_fn!(read_u8, u8);

    read_int_fn!(read_i64, i64);
    read_int_fn!(read_i32, i32);
    read_int_fn!(read_i16, i16);
    read_int_fn!(read_i8, i8);

    read_int_fn!(read_f64, f64);
    read_int_fn!(read_f32, f32);

    /// Read an unsigned 32-bit integer from the data file while advancing the data position.
    /// The integer will be converted to a `usize` for easy use in indices.
    pub fn read_usize(&mut self) -> Result<usize> {
        let number = self.read_u32()?;
        Ok(number as usize)
    }

    /// Read an unsigned 32-bit integer from the data file while advancing the data position.
    /// Returns zero if the read number is -1 or 0.
    pub fn read_count(&mut self, purpose: &'static str) -> Result<u32> {
        let number = self.read_i32()?;
        match number {
            -1 => Ok(0),
            n if n >= 0 => Ok(n as u32),
            _ => bail!("Negative {purpose} count {number} (0x{number:08X})"),
        }
    }
}

