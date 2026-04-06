use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bool {
    pub boolean: bool,
}

impl GMElement for Bool {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let boolean = reader.read_bool32()?;
        Ok(Self { boolean })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.boolean);
        Ok(())
    }
}
