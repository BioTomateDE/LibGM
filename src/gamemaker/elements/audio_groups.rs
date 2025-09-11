use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMAudioGroups {
    pub audio_groups: Vec<GMAudioGroup>,
    pub exists: bool,
}
impl GMChunkElement for GMAudioGroups {
    fn stub() -> Self {
        Self { audio_groups: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMAudioGroups {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let audio_groups: Vec<GMAudioGroup> = reader.read_pointer_list()?;
        Ok(Self { audio_groups, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.audio_groups)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAudioGroup {
    pub name: GMRef<String>,
    pub path: Option<GMRef<String>>,
}
impl GMElement for GMAudioGroup {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let path: Option<GMRef<String>> = if reader.general_info.is_version_at_least((2024, 14)) {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        Ok(Self { name, path })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        if builder.is_gm_version_at_least((2024, 14)) {
            builder.write_gm_string(&self.path.ok_or("Audio Group Path not set for 2024.14+")?)?;
        }
        Ok(())
    }
}

