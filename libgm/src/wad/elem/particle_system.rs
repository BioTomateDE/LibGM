// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::particle_emitter::GMParticleEmitter;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMParticleSystems {
    pub particle_systems: Vec<Option<GMParticleSystem>>,
    pub exists: bool,
}

gm_named_list_chunk!(
    PSYS,
    GMParticleSystems,
    GMParticleSystem,
    particle_systems,
    nullable
);

impl GMElement for GMParticleSystems {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("PSYS Version")?;
        let particle_systems: Vec<Option<GMParticleSystem>> = reader.read_pointer_list_opt()?;
        Ok(Self { particle_systems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_u32(1); // PSYS Version
        builder.write_pointer_list_opt(&self.particle_systems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleSystem {
    pub name: GMRef<String>,
    pub origin_x: i32,
    pub origin_y: i32,
    pub draw_order: i32,
    pub global_space_particles: Option<bool>,
    pub emitters: Vec<GMRef<GMParticleEmitter>>,
}

impl GMElement for GMParticleSystem {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.chunk.length() > 4 {
            log::warn!("Particle systems are not tested");
        }
        let name: GMRef<String> = reader.read_gm_string()?;
        let origin_x = reader.read_i32()?;
        let origin_y = reader.read_i32()?;
        let draw_order = reader.read_i32()?;
        let global_space_particles: Option<bool> = reader.deserialize_if_gm_version((2023, 8))?;
        let emitters: Vec<GMRef<GMParticleEmitter>> = reader.read_simple_list()?;
        Ok(Self {
            name,
            origin_x,
            origin_y,
            draw_order,
            global_space_particles,
            emitters,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_i32(self.origin_x);
        builder.write_i32(self.origin_y);
        builder.write_i32(self.draw_order);
        builder.write_if_ver(
            &self.global_space_particles,
            "Global Space Particles",
            (2023, 8),
        )?;
        builder.write_simple_list(&self.emitters)?;
        Ok(())
    }
}
