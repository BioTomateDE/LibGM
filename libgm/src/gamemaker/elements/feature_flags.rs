use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        chunk::ChunkName,
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMFeatureFlags {
    pub feature_flags: Vec<String>,
    pub exists: bool,
}

impl Deref for GMFeatureFlags {
    type Target = Vec<String>;
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
    const NAME: ChunkName = ChunkName::new("FEAT");
    fn exists(&self) -> bool {
        self.exists
    }
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
