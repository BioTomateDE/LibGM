// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::room;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectLayer {
    pub enabled: bool,
    pub effect_type: GMRef<String>,
    pub properties: Vec<room::layer::effect::Property>,
}

impl GMElement for EffectLayer {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let enabled = reader.read_bool32()?;
        let effect_type: GMRef<String> = reader.read_gm_string()?;
        let properties: Vec<room::layer::effect::Property> = reader.read_pointer_list()?;
        Ok(Self { enabled, effect_type, properties })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_bool32(self.enabled);
        writer.write_gm_string(self.effect_type)?;
        writer.write_pointer_list(&self.properties)?;
        Ok(())
    }
}
