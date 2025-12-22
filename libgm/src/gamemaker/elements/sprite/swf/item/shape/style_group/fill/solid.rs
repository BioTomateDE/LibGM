use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let red = reader.read_u8()?;
        let green = reader.read_u8()?;
        let blue = reader.read_u8()?;
        let alpha = reader.read_u8()?;
        Ok(Self { red, green, blue, alpha })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u8(self.red);
        builder.write_u8(self.green);
        builder.write_u8(self.blue);
        builder.write_u8(self.alpha);
        Ok(())
    }
}
