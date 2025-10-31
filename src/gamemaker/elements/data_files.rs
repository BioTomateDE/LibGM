use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct GMDataFiles;

impl GMChunkElement for GMDataFiles {
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
