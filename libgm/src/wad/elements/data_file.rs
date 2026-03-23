use crate::{
    prelude::*,
    wad::{
        chunk::ChunkName,
        deserialize::reader::DataReader,
        elements::{GMChunk, GMElement},
        serialize::builder::DataBuilder,
    },
};

/// This is the always-empty unused chunk `DAFL`.
///
/// It is unrelated to the `GMData` struct.
#[derive(Debug, Clone, Default)]
pub struct GMDataFiles;

impl GMChunk for GMDataFiles {
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
