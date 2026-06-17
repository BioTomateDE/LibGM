// SPDX-License-Identifier: GPL-3.0-only

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMGameEndScripts {
    pub elems: Vec<GMRef<GMCode>>,
    pub exists: bool,
}

gm_list_chunk!(GMEN, GMGameEndScripts, GMRef<GMCode>, direct);

impl GMElement for GMGameEndScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<GMRef<GMCode>> = reader.read_simple_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.elems)?;
        Ok(())
    }
}
