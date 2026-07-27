// SPDX-License-Identifier: GPL-3.0-only
pub mod bitmap;
pub mod gradient;
pub mod solid;

use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

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
                "Invalid YYSWF Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF Fill \
                 Data",
                fill_type,
                reader.cur_pos,
            ),
        };
        Ok(fill_data)
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i32(match self {
            Self::Solid(_) => 1,
            Self::Gradient(_) => 2,
            Self::Bitmap(_) => 3,
        });
        match self {
            Self::Solid(solid_data) => writer.write_elem(solid_data)?,
            Self::Gradient(gradient_data) => writer.write_elem(gradient_data)?,
            Self::Bitmap(bitmap_data) => writer.write_elem(bitmap_data)?,
        }
        Ok(())
    }
}
