use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::assert::assert_int,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMFilterEffects {
    pub filter_effects: Vec<GMFilterEffect>,
    pub exists: bool,
}

impl Deref for GMFilterEffects {
    type Target = Vec<GMFilterEffect>;
    fn deref(&self) -> &Self::Target {
        &self.filter_effects
    }
}

impl DerefMut for GMFilterEffects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.filter_effects
    }
}

impl GMChunkElement for GMFilterEffects {
    const NAME: &'static str = "FEDS";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMFilterEffects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        assert_int("FEDS Version", 1, reader.read_u32()?)?;
        let filter_effects: Vec<GMFilterEffect> = reader.read_pointer_list()?;
        Ok(Self { filter_effects, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1); // FEDS version
        builder.write_pointer_list(&self.filter_effects)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMFilterEffect {
    pub name: String,
    pub value: String,
}

impl GMElement for GMFilterEffect {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let value: String = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_gm_string(&self.value);
        Ok(())
    }
}
