use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, room},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectLayer {
    pub enabled: bool,
    pub effect_type: String,
    pub properties: Vec<room::layer::effect::Property>,
}

impl GMElement for EffectLayer {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let enabled = reader.read_bool32()?;
        let effect_type: String = reader.read_gm_string()?;
        let properties: Vec<room::layer::effect::Property> = reader.read_pointer_list()?;
        Ok(Self { enabled, effect_type, properties })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.enabled);
        builder.write_gm_string(&self.effect_type);
        builder.write_pointer_list(&self.properties)?;
        Ok(())
    }
}
