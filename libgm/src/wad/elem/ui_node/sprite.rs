// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::room;
use crate::wad::elem::ui_node::flex;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteInstance {
    pub flex_instance_properties: flex::instance::Properties,
    pub sprite_instance: room::layer::assets::SpriteInstance,
}

impl GMElement for SpriteInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sprite_instance = room::layer::assets::SpriteInstance::deserialize(reader)?;
        let flex_instance_properties = flex::instance::Properties::deserialize(reader)?;
        Ok(Self { flex_instance_properties, sprite_instance })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_elem(&self.sprite_instance)?;
        writer.write_elem(&self.flex_instance_properties)?;
        Ok(())
    }
}
