// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::particle_system::GMParticleSystem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

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
