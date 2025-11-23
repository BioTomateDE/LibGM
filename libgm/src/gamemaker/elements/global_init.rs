use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::reference::GMRef;
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::gml::instructions::GMCode;
use crate::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
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
    const NAME: &'static str = "GLOB";
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
