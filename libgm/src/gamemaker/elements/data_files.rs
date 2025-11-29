use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, Default)]
pub struct GMDataFiles;

impl GMChunkElement for GMDataFiles {
    const NAME: &'static str = "DAFL";
    /// This chunk is completely useless and should never be serialized.
    fn exists(&self) -> bool {
        false
    }
}

impl GMElement for GMDataFiles {
    fn deserialize(_: &mut DataReader) -> Result<Self> {
        Ok(Self)
    }

    fn serialize(&self, _: &mut DataBuilder) -> Result<()> {
        Ok(())
    }
}
