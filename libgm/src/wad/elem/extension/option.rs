// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtOption {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
    pub kind: Kind,
}

impl GMElement for ExtOption {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        let kind: Kind = reader.read_enum()?;
        Ok(Self { name, value, kind })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_gm_string(self.value)?;
        builder.write_enum(self.kind);
        Ok(())
    }
}

gm_enum!(Kind {
    Boolean = 0,
    Number = 1,
    String = 2,
});
