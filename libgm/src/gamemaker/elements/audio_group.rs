use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::GMElement,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
};

/// Audio Groups allow you to manage a set sound entries easier.
/// You can use these for memory management, volume control and more.
/// ___
/// Audio Groups are only available to use in the regular audio system.
#[named_list_chunk("AGRP")]
pub struct GMAudioGroups {
    pub audio_groups: Vec<GMAudioGroup>,
    pub exists: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let path: Option<String> = reader.deserialize_if_gm_version((2024, 14))?;
        Ok(Self { name, path })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        self.path.serialize_if_gm_ver(builder, "Path", (2024, 14))?;
        Ok(())
    }
}
