use crate::prelude::*;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMFilterEffects {
    pub filter_effects: Vec<GMFilterEffect>,
    pub exists: bool,
}

impl GMChunkElement for GMFilterEffects {
    fn stub() -> Self {
        Self { filter_effects: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMFilterEffects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let version = reader.read_i32()?;
        if version != 1 {
            bail!("Expected FEDS version 1 but got {version}");
        }
        let filter_effects: Vec<GMFilterEffect> = reader.read_pointer_list()?;
        Ok(Self { filter_effects, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1);   // FEDS version
        builder.write_pointer_list(&self.filter_effects)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMFilterEffect {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}

impl GMElement for GMFilterEffect {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string(&self.value)?;
        Ok(())
    }
}

