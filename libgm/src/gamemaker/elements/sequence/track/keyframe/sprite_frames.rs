use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteFrames {
    pub value: i32,
}

impl GMElement for SpriteFrames {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let value = reader.read_i32()?;
        Ok(Self { value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.value);
        Ok(())
    }
}
