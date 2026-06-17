// SPDX-License-Identifier: GPL-3.0-only
use super::flex;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::room::RoomGameObject;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
    pub flex_instance_properties: flex::instance::Properties,
    pub room_game_object: RoomGameObject,
}

impl GMElement for GameObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let room_game_object = RoomGameObject::deserialize(reader)?;
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
