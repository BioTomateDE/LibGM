// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::Code;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobalInitScripts {
    pub elems: Vec<GMRef<Code>>,
}

gm_list_chunk!(GLOB, GlobalInitScripts, GMRef<Code>, direct);

impl GMElement for GlobalInitScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<GMRef<Code>> = reader.read_simple_list()?;
        Ok(Self { elems })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.elems)?;
        Ok(())
    }
}
