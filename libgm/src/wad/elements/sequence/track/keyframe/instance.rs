use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::elements::game_object::GMGameObject;
use crate::wad::reference::GMRef;
use crate::wad::serialize::builder::DataBuilder;
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
