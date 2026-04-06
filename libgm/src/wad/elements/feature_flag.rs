use macros::list_chunk;

use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;

#[list_chunk("FEAT")]
pub struct GMFeatureFlags {
    pub feature_flags: Vec<String>,
    pub exists: bool,
}

impl GMElement for GMFeatureFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let feature_flags: Vec<String> = reader.read_simple_list()?;
        Ok(Self { feature_flags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_simple_list(&self.feature_flags)?;
        Ok(())
    }
}
