// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::elem::GMElement;
use crate::wad::elem::room;
use crate::wad::elem::ui_node::flex;
use crate::wad::build::builder::DataBuilder;

#[derive(Debug, Clone, PartialEq)]
pub struct TextItemInstance {
    pub flex_instance_properties: flex::instance::Properties,
    pub text_item_instance: room::layer::assets::TextItemInstance,
}

impl GMElement for TextItemInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let text_item_instance = room::layer::assets::TextItemInstance::deserialize(reader)?;
        let flex_instance_properties = flex::instance::Properties::deserialize(reader)?;
        Ok(Self {
            flex_instance_properties,
            text_item_instance,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        self.text_item_instance.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}
