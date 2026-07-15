// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

/// Audio Groups allow you to manage a set sound entries easier.
/// You can use these for memory management, volume control and more.
/// ___
/// Audio Groups are only available to use in the regular audio system.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudioGroups {
    pub elems: Vec<Option<AudioGroup>>,
    pub exists: bool,
}

gm_named_list_chunk!(AGRP, AudioGroups, AudioGroup, nullable);

impl GMElement for AudioGroups {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<AudioGroup>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioGroup {
    /// The name of the audio group.
    /// This is how the audio group is referenced from code.
    pub name: GMRef<String>,

    /// Relative path (from the main data file) to the audio group file, in
    /// GameMaker 2024.14 and above. ___
    /// Prior to 2024.14, audio groups were all numerically assigned filenames
    /// and all in the root directory.
    pub path: GMRef<String>,
}

impl GMElement for AudioGroup {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let path: GMRef<String> = if reader.version >= GMVersion::GM2024_14 {
            reader.read_gm_string()?
        } else {
            GMRef::none()
        };
        Ok(Self { name, path })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        if builder.version() >= GMVersion::GM2024_14 {
            builder.write_gm_string(self.path)?;
        }
        Ok(())
    }
}
