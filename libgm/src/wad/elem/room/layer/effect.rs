// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    pub effect_type: GMRef<String>,
    pub properties: Vec<Property>,
}

impl GMElement for Effect {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        // {~~} dont serialize_old if >= 2022.1??
        let effect_type: GMRef<String> = reader.read_gm_string()?;
        let properties: Vec<Property> = reader.read_simple_list()?;
        Ok(Self { effect_type, properties })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.effect_type)?;
        writer.write_simple_list(&self.properties)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub kind: PropertyKind,
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}

impl GMElement for Property {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let kind: PropertyKind = reader.read_enum()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { kind, name, value })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_enum(self.kind);
        writer.write_gm_string(self.name)?;
        writer.write_gm_string(self.value)?;
        Ok(())
    }
}

gm_enum!(PropertyKind {
    Real = 0,
    Color = 1,
    Sampler = 2,
});
