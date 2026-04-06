use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::elements::sprite::GMSprite;
use crate::wad::reference::GMRef;
use crate::wad::serialize::builder::DataBuilder;
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
