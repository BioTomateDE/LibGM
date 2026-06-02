// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::extension::Kind;
use crate::wad::elem::extension::function::Function;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub filename: GMRef<String>,
    pub cleanup_script: GMRef<String>,
    pub init_script: GMRef<String>,
    pub kind: Kind,
    pub functions: Vec<Function>,
}

impl GMElement for File {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let filename: GMRef<String> = reader.read_gm_string()?;
        let cleanup_script: GMRef<String> = reader.read_gm_string()?;
        let init_script: GMRef<String> = reader.read_gm_string()?;
        let kind: Kind = reader.read_enum()?;
        let functions: Vec<Function> = reader.read_pointer_list()?;
        Ok(Self {
            filename,
            cleanup_script,
            init_script,
            kind,
            functions,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.filename)?;
        builder.write_gm_string(self.cleanup_script)?;
        builder.write_gm_string(self.init_script)?;
        builder.write_enum(self.kind);
        builder.write_pointer_list(&self.functions)?;
        Ok(())
    }
}
