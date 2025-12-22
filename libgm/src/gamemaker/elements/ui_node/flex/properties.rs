use macros::num_enum;

use super::value::FlexValue;
use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Properties {
    align_items: AlignmentKind,
    flex_direction: FlexDirectionKind,
    flex_wrap: WrapKind,
    align_content: AlignmentKind,
    gap_row: f32,
    gap_column: f32,
    padding_left: FlexValue,
    padding_right: FlexValue,
    padding_top: FlexValue,
    padding_bottom: FlexValue,
    justify_content: JustifyKind,
    layout_direction: LayoutDirectionKind,
}

impl GMElement for Properties {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let align_items = num_enum_from(reader.read_i32()?)?;
        let flex_direction = num_enum_from(reader.read_i32()?)?;
        let flex_wrap = num_enum_from(reader.read_i32()?)?;
        let align_content = num_enum_from(reader.read_i32()?)?;
        let gap_row = reader.read_f32()?;
        let gap_column = reader.read_f32()?;
        let padding_left = FlexValue::deserialize(reader)?;
        let padding_right = FlexValue::deserialize(reader)?;
        let padding_top = FlexValue::deserialize(reader)?;
        let padding_bottom = FlexValue::deserialize(reader)?;
        let justify_content = num_enum_from(reader.read_i32()?)?;
        let layout_direction = num_enum_from(reader.read_i32()?)?;
        Ok(Self {
            align_items,
            flex_direction,
            flex_wrap,
            align_content,
            gap_row,
            gap_column,
            padding_left,
            padding_right,
            padding_top,
            padding_bottom,
            justify_content,
            layout_direction,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.align_items.into());
        builder.write_i32(self.flex_direction.into());
        builder.write_i32(self.flex_wrap.into());
        builder.write_i32(self.align_content.into());
        builder.write_f32(self.gap_row);
        builder.write_f32(self.gap_column);
        self.padding_left.serialize(builder)?;
        self.padding_right.serialize(builder)?;
        self.padding_top.serialize(builder)?;
        self.padding_bottom.serialize(builder)?;
        builder.write_i32(self.justify_content.into());
        builder.write_i32(self.layout_direction.into());
        Ok(())
    }
}

#[num_enum(i32)]
pub enum AlignmentKind {
    Auto = 0,
    FlexStart = 1,
    Center = 2,
    FlexEnd = 3,
    Stretch = 4,
    Baseline = 5,
    SpaceBetween = 6,
    SpaceAround = 7,
    SpaceEvenly = 8,
}

#[num_enum(i32)]
pub enum FlexDirectionKind {
    Column = 0,
    ColumnReverse = 1,
    Row = 2,
    RowReverse = 3,
}

#[num_enum(i32)]
pub enum WrapKind {
    NoWrap = 0,
    Wrap = 1,
    WrapReverse = 2,
}

#[num_enum(i32)]
pub enum JustifyKind {
    FlexStart = 0,
    Center = 1,
    FlexEnd = 2,
    SpaceBetween = 3,
    SpaceAround = 4,
    SpaceEvenly = 5,
}

#[num_enum(i32)]
pub enum LayoutDirectionKind {
    Inherit = 0,
    LeftToRight = 1,
    RightToLeft = 2,
}
