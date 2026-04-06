use macros::num_enum;

use crate::prelude::*;
use crate::util::init::num_enum_from;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Option {
    pub name: String,
    pub value: String,
    pub kind: Kind,
}

impl GMElement for Option {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let value: String = reader.read_gm_string()?;
        let kind: Kind = num_enum_from(reader.read_i32()?)?;
        Ok(Self { name, value, kind })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_gm_string(&self.value);
        builder.write_i32(self.kind.into());
        Ok(())
    }
}

#[num_enum(i32)]
pub enum Kind {
    Boolean = 0,
    Number = 1,
    String = 2,
}
