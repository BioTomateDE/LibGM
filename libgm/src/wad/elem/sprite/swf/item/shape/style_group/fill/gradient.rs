// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::sprite::swf::Matrix33;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub tpe_index: Option<i32>,
    pub fill_type: FillType,
    pub transformation_matrix: Matrix33,
    pub records: Vec<Record>,
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let fill_type: FillType = reader.read_enum()?;
        let tpe_index: Option<i32> = reader.deserialize_if_version(GMVersion::GM2022_1)?;
        let transformation_matrix = Matrix33::deserialize(reader)?;
        let records: Vec<Record> = reader.read_simple_list()?;
        Ok(Self {
            tpe_index,
            fill_type,
            transformation_matrix,
            records,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_enum(self.fill_type);
        builder.write_if_ver(&self.tpe_index, "TPE Index", GMVersion::GM2022_1)?;
        self.transformation_matrix.serialize(builder)?;
        builder.write_simple_list(&self.records)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub ratio: i32,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl GMElement for Record {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let ratio = reader.read_i32()?;
        let red = reader.read_u8()?;
        let green = reader.read_u8()?;
        let blue = reader.read_u8()?;
        let alpha = reader.read_u8()?;
        Ok(Self { ratio, red, green, blue, alpha })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.ratio);
        builder.write_u8(self.red);
        builder.write_u8(self.green);
        builder.write_u8(self.blue);
        builder.write_u8(self.alpha);
        Ok(())
    }
}

gm_enum!(FillType {
    Linear = 0,
    Radial = 1,
});
