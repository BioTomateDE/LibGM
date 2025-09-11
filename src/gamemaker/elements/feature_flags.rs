use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMFeatureFlags {
    pub feature_flags: Vec<GMRef<String>>,
    pub exists: bool,
}

impl GMChunkElement for GMFeatureFlags {
    fn stub() -> Self {
        Self { feature_flags: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMFeatureFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.align(4)?;
        let feature_flags: Vec<GMRef<String>> = reader.read_simple_list_of_strings()?;
        Ok(Self { feature_flags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.align(4);
        builder.write_simple_list_of_strings(&self.feature_flags)?;
        Ok(())
    }
}

