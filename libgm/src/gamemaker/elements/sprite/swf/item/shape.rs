mod style_group;
pub use style_group::StyleGroup;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Data<T> {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub style_groups: Vec<StyleGroup<T>>,
}
impl<T: GMElement> GMElement for Data<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let min_x = reader.read_f32()?;
        let max_x = reader.read_f32()?;
        let min_y = reader.read_f32()?;
        let max_y = reader.read_f32()?;
        let style_groups: Vec<StyleGroup<T>> = reader.read_simple_list()?;
        Ok(Self { min_x, max_x, min_y, max_y, style_groups })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        builder.write_simple_list(&self.style_groups)?;
        Ok(())
    }
}
