use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use std::ops::{Deref, DerefMut};

/// Audio Groups allow you to manage a set sound entries easier.
/// You can use these for memory management, volume control and more.
/// ___
/// Audio Groups are only available to use in the regular audio system.
#[derive(Debug, Clone, Default)]
pub struct GMAudioGroups {
    pub audio_groups: Vec<GMAudioGroup>,
    pub exists: bool,
}

impl Deref for GMAudioGroups {
    type Target = Vec<GMAudioGroup>;
    fn deref(&self) -> &Self::Target {
        &self.audio_groups
    }
}

impl DerefMut for GMAudioGroups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.audio_groups
    }
}

impl GMChunkElement for GMAudioGroups {
    const NAME: &'static str = "AGRP";
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
    pub name: String,

    /// Relative path (from the main data file) to the audio group file, in GameMaker 2024.14 and above.
    /// ___
    /// Prior to 2024.14, audio groups were all numerically assigned filenames and all in the root directory.
    pub path: Option<String>,
}

impl GMElement for GMAudioGroup {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let path: Option<String> = if reader.general_info.is_version_at_least((2024, 14)) {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        Ok(Self { name, path })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        if builder.is_gm_version_at_least((2024, 14)) {
            let path = self.path.as_ref().ok_or("Audio Group Path not set for 2024.14+")?;
            builder.write_gm_string(path);
        }
        Ok(())
    }
}
