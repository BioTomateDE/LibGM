// SPDX-License-Identifier: GPL-3.0-only
use macros::list_chunk;

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::elem::GMElement;
use crate::wad::reference::GMRef;
use crate::wad::build::builder::DataBuilder;

#[list_chunk("GMEN")]
pub struct GMGameEndScripts {
    pub game_end_scripts: Vec<GMRef<GMCode>>,
    pub exists: bool,
}

impl GMElement for GMGameEndScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let game_end_scripts: Vec<GMRef<GMCode>> = reader.read_simple_list()?;
        Ok(Self { game_end_scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.game_end_scripts)?;
        Ok(())
    }
}
