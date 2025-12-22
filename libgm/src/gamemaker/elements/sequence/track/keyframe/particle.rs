use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, particle_system::GMParticleSystem},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    pub particle: GMRef<GMParticleSystem>,
}

impl GMElement for Particle {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let particle: GMRef<GMParticleSystem> = reader.read_resource_by_id()?;
        Ok(Self { particle })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.particle);
        Ok(())
    }
}
