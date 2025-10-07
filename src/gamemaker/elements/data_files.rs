use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::prelude::*;

/// most useful gamemaker chunk:
#[derive(Debug, Clone)]
pub struct GMDataFiles {
    pub exists: bool,
}

impl GMChunkElement for GMDataFiles {
    fn stub() -> Self {
        Self { exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMDataFiles {
    fn deserialize(_reader: &mut DataReader) -> Result<Self> {
        Ok(Self { exists: true })
    }

    fn serialize(&self, _builder: &mut DataBuilder) -> Result<()> {
        Ok(())
    }
}
