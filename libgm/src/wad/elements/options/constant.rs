use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    pub name: String,
    pub value: String,
}

impl GMElement for Constant {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let value: String = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_gm_string(&self.value);
        Ok(())
    }
}
