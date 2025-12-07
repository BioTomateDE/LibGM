use macros::list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[list_chunk("FEAT")]
pub struct GMFeatureFlags {
    pub feature_flags: Vec<String>,
    pub exists: bool,
}

impl GMElement for GMFeatureFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let feature_flags: Vec<String> = reader.read_simple_list_of_strings()?;
        Ok(Self { feature_flags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_simple_list_of_strings(&self.feature_flags)?;
        Ok(())
    }
}
