use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, room, ui_node::flex},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceInstance {
    pub flex_instance_properties: flex::instance::Properties,
    pub sequence_instance: room::layer::assets::SequenceInstance,
}

impl GMElement for SequenceInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sequence_instance = room::layer::assets::SequenceInstance::deserialize(reader)?;
        let flex_instance_properties = flex::instance::Properties::deserialize(reader)?;
        Ok(Self {
            flex_instance_properties,
            sequence_instance,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        self.sequence_instance.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}
