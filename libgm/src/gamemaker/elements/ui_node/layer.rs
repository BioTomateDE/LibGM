use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub name: String,
    pub draw_space: DrawSpaceKind,
    pub visible: bool,
}

impl GMElement for Layer {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let draw_space: DrawSpaceKind = num_enum_from(reader.read_i32()?)?;
        let visible = reader.read_bool32()?;
        Ok(Self { name, draw_space, visible })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_i32(self.draw_space.into());
        builder.write_bool32(self.visible);
        Ok(())
    }
}

#[num_enum(i32)]
pub enum DrawSpaceKind {
    GUI = 1,
    View = 2,
}
