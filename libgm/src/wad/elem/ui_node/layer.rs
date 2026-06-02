// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_enum(self.draw_space);
        builder.write_bool32(self.visible);
        Ok(())
    }
}

gm_enum!(DrawSpaceKind {
    GUI = 1,
    View = 2,
});
