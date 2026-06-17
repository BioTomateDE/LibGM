// SPDX-License-Identifier: GPL-3.0-only

use crate::gml::Code;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GameEndScripts {
    pub elems: Vec<GMRef<Code>>,
    pub exists: bool,
}

gm_list_chunk!(GMEN, GameEndScripts, GMRef<Code>, direct);

impl GMElement for GameEndScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<GMRef<Code>> = reader.read_simple_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.elems)?;
        Ok(())
    }
}
