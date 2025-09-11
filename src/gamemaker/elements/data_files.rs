use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;


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
    fn deserialize(_reader: &mut DataReader) -> Result<Self, String> {
        Ok(Self { exists: true })
    }

    fn serialize(&self, _builder: &mut DataBuilder) -> Result<(), String> {
        Ok(())
    }
}

