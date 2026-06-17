// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::sequence::SpeedType;
use crate::wad::elem::sprite::Sprite;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, PartialEq)]
pub struct Background {
    pub visible: bool,
    pub foreground: bool,
    pub sprite: GMRef<Sprite>,
    pub tiled_horizontally: bool,
    pub tiled_vertically: bool,
    pub stretch: bool,
    pub color: u32,
    pub first_frame: f32,
    pub animation_speed: f32,
    pub animation_speed_type: SpeedType,
}

impl GMElement for Background {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let visible = reader.read_bool32()?;
        let foreground = reader.read_bool32()?;
        let sprite: GMRef<Sprite> = reader.read_resource_by_id()?;
        let tiled_horizontally = reader.read_bool32()?;
        let tiled_vertically = reader.read_bool32()?;
        let stretch = reader.read_bool32()?;
        let color = reader.read_u32()?;
        let first_frame = reader.read_f32()?;
        let animation_speed = reader.read_f32()?;
        let animation_speed_type: SpeedType = reader.read_enum()?;

        Ok(Self {
            visible,
            foreground,
            sprite,
            tiled_horizontally,
            tiled_vertically,
            stretch,
            color,
            first_frame,
            animation_speed,
            animation_speed_type,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.visible);
        builder.write_bool32(self.foreground);
        builder.write_resource_id(self.sprite);
        builder.write_bool32(self.tiled_horizontally);
        builder.write_bool32(self.tiled_vertically);
        builder.write_bool32(self.stretch);
        builder.write_u32(self.color);
        builder.write_f32(self.first_frame);
        builder.write_f32(self.animation_speed);
        builder.write_enum(self.animation_speed_type);
        Ok(())
    }
}
