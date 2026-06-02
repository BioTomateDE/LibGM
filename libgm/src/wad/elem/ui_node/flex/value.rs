// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct FlexValue {
    pub value: f32,
    pub unit: Unit,
}

impl GMElement for FlexValue {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let value = reader.read_f32()?;
        let unit: Unit = reader.read_enum()?;
        Ok(Self { value, unit })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.value);
        builder.write_enum(self.unit);
        Ok(())
    }
}

gm_enum!(Unit {
    Undefined = 0,
    Point = 1,
    Percent = 2,
    Auto = 3,
});
