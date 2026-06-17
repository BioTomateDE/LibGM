// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMFeatureFlags {
    pub elems: Vec<GMRef<String>>,
    pub exists: bool,
}

gm_list_chunk!(FEAT, GMFeatureFlags, GMRef<String>, direct);

impl GMElement for GMFeatureFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let elems: Vec<GMRef<String>> = reader.read_simple_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_simple_list(&self.elems)?;
        Ok(())
    }
}
