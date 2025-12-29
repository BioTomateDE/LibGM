use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

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
            *tile_mode = num_enum_from(reader.read_i32()?)?;
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
            builder.write_i32((*tile_mode).into());
        }
        Ok(())
    }
}

#[num_enum(i32)]
pub enum TileMode {
    Stretch = 0,
    Repeat = 1,
    Mirror = 2,
    BlankRepeat = 3,
    Hide = 4,
}
