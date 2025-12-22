use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[named_list_chunk("FEDS")]
pub struct GMFilterEffects {
    pub filter_effects: Vec<GMFilterEffect>,
    pub exists: bool,
}

impl GMElement for GMFilterEffects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("FEDS Version")?;
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
