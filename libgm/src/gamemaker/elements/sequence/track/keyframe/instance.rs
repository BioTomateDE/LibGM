use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, game_object::GMGameObject},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub game_object: GMRef<GMGameObject>,
}

impl GMElement for Instance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let game_object: GMRef<GMGameObject> = reader.read_resource_by_id()?;
        Ok(Self { game_object })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.game_object);
        Ok(())
    }
}
