use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        chunk::ChunkName,
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::instructions::GMCode,
    prelude::*,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMGlobalInitScripts {
    pub global_init_scripts: Vec<GMRef<GMCode>>,
    pub exists: bool,
}

impl Deref for GMGlobalInitScripts {
    type Target = Vec<GMRef<GMCode>>;
    fn deref(&self) -> &Self::Target {
        &self.global_init_scripts
    }
}

impl DerefMut for GMGlobalInitScripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.global_init_scripts
    }
}

impl GMChunkElement for GMGlobalInitScripts {
    const NAME: ChunkName = ChunkName::new("GLOB");
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMGlobalInitScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let global_init_scripts: Vec<GMRef<GMCode>> = reader.read_simple_list_of_resource_ids()?;
        Ok(Self { global_init_scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list_of_resource_ids(&self.global_init_scripts)?;
        Ok(())
    }
}
