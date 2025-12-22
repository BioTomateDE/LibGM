use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, sprite::swf::Matrix33},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub bitmap_fill_type: FillType,
    pub char_id: i32,
    transformation_matrix: Matrix33,
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let bitmap_fill_type: FillType = num_enum_from(reader.read_i32()?)?;
        let char_id = reader.read_i32()?;
        let transformation_matrix = Matrix33::deserialize(reader)?;
        Ok(Self {
            bitmap_fill_type,
            char_id,
            transformation_matrix,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.bitmap_fill_type.into());
        builder.write_i32(self.char_id);
        self.transformation_matrix.serialize(builder)?;
        Ok(())
    }
}

#[num_enum(i32)]
pub enum FillType {
    FillRepeat,
    FillClamp,
    FillRepeatPoint,
    FillClampPoint,
}
