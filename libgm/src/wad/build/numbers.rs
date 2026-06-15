use crate::gm_enum::GMEnum;
// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::data::Endianness;

macro_rules! write_int_fn {
    ($method_name:ident, $int_type:ty) => {
        pub fn $method_name(&mut self, number: $int_type) {
            let bytes = match self.gm_data.meta.endianness {
                Endianness::Little => number.to_le_bytes(),
                Endianness::Big => number.to_be_bytes(),
            };
            self.write_bytes(&bytes);
        }
    };
}

impl DataBuilder<'_> {
    write_int_fn!(write_u64, u64);

    write_int_fn!(write_u32, u32);

    write_int_fn!(write_u16, u16);

    write_int_fn!(write_u8, u8);

    write_int_fn!(write_i64, i64);

    write_int_fn!(write_i32, i32);

    write_int_fn!(write_i16, i16);

    write_int_fn!(write_i8, i8);

    write_int_fn!(write_f64, f64);

    write_int_fn!(write_f32, f32);

    pub fn write_enum<T: GMEnum>(&mut self, gm_enum: T) {
        self.write_i32(gm_enum.as_i32());
    }

    pub fn write_usize(&mut self, number: usize) -> Result<()> {
        let number: u32 = number.try_into().ctx_any(|| {
            format!(
                "Number {number} (0x{number:016X}) does not fit into 32 bits while writing usize \
                 integer"
            )
        })?;
        self.write_u32(number);
        Ok(())
    }
}
