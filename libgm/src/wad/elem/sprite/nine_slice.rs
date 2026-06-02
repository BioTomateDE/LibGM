// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::GMEnum;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NineSlice {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub enabled: bool,
    pub tile_modes: [TileMode; 5],
}

impl GMElement for NineSlice {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let left = reader.read_i32()?;
        let top = reader.read_i32()?;
        let right = reader.read_i32()?;
        let bottom = reader.read_i32()?;
        let enabled = reader.read_bool32()?;

        let mut tile_modes: [TileMode; 5] = [TileMode::Stretch; 5]; // Ignore default value
        for tile_mode in &mut tile_modes {
            *tile_mode = reader.read_enum()?;
        }

        Ok(Self {
            left,
            top,
            right,
            bottom,
            enabled,
            tile_modes,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.left);
        builder.write_i32(self.top);
        builder.write_i32(self.right);
        builder.write_i32(self.bottom);
        builder.write_bool32(self.enabled);
        for tile_mode in &self.tile_modes {
            builder.write_i32(tile_mode.as_i32());
        }
        Ok(())
    }
}

gm_enum!(TileMode {
    Stretch = 0,
    Repeat = 1,
    Mirror = 2,
    BlankRepeat = 3,
    Hide = 4,
});
