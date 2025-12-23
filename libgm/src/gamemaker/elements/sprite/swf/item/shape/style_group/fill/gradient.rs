use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, sprite::swf::Matrix33},
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub tpe_index: Option<i32>,
    pub fill_type: FillType,
    pub transformation_matrix: Matrix33,
    pub records: Vec<Record>,
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let fill_type: FillType = num_enum_from(reader.read_i32()?)?;
        let tpe_index: Option<i32> = reader.deserialize_if_gm_version((2022, 1))?;
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
        builder.write_i32(self.fill_type.into());
        self.tpe_index
            .serialize_if_gm_ver(builder, "TPE Index", (2022, 1))?;
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

#[num_enum(i32)]
pub enum FillType {
    FillLinear,
    FillRadial,
}
