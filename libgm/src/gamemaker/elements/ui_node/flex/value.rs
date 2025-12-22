use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FlexValue {
    pub value: f32,
    pub unit: Unit,
}

impl GMElement for FlexValue {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let value = reader.read_f32()?;
        let unit: Unit = num_enum_from(reader.read_i32()?)?;
        Ok(Self { value, unit })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.value);
        builder.write_i32(self.unit.into());
        Ok(())
    }
}

#[num_enum(i32)]
pub enum Unit {
    Undefined = 0,
    Point = 1,
    Percent = 2,
    Auto = 3,
}
