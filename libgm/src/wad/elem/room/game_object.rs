// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::game_object::GMGameObject;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
    pub x: i32,
    pub y: i32,
    pub object_definition: GMRef<GMGameObject>,
    pub instance_id: u32,
    pub creation_code: GMRef<GMCode>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub image_speed: Option<f32>,
    pub image_index: Option<u32>,
    pub color: u32,
    pub rotation: f32,
    pub pre_create_code: GMRef<GMCode>,
}

impl GMElement for GameObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let object_definition: GMRef<GMGameObject> = reader.read_resource_by_id()?;
        let instance_id = reader.read_u32()?;
        let creation_code: GMRef<GMCode> = reader.read_resource_by_id()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let mut image_speed: Option<f32> = None;
        let mut image_index: Option<u32> = None;
        if reader.general_info.version >= (2, 2, 2, 302) {
            image_speed = Some(reader.read_f32()?);
            image_index = Some(reader.read_u32()?);
        }
        let color = reader.read_u32()?;
        let rotation = reader.read_f32()?; // {~~} FloatAsInt (negative zero handling stuff)

        // [From UndertaleModTool] "is that dependent on WAD or something else?"
        let pre_create_code: GMRef<GMCode> = if reader.general_info.wad_version >= 16 {
            reader.read_resource_by_id()?
        } else {
            GMRef::none()
        };

        Ok(Self {
            x,
            y,
            object_definition,
            instance_id,
            creation_code,
            scale_x,
            scale_y,
            image_speed,
            image_index,
            color,
            rotation,
            pre_create_code,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_resource_id(self.object_definition);
        builder.write_u32(self.instance_id);
        builder.write_resource_id(self.creation_code);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_if_ver(&self.image_speed, "Image Speed", (2, 2, 2, 302))?;
        builder.write_if_ver(&self.image_index, "Image Index", (2, 2, 2, 302))?;
        builder.write_u32(self.color);
        builder.write_f32(self.rotation);
        if builder.wad_version() >= 16 {
            builder.write_resource_id(self.pre_create_code);
        }
        Ok(())
    }
}
