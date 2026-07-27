// SPDX-License-Identifier: GPL-3.0-only
mod style_group;
pub use style_group::StyleGroup;

use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;
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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_f32(self.min_x);
        writer.write_f32(self.max_x);
        writer.write_f32(self.min_y);
        writer.write_f32(self.max_y);
        writer.write_simple_list(&self.style_groups)?;
        Ok(())
    }
}
