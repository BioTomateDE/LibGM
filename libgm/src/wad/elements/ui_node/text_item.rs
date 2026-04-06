use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::elements::room;
use crate::wad::elements::ui_node::flex;
use crate::wad::serialize::builder::DataBuilder;

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
