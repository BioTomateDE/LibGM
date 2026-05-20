// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::elem::GMElement;
use crate::wad::elem::sound::GMSound;
use crate::wad::reference::GMRef;
use crate::wad::build::builder::DataBuilder;

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sound: GMRef<GMSound>,
    pub mode: i32,
}

impl GMElement for Audio {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sound: GMRef<GMSound> = reader.read_resource_by_id()?;
        let mode = reader.read_i32()?;
        Ok(Self { sound, mode })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.sound);
        builder.write_i32(self.mode);
        Ok(())
    }
}
