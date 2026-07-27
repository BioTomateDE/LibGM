// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::elem::GMElement;
use crate::wad::elem::room::InstanceID;
use crate::wad::elem::sprite::Sprite;
use crate::wad::elem::tileset::Tileset;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

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
        let texture: Texture = if reader.version >= GMVersion::GMS2 {
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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i32(self.x);
        writer.write_i32(self.y);
        match self.texture {
            Texture::Sprite(sprite_ref) => {
                if writer.version() >= GMVersion::GMS2 {
                    writer.write_resource_id(sprite_ref);
                } else {
                    bail!(
                        "Room tile texture should be a Background reference before GMS2; not a \
                         Sprite reference"
                    );
                }
            }
            Texture::Background(background_ref) => {
                if writer.version() >= GMVersion::GMS2 {
                    bail!(
                        "Room tile texture should be a Sprite reference in GMS2+; not a \
                         Background reference"
                    );
                }
                writer.write_resource_id(background_ref);
            }
        }
        writer.write_u32(self.source_x);
        writer.write_u32(self.source_y);
        writer.write_u32(self.width);
        writer.write_u32(self.height);
        writer.write_i32(self.tile_depth);
        writer.write_i32(self.instance_id.0);
        writer.write_f32(self.scale_x);
        writer.write_f32(self.scale_y);
        writer.write_u32(self.color);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Texture {
    Sprite(GMRef<Sprite>),
    Background(GMRef<Tileset>),
}
