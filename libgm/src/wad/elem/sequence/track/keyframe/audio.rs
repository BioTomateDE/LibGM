// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::sound::Sound;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sound: GMRef<Sound>,
    pub mode: i32,
}

impl GMElement for Audio {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sound: GMRef<Sound> = reader.read_resource_by_id()?;
        let mode = reader.read_i32()?;
        Ok(Self { sound, mode })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.sound);
        builder.write_i32(self.mode);
        Ok(())
    }
}
