use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, background::GMBackground, sprite::GMSprite},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub texture: Option<Texture>,
    pub source_x: u32,
    pub source_y: u32,
    pub width: u32,
    pub height: u32,
    pub tile_depth: i32,
    pub instance_id: u32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
}

impl GMElement for Tile {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let texture: Option<Texture> = if reader.general_info.is_version_at_least((2, 0)) {
            reader.read_resource_by_id_opt()?.map(Texture::Sprite)
        } else {
            reader.read_resource_by_id_opt()?.map(Texture::Background)
        };
        let source_x = reader.read_u32()?;
        let source_y = reader.read_u32()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let tile_depth = reader.read_i32()?;
        let instance_id = reader.read_u32()?;
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
            Some(Texture::Sprite(sprite_ref)) => {
                if builder.is_gm_version_at_least((2, 0)) {
                    builder.write_resource_id(sprite_ref);
                } else {
                    bail!(
                        "Room tile texture should be a Background reference before GMS2; not a Sprite reference"
                    );
                }
            },
            Some(Texture::Background(background_ref)) => {
                if builder.is_gm_version_at_least((2, 0)) {
                    bail!(
                        "Room tile texture should be a Sprite reference in GMS2+; not a Background reference"
                    );
                }
                builder.write_resource_id(background_ref);
            },
            None => builder.write_u32(0),
        }
        builder.write_u32(self.source_x);
        builder.write_u32(self.source_y);
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        builder.write_i32(self.tile_depth);
        builder.write_u32(self.instance_id);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_u32(self.color);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Texture {
    Sprite(GMRef<GMSprite>),
    Background(GMRef<GMBackground>),
}
