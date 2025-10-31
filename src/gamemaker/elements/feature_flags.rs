use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct GMFeatureFlags {
    pub feature_flags: Vec<GMRef<String>>,
    pub exists: bool,
}

impl GMChunkElement for GMFeatureFlags {
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMFeatureFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let feature_flags: Vec<GMRef<String>> = reader.read_simple_list_of_strings()?;
        Ok(Self { feature_flags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_simple_list_of_strings(&self.feature_flags)?;
        Ok(())
    }
}
