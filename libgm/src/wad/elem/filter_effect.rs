// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilterEffects {
    pub elems: Vec<FilterEffect>,
}

gm_list_chunk!(FEDS, FilterEffects, FilterEffect);

impl GMElement for FilterEffects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("FEDS Version")?;
        let elems: Vec<FilterEffect> = reader.read_pointer_list()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.align(4);
        writer.write_i32(1); // FEDS version
        writer.write_pointer_list(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterEffect {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}

impl GMElement for FilterEffect {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_gm_string(self.value)?;
        Ok(())
    }
}
