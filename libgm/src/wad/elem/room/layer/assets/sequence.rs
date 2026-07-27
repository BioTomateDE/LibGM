// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::sequence::Sequence;
use crate::wad::elem::sequence::SpeedType;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceInstance {
    pub name: GMRef<String>,
    pub sequence: GMRef<Sequence>,
    pub x: i32,
    pub y: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
    pub animation_speed: f32,
    pub animation_speed_type: SpeedType,
    pub frame_index: f32,
    pub rotation: f32,
}

impl GMElement for SequenceInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let sequence: GMRef<Sequence> = reader.read_resource_by_id()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let color = reader.read_u32()?;
        let animation_speed = reader.read_f32()?;
        let animation_speed_type: SpeedType = reader.read_enum()?;
        let frame_index = reader.read_f32()?;
        let rotation = reader.read_f32()?;
        Ok(Self {
            name,
            sequence,
            x,
            y,
            scale_x,
            scale_y,
            color,
            animation_speed,
            animation_speed_type,
            frame_index,
            rotation,
        })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_resource_id(self.sequence);
        writer.write_i32(self.x);
        writer.write_i32(self.y);
        writer.write_f32(self.scale_x);
        writer.write_f32(self.scale_y);
        writer.write_u32(self.color);
        writer.write_f32(self.animation_speed);
        writer.write_enum(self.animation_speed_type);
        writer.write_f32(self.frame_index);
        writer.write_f32(self.rotation);
        Ok(())
    }
}
