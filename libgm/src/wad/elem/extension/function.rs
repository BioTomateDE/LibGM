// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::extension::Kind;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: GMRef<String>,
    pub id: u32,
    pub kind: Kind,
    pub return_type: ReturnType,
    pub ext_name: GMRef<String>,
    pub arguments: Vec<Argument>,
}

impl GMElement for Function {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let id = reader.read_u32()?;
        let kind: Kind = reader.read_enum()?; // Assumption; UTMT uses u32
        let return_type: ReturnType = reader.read_enum()?;
        let ext_name: GMRef<String> = reader.read_gm_string()?;
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
        builder.write_gm_string(self.name)?;
        builder.write_u32(self.id);
        builder.write_enum(self.kind);
        builder.write_enum(self.return_type);
        builder.write_gm_string(self.ext_name)?;
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
        let return_type: ReturnType = reader.read_enum()?;
        Ok(Self { return_type })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_enum(self.return_type);
        Ok(())
    }
}

gm_enum!(ReturnType {
    String = 1,
    Double = 2,
});
