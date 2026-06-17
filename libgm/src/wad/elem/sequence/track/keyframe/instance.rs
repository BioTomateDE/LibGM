// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::game_object::GameObject;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub game_object: GMRef<GameObject>,
}

impl GMElement for Instance {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let game_object: GMRef<GameObject> = reader.read_resource_by_id()?;
        Ok(Self { game_object })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.game_object);
        Ok(())
    }
}
