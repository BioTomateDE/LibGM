// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KString {
    pub string: GMRef<String>,
}

impl GMElement for KString {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let string: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { string })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.string)?;
        Ok(())
    }
}
