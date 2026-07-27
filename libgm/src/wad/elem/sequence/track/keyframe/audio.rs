// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::sound::Sound;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_resource_id(self.sound);
        writer.write_i32(self.mode);
        Ok(())
    }
}
