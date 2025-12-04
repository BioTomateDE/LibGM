use crate::{
    gamemaker::{
        chunk::ChunkName,
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, Default)]
pub struct GMDataFiles;

impl GMChunkElement for GMDataFiles {
    const NAME: ChunkName = ChunkName::new("DAFL");
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
