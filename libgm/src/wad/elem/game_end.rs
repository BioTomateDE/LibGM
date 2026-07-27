// SPDX-License-Identifier: GPL-3.0-only

use crate::gml::Code;
use crate::prelude::*;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GameEndScripts {
    pub elems: Vec<GMRef<Code>>,
}

gm_list_chunk!(GMEN, GameEndScripts, GMRef<Code>, direct);

impl GMElement for GameEndScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<GMRef<Code>> = reader.read_simple_list()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_simple_list(&self.elems)?;
        Ok(())
    }
}
