use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::elem::GMElement;
use crate::wad::elem::sprite::GMSprite;
use crate::wad::reference::GMRef;
use crate::wad::build::builder::DataBuilder;
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
