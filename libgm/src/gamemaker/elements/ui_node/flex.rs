pub mod instance;
mod properties;
mod value;

use macros::num_enum;
pub use properties::Properties;
pub use value::FlexValue;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FlexPanel {
    pub name: String,
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
        let name: String = reader.read_gm_string()?;
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
        let position_type: PositionKind = num_enum_from(reader.read_i32()?)?;
        let align_self: properties::AlignmentKind = num_enum_from(reader.read_i32()?)?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        self.width.serialize(builder)?;
        self.height.serialize(builder)?;
        self.minimum_width.serialize(builder)?;
        self.minimum_height.serialize(builder)?;
        self.maximum_width.serialize(builder)?;
        self.maximum_height.serialize(builder)?;
        self.offset_left.serialize(builder)?;
        self.offset_right.serialize(builder)?;
        self.offset_top.serialize(builder)?;
        self.offset_bottom.serialize(builder)?;
        self.clips_contents.serialize(builder)?;
        builder.write_i32(self.position_type.into());
        builder.write_i32(self.align_self.into());
        self.margin_left.serialize(builder)?;
        self.margin_right.serialize(builder)?;
        self.margin_top.serialize(builder)?;
        self.margin_bottom.serialize(builder)?;
        builder.write_f32(self.flex_grow);
        builder.write_f32(self.flex_shrink);
        self.flex_properties.serialize(builder)?;
        Ok(())
    }
}

#[num_enum(i32)]
pub enum PositionKind {
    Static = 0,
    Relative = 1,
    Absolute = 2,
}
