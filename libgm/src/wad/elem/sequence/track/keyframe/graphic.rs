// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::sprite::Sprite;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;
#[derive(Debug, Clone, PartialEq)]
pub struct Graphic {
    pub sprite: GMRef<Sprite>,
}

impl GMElement for Graphic {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sprite: GMRef<Sprite> = reader.read_resource_by_id()?;
        Ok(Self { sprite })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.sprite);
        Ok(())
    }
}
