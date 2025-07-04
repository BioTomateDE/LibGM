use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement};
use crate::gm_serialize::DataBuilder;


/// most useful gamemaker chunk:
#[derive(Debug, Clone)]
pub struct GMDataFiles {
    pub exists: bool,
}

impl GMChunkElement for GMDataFiles {
    fn empty() -> Self {
        Self { exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMDataFiles {
    fn deserialize(_reader: &mut DataReader) -> Result<Self, String> {
        Ok(Self { exists: true })
    }

    fn serialize(&self, _builder: &mut DataBuilder) -> Result<(), String> {
        Ok(())
    }
}

