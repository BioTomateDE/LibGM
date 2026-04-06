use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::elements::particle_system::GMParticleSystem;
use crate::wad::reference::GMRef;
use crate::wad::serialize::builder::DataBuilder;

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
