// SPDX-License-Identifier: GPL-3.0-only

use super::value::FlexValue;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

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
        let align_items = reader.read_enum()?;
        let flex_direction = reader.read_enum()?;
        let flex_wrap = reader.read_enum()?;
        let align_content = reader.read_enum()?;
        let gap_row = reader.read_f32()?;
        let gap_column = reader.read_f32()?;
        let padding_left = FlexValue::deserialize(reader)?;
        let padding_right = FlexValue::deserialize(reader)?;
        let padding_top = FlexValue::deserialize(reader)?;
        let padding_bottom = FlexValue::deserialize(reader)?;
        let justify_content = reader.read_enum()?;
        let layout_direction = reader.read_enum()?;
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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_enum(self.align_items);
        writer.write_enum(self.flex_direction);
        writer.write_enum(self.flex_wrap);
        writer.write_enum(self.align_content);
        writer.write_f32(self.gap_row);
        writer.write_f32(self.gap_column);
        writer.write_elem(&self.padding_left)?;
        writer.write_elem(&self.padding_right)?;
        writer.write_elem(&self.padding_top)?;
        writer.write_elem(&self.padding_bottom)?;
        writer.write_enum(self.justify_content);
        writer.write_enum(self.layout_direction);
        Ok(())
    }
}

gm_enum!(AlignmentKind {
    Auto = 0,
    FlexStart = 1,
    Center = 2,
    FlexEnd = 3,
    Stretch = 4,
    Baseline = 5,
    SpaceBetween = 6,
    SpaceAround = 7,
    SpaceEvenly = 8,
});

gm_enum!(FlexDirectionKind {
    Column = 0,
    ColumnReverse = 1,
    Row = 2,
    RowReverse = 3,
});

gm_enum!(WrapKind {
    NoWrap = 0,
    Wrap = 1,
    WrapReverse = 2,
});

gm_enum!(JustifyKind {
    FlexStart = 0,
    Center = 1,
    FlexEnd = 2,
    SpaceBetween = 3,
    SpaceAround = 4,
    SpaceEvenly = 5,
});

gm_enum!(LayoutDirectionKind {
    Inherit = 0,
    LeftToRight = 1,
    RightToLeft = 2,
});
