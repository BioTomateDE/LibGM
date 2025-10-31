use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::prelude::*;

/// Audio Groups allow you to manage a set sound entries easier.
/// You can use these for memory management, volume control and more.
/// ___
/// Audio Groups are only available to use in the regular audio system.
#[derive(Debug, Clone, Default)]
pub struct GMAudioGroups {
    pub audio_groups: Vec<GMAudioGroup>,
    pub exists: bool,
}

impl GMChunkElement for GMAudioGroups {
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMAudioGroups {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let audio_groups: Vec<GMAudioGroup> = reader.read_pointer_list()?;
        Ok(Self { audio_groups, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.audio_groups)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAudioGroup {
    /// The name of the audio group.
    /// This is how the audio group is referenced from code.
    pub name: GMRef<String>,

    /// Relative path (from the main data file) to the audio group file, in GameMaker 2024.14 and above.
    /// ___
    /// Prior to 2024.14, audio groups were all numerically assigned filenames and all in the root directory.
    pub path: Option<GMRef<String>>,
}
impl GMElement for GMAudioGroup {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let path: Option<GMRef<String>> = if reader.general_info.is_version_at_least((2024, 14)) {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        Ok(Self { name, path })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        if builder.is_gm_version_at_least((2024, 14)) {
            builder.write_gm_string(&self.path.context("Audio Group Path not set for 2024.14+")?)?;
        }
        Ok(())
    }
}
