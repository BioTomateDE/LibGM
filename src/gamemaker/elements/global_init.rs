use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMGlobalInitScripts {
    pub global_init_scripts: Vec<GMRef<GMCode>>,
    pub exists: bool,
}
impl GMChunkElement for GMGlobalInitScripts {
    fn empty() -> Self {
        Self { global_init_scripts: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMGlobalInitScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let global_init_scripts: Vec<GMRef<GMCode>> = reader.read_simple_list_of_resource_ids()?;
        Ok(Self { global_init_scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_simple_list_of_resource_ids(&self.global_init_scripts)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMGameEndScripts {
    pub game_end_scripts: Vec<GMRef<GMCode>>,
    pub exists: bool,
}
impl GMChunkElement for GMGameEndScripts {
    fn empty() -> Self {
        Self { game_end_scripts: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMGameEndScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let game_end_scripts: Vec<GMRef<GMCode>> = reader.read_simple_list_of_resource_ids()?;
        Ok(Self { game_end_scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_simple_list_of_resource_ids(&self.game_end_scripts)?;
        Ok(())
    }
}

