pub mod bitmap;
pub mod gradient;
pub mod solid;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Solid(solid::Data),
    Gradient(gradient::Data),
    Bitmap(bitmap::Data),
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let fill_type = reader.read_i32()?;
        let fill_data = match fill_type {
            1 => Self::Solid(solid::Data::deserialize(reader)?),
            2 => Self::Gradient(gradient::Data::deserialize(reader)?),
            3 => Self::Bitmap(bitmap::Data::deserialize(reader)?),
            _ => bail!(
                "Invalid YYSWF Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF Fill Data",
                fill_type,
                reader.cur_pos,
            ),
        };
        Ok(fill_data)
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(match self {
            Self::Solid(_) => 1,
            Self::Gradient(_) => 2,
            Self::Bitmap(_) => 3,
        });
        match self {
            Self::Solid(solid_data) => solid_data.serialize(builder)?,
            Self::Gradient(gradient_data) => gradient_data.serialize(builder)?,
            Self::Bitmap(bitmap_data) => bitmap_data.serialize(builder)?,
        }
        Ok(())
    }
}
