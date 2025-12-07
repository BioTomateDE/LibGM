use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, particle_emitters::GMParticleEmitter},
        reference::GMRef,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
    util::assert::assert_int,
};

#[named_list_chunk("PSYS")]
pub struct GMParticleSystems {
    pub particle_systems: Vec<GMParticleSystem>,
    pub exists: bool,
}

impl GMElement for GMParticleSystems {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        assert_int("PSYS Version", 1, reader.read_u32()?)?;
        let particle_systems: Vec<GMParticleSystem> = reader.read_pointer_list()?;
        Ok(Self { particle_systems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_u32(1); // PSYS Version
        builder.write_pointer_list(&self.particle_systems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleSystem {
    pub name: String,
    pub origin_x: i32,
    pub origin_y: i32,
    pub draw_order: i32,
    pub global_space_particles: Option<bool>,
    pub emitters: Vec<GMRef<GMParticleEmitter>>,
}

impl GMElement for GMParticleSystem {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
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
        builder.write_gm_string(&self.name);
        builder.write_i32(self.origin_x);
        builder.write_i32(self.origin_y);
        builder.write_i32(self.draw_order);
        self.global_space_particles.serialize_if_gm_ver(
            builder,
            "Global Space Particles",
            (2023, 8),
        )?;
        builder.write_simple_list(&self.emitters)?;
        Ok(())
    }
}
