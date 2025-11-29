use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::instructions::GMCode,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMGameEndScripts {
    pub game_end_scripts: Vec<GMRef<GMCode>>,
    pub exists: bool,
}

impl Deref for GMGameEndScripts {
    type Target = Vec<GMRef<GMCode>>;
    fn deref(&self) -> &Self::Target {
        &self.game_end_scripts
    }
}

impl DerefMut for GMGameEndScripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game_end_scripts
    }
}

impl GMChunkElement for GMGameEndScripts {
    const NAME: &'static str = "GMEN";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMGameEndScripts {
    fn deserialize(reader: &mut DataReader) -> crate::error::Result<Self> {
        let game_end_scripts: Vec<GMRef<GMCode>> = reader.read_simple_list_of_resource_ids()?;
        Ok(Self { game_end_scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> crate::error::Result<()> {
        builder.write_simple_list_of_resource_ids(&self.game_end_scripts)?;
        Ok(())
    }
}
