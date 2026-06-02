// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::sprite::swf::Matrix33;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub bitmap_fill_type: FillType,
    pub char_id: i32,
    transformation_matrix: Matrix33,
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let bitmap_fill_type: FillType = reader.read_enum()?;
        let char_id = reader.read_i32()?;
        let transformation_matrix = Matrix33::deserialize(reader)?;
        Ok(Self {
            bitmap_fill_type,
            char_id,
            transformation_matrix,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_enum(self.bitmap_fill_type);
        builder.write_i32(self.char_id);
        self.transformation_matrix.serialize(builder)?;
        Ok(())
    }
}

gm_enum!(FillType {
    Repeat = 0,
    Clamp = 1,
    RepeatPoint = 2,
    ClampPoint = 3,
});
