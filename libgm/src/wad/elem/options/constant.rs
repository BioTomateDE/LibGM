// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::string::Strings;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}

impl GMElement for Constant {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_gm_string(self.value)?;
        Ok(())
    }
}

impl Constant {
    #[must_use]
    pub fn new(name: &str, value: &str, strings: &mut Strings) -> Self {
        Self {
            name: strings.make(name),
            value: strings.make(value),
        }
    }
}
