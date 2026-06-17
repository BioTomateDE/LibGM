// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::particle_emitter::ParticleEmitter;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParticleSystems {
    pub elems: Vec<Option<ParticleSystem>>,
    pub exists: bool,
}

gm_named_list_chunk!(PSYS, ParticleSystems, ParticleSystem, nullable);

impl GMElement for ParticleSystems {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("PSYS Version")?;
        let elems: Vec<Option<ParticleSystem>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_u32(1); // PSYS Version
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSystem {
    pub name: GMRef<String>,
    pub origin_x: i32,
    pub origin_y: i32,
    pub draw_order: i32,
    pub global_space_particles: Option<bool>,
    pub emitters: Vec<GMRef<ParticleEmitter>>,
}

impl GMElement for ParticleSystem {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.chunk.length() > 4 {
            log::warn!("Particle systems are not tested");
        }
        let name: GMRef<String> = reader.read_gm_string()?;
        let origin_x = reader.read_i32()?;
        let origin_y = reader.read_i32()?;
        let draw_order = reader.read_i32()?;
        let global_space_particles: Option<bool> = reader.deserialize_if_gm_version((2023, 8))?;
        let emitters: Vec<GMRef<ParticleEmitter>> = reader.read_simple_list()?;
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
