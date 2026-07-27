// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::particle_system::ParticleSystem;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    pub particle: GMRef<ParticleSystem>,
}

impl GMElement for Particle {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let particle: GMRef<ParticleSystem> = reader.read_resource_by_id()?;
        Ok(Self { particle })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_resource_id(self.particle);
        Ok(())
    }
}
