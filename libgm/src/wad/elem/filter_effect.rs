// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMFilterEffects {
    pub elems: Vec<GMFilterEffect>,
    pub exists: bool,
}

gm_named_list_chunk!(FEDS, GMFilterEffects, GMFilterEffect, direct);

impl GMElement for GMFilterEffects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("FEDS Version")?;
        let elems: Vec<GMFilterEffect> = reader.read_pointer_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1); // FEDS version
        builder.write_pointer_list(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMFilterEffect {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}

impl GMElement for GMFilterEffect {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_gm_string(self.value)?;
        Ok(())
    }
}
