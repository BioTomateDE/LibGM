use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
pub struct GMFeatureFlags {
    pub feature_flags: Vec<GMRef<String>>,
    pub exists: bool,
}

impl Deref for GMFeatureFlags {
    type Target = Vec<GMRef<String>>;
    fn deref(&self) -> &Self::Target {
        &self.feature_flags
    }
}

impl DerefMut for GMFeatureFlags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.feature_flags
    }
}

impl GMChunkElement for GMFeatureFlags {
    const NAME: &'static str = "FEAT";
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
