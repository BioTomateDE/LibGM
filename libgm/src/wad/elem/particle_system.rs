// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::chunk::gm_asset_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::particle_emitter::ParticleEmitter;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParticleSystems {
    pub elems: Vec<Option<ParticleSystem>>,
}

gm_asset_list_chunk!(PSYS, ParticleSystems, ParticleSystem);

impl GMElement for ParticleSystems {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("PSYS Version")?;
        let elems: Vec<Option<ParticleSystem>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.align(4);
        writer.write_u32(1); // PSYS Version
        writer.write_pointer_list_opt(&self.elems)?;
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
        let global_space_particles: Option<bool> =
            reader.deserialize_if_version(GMVersion::GM2023_8)?;
        let emitters: Vec<GMRef<ParticleEmitter>> = reader.read_simple_list()?;
        Ok(Self { name, origin_x, origin_y, draw_order, global_space_particles, emitters })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_i32(self.origin_x);
        writer.write_i32(self.origin_y);
        writer.write_i32(self.draw_order);
        writer.write_if_ver(
            &self.global_space_particles,
            "Global Space Particles",
            GMVersion::GM2023_8,
        )?;
        writer.write_simple_list(&self.emitters)?;
        Ok(())
    }
}
