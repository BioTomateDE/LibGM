use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, sprite::GMSprite},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Graphic {
    pub sprite: GMRef<GMSprite>,
}

impl GMElement for Graphic {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sprite: GMRef<GMSprite> = reader.read_resource_by_id()?;
        Ok(Self { sprite })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.sprite);
        Ok(())
    }
}
