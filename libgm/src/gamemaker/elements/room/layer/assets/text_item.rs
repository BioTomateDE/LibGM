use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, font::GMFont},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq)]
pub struct TextItemInstance {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub font: GMRef<GMFont>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub color: u32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub text: String,
    pub alignment: i32,
    pub character_spacing: f32,
    pub line_spacing: f32,
    pub frame_width: f32,
    pub frame_height: f32,
    pub wrap: bool,
}

impl GMElement for TextItemInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let font: GMRef<GMFont> = reader.read_resource_by_id()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let rotation = reader.read_f32()?;
        let color = reader.read_u32()?;
        let origin_x = reader.read_f32()?;
        let origin_y = reader.read_f32()?;
        let text: String = reader.read_gm_string()?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_resource_id(self.font);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_f32(self.rotation);
        builder.write_u32(self.color);
        builder.write_f32(self.origin_x);
        builder.write_f32(self.origin_y);
        builder.write_gm_string(&self.text);
        builder.write_i32(self.alignment);
        builder.write_f32(self.character_spacing);
        builder.write_f32(self.line_spacing);
        builder.write_f32(self.frame_width);
        builder.write_f32(self.frame_height);
        builder.write_bool32(self.wrap);
        Ok(())
    }
}
