// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::font::Font;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;
#[derive(Debug, Clone, PartialEq)]
pub struct TextItemInstance {
    pub name: GMRef<String>,
    pub x: i32,
    pub y: i32,
    pub font: GMRef<Font>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub color: u32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub text: GMRef<String>,
    pub alignment: i32,
    pub character_spacing: f32,
    pub line_spacing: f32,
    pub frame_width: f32,
    pub frame_height: f32,
    pub wrap: bool,
}

impl GMElement for TextItemInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let font: GMRef<Font> = reader.read_resource_by_id()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let rotation = reader.read_f32()?;
        let color = reader.read_u32()?;
        let origin_x = reader.read_f32()?;
        let origin_y = reader.read_f32()?;
        let text: GMRef<String> = reader.read_gm_string()?;
        let alignment = reader.read_i32()?;
        let character_spacing = reader.read_f32()?;
        let line_spacing = reader.read_f32()?;
        let frame_width = reader.read_f32()?;
        let frame_height = reader.read_f32()?;
        let wrap = reader.read_bool32()?;

        Ok(Self {
            name,
            x,
            y,
            font,
            scale_x,
            scale_y,
            rotation,
            color,
            origin_x,
            origin_y,
            text,
            alignment,
            character_spacing,
            line_spacing,
            frame_width,
            frame_height,
            wrap,
        })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_i32(self.x);
        writer.write_i32(self.y);
        writer.write_resource_id(self.font);
        writer.write_f32(self.scale_x);
        writer.write_f32(self.scale_y);
        writer.write_f32(self.rotation);
        writer.write_u32(self.color);
        writer.write_f32(self.origin_x);
        writer.write_f32(self.origin_y);
        writer.write_gm_string(self.text)?;
        writer.write_i32(self.alignment);
        writer.write_f32(self.character_spacing);
        writer.write_f32(self.line_spacing);
        writer.write_f32(self.frame_width);
        writer.write_f32(self.frame_height);
        writer.write_bool32(self.wrap);
        Ok(())
    }
}
