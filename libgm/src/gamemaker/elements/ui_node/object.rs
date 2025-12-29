use super::flex;
use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, room},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
    pub flex_instance_properties: flex::instance::Properties,
    pub room_game_object: room::GameObject,
}

impl GMElement for GameObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let room_game_object = room::GameObject::deserialize(reader)?;
        let flex_instance_properties = flex::instance::Properties::deserialize(reader)?;
        Ok(Self {
            flex_instance_properties,
            room_game_object,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        self.room_game_object.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}
