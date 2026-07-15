// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::room::InstanceID;
use crate::wad::elem::sprite::Sprite;
use crate::wad::elem::tileset::Tileset;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, PartialEq)]
pub struct RoomTile {
    pub x: i32,
    pub y: i32,
    pub texture: Texture,
    pub source_x: u32,
    pub source_y: u32,
    pub width: u32,
    pub height: u32,
    pub tile_depth: i32,
    pub instance_id: InstanceID,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
}

impl GMElement for RoomTile {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let texture: Texture = if reader.version >= GMVersion::Studio2 {
            Texture::Sprite(reader.read_resource_by_id()?)
        } else {
            Texture::Background(reader.read_resource_by_id()?)
        };
        let source_x = reader.read_u32()?;
        let source_y = reader.read_u32()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let tile_depth = reader.read_i32()?;
        let instance_id = InstanceID(reader.read_i32()?);
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let color = reader.read_u32()?;
        Ok(Self {
            x,
            y,
            texture,
            source_x,
            source_y,
            width,
            height,
            tile_depth,
            instance_id,
            scale_x,
            scale_y,
            color,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        match self.texture {
            Texture::Sprite(sprite_ref) => {
                if builder.version() >= GMVersion::Studio2 {
                    builder.write_resource_id(sprite_ref);
                } else {
                    bail!(
                        "Room tile texture should be a Background reference before GMS2; not a \
                         Sprite reference"
                    );
                }
            }
            Texture::Background(background_ref) => {
                if builder.version() >= GMVersion::Studio2 {
                    bail!(
                        "Room tile texture should be a Sprite reference in GMS2+; not a \
                         Background reference"
                    );
                }
                builder.write_resource_id(background_ref);
            }
        }
        builder.write_u32(self.source_x);
        builder.write_u32(self.source_y);
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        builder.write_i32(self.tile_depth);
        builder.write_i32(self.instance_id.0);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_u32(self.color);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Texture {
    Sprite(GMRef<Sprite>),
    Background(GMRef<Tileset>),
}
