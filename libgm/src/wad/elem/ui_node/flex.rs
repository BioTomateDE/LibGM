// SPDX-License-Identifier: GPL-3.0-only
pub mod instance;
mod properties;
mod value;

pub use properties::Properties;
pub use value::FlexValue;

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct FlexPanel {
    pub name: GMRef<String>,
    pub width: FlexValue,
    pub height: FlexValue,
    pub minimum_width: FlexValue,
    pub minimum_height: FlexValue,
    pub maximum_width: FlexValue,
    pub maximum_height: FlexValue,
    pub offset_left: FlexValue,
    pub offset_right: FlexValue,
    pub offset_top: FlexValue,
    pub offset_bottom: FlexValue,
    pub clips_contents: bool,
    pub position_type: PositionKind,
    pub align_self: properties::AlignmentKind,
    pub margin_left: FlexValue,
    pub margin_right: FlexValue,
    pub margin_top: FlexValue,
    pub margin_bottom: FlexValue,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_properties: Properties,
}

impl GMElement for FlexPanel {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let width = FlexValue::deserialize(reader)?;
        let height = FlexValue::deserialize(reader)?;
        let minimum_width = FlexValue::deserialize(reader)?;
        let minimum_height = FlexValue::deserialize(reader)?;
        let maximum_width = FlexValue::deserialize(reader)?;
        let maximum_height = FlexValue::deserialize(reader)?;
        let offset_left = FlexValue::deserialize(reader)?;
        let offset_right = FlexValue::deserialize(reader)?;
        let offset_top = FlexValue::deserialize(reader)?;
        let offset_bottom = FlexValue::deserialize(reader)?;
        let clips_contents = reader.read_bool32()?;
        let position_type: PositionKind = reader.read_enum()?;
        let align_self: properties::AlignmentKind = reader.read_enum()?;
        let margin_left = FlexValue::deserialize(reader)?;
        let margin_right = FlexValue::deserialize(reader)?;
        let margin_top = FlexValue::deserialize(reader)?;
        let margin_bottom = FlexValue::deserialize(reader)?;
        let flex_grow = reader.read_f32()?;
        let flex_shrink = reader.read_f32()?;
        let flex_properties = Properties::deserialize(reader)?;
        Ok(Self {
            name,
            width,
            height,
            minimum_width,
            minimum_height,
            maximum_width,
            maximum_height,
            offset_left,
            offset_right,
            offset_top,
            offset_bottom,
            clips_contents,
            position_type,
            align_self,
            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            flex_grow,
            flex_shrink,
            flex_properties,
        })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_elem(&self.width)?;
        writer.write_elem(&self.height)?;
        writer.write_elem(&self.minimum_width)?;
        writer.write_elem(&self.minimum_height)?;
        writer.write_elem(&self.maximum_width)?;
        writer.write_elem(&self.maximum_height)?;
        writer.write_elem(&self.offset_left)?;
        writer.write_elem(&self.offset_right)?;
        writer.write_elem(&self.offset_top)?;
        writer.write_elem(&self.offset_bottom)?;
        writer.write_elem(&self.clips_contents)?;
        writer.write_enum(self.position_type);
        writer.write_enum(self.align_self);
        writer.write_elem(&self.margin_left)?;
        writer.write_elem(&self.margin_right)?;
        writer.write_elem(&self.margin_top)?;
        writer.write_elem(&self.margin_bottom)?;
        writer.write_f32(self.flex_grow);
        writer.write_f32(self.flex_shrink);
        writer.write_elem(&self.flex_properties)?;
        Ok(())
    }
}

gm_enum!(PositionKind {
    Static = 0,
    Relative = 1,
    Absolute = 2,
});
