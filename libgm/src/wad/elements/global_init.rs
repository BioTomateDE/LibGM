use macros::list_chunk;

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::reference::GMRef;
use crate::wad::serialize::builder::DataBuilder;

#[list_chunk("GLOB")]
pub struct GMGlobalInitScripts {
    pub global_init_scripts: Vec<GMRef<GMCode>>,
    pub exists: bool,
}

impl GMElement for GMGlobalInitScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let global_init_scripts: Vec<GMRef<GMCode>> = reader.read_simple_list()?;
        Ok(Self { global_init_scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.global_init_scripts)?;
        Ok(())
    }
}
