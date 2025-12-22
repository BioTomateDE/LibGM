use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, room, ui_node::flex},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteInstance {
    pub flex_instance_properties: flex::instance::Properties,
    pub sprite_instance: room::layer::assets::SpriteInstance,
}

impl GMElement for SpriteInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sprite_instance = room::layer::assets::SpriteInstance::deserialize(reader)?;
        let flex_instance_properties = flex::instance::Properties::deserialize(reader)?;
        Ok(Self {
            flex_instance_properties,
            sprite_instance,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        self.sprite_instance.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}
