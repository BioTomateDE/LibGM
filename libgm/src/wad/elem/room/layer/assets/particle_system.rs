// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::particle_system::ParticleSystem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSystemInstance {
    pub name: GMRef<String>,
    pub particle_system: GMRef<ParticleSystem>,
    pub x: i32,
    pub y: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
    pub rotation: f32,
}

impl GMElement for ParticleSystemInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let particle_system: GMRef<ParticleSystem> = reader.read_resource_by_id()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let color = reader.read_u32()?;
        let rotation = reader.read_f32()?;
        Ok(Self {
            name,
            particle_system,
            x,
            y,
            scale_x,
            scale_y,
            color,
            rotation,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_resource_id(self.particle_system);
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_u32(self.color);
        builder.write_f32(self.rotation);
        Ok(())
    }
}
