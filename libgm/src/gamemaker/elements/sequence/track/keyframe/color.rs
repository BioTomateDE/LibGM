use super::Keyframe;
use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframesData<T> {
    pub interpolation: i32,
    pub keyframes: Vec<Keyframe<T>>,
}
impl<T: GMElement> GMElement for KeyframesData<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let interpolation = reader.read_i32()?;
        let keyframes: Vec<Keyframe<T>> = reader.read_simple_list()?;
        Ok(Self { interpolation, keyframes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(self.interpolation);
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
