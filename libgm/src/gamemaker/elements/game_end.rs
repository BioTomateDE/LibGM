use macros::list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::instruction::GMCode,
    prelude::*,
};

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
