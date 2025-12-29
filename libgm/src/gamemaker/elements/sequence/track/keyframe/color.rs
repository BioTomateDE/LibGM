use macros::num_enum;

use super::Keyframe;
use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct KeyframesData {
    pub interpolation: InterpolationMode,
    pub keyframes: Vec<Keyframe<Color>>,
}

impl GMElement for KeyframesData {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let interpolation = num_enum_from(reader.read_i32()?)?;
        let keyframes: Vec<Keyframe<Color>> = reader.read_simple_list()?;
        Ok(Self { interpolation, keyframes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(self.interpolation.into());
        builder.write_simple_list(&self.keyframes)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub value: f32,
}

impl GMElement for Color {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let value = reader.read_f32()?;
        Ok(Self { value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.value);
        Ok(())
    }
}

#[num_enum(i32)]
pub enum InterpolationMode {
    None = 0,
    Linear = 1,
}
