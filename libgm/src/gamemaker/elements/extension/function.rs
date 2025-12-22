use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, extension::GMExtensionKind},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub id: u32,
    pub kind: GMExtensionKind,
    pub return_type: ReturnType,
    pub ext_name: String,
    pub arguments: Vec<Argument>,
}

impl GMElement for Function {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let id = reader.read_u32()?;
        let kind: GMExtensionKind = num_enum_from(reader.read_i32()?)?; // Assumption; UTMT uses u32
        let return_type: ReturnType = num_enum_from(reader.read_i32()?)?;
        let ext_name: String = reader.read_gm_string()?;
        let arguments: Vec<Argument> = reader.read_simple_list()?;
        Ok(Self {
            name,
            id,
            kind,
            return_type,
            ext_name,
            arguments,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_u32(self.id);
        builder.write_i32(self.kind.into());
        builder.write_i32(self.return_type.into());
        builder.write_gm_string(&self.ext_name);
        builder.write_simple_list(&self.arguments)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub return_type: ReturnType,
}

impl GMElement for Argument {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let return_type: ReturnType = num_enum_from(reader.read_i32()?)?;
        Ok(Self { return_type })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.return_type.into());
        Ok(())
    }
}

#[num_enum(i32)]
pub enum ReturnType {
    String = 1,
    Double = 2,
}
