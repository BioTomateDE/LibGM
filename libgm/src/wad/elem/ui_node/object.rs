// SPDX-License-Identifier: GPL-3.0-only
use super::flex;
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::room::RoomGameObject;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
    pub flex_instance_properties: flex::instance::Properties,
    pub room_game_object: RoomGameObject,
}

impl GMElement for GameObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let room_game_object = RoomGameObject::deserialize(reader)?;
        let flex_instance_properties = flex::instance::Properties::deserialize(reader)?;
        Ok(Self { flex_instance_properties, room_game_object })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_elem(&self.room_game_object)?;
        writer.write_elem(&self.flex_instance_properties)?;
        Ok(())
    }
}
