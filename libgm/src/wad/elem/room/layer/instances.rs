// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::elem::GMElement;
use crate::wad::build::builder::DataBuilder;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instances {
    pub instances: Vec<u32>,
}

impl GMElement for Instances {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let instances: Vec<u32> = reader.read_simple_list()?;
        Ok(Self { instances })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.instances)?;
        Ok(())
    }
}
