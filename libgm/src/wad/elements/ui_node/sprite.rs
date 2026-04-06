use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::elements::room;
use crate::wad::elements::ui_node::flex;
use crate::wad::serialize::builder::DataBuilder;
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
