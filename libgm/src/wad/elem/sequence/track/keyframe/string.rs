use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::elem::GMElement;
use crate::wad::build::builder::DataBuilder;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct String {
    pub string: std::string::String,
}

impl GMElement for String {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let string: std::string::String = reader.read_gm_string()?;
        Ok(Self { string })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.string);
        Ok(())
    }
}
