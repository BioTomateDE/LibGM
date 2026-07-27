// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub name: GMRef<String>,
    pub draw_space: DrawSpaceKind,
    pub visible: bool,
}

impl GMElement for Layer {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let draw_space: DrawSpaceKind = reader.read_enum()?;
        let visible = reader.read_bool32()?;
        Ok(Self { name, draw_space, visible })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_enum(self.draw_space);
        writer.write_bool32(self.visible);
        Ok(())
    }
}

gm_enum!(DrawSpaceKind {
    GUI = 1,
    View = 2,
});
