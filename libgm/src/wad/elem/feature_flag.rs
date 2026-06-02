// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMFeatureFlags {
    pub feature_flags: Vec<GMRef<String>>,
    pub exists: bool,
}

gm_list_chunk!(FEAT, GMFeatureFlags, GMRef<String>, feature_flags, direct);

impl GMElement for GMFeatureFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let feature_flags: Vec<GMRef<String>> = reader.read_simple_list()?;
        Ok(Self { feature_flags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_simple_list(&self.feature_flags)?;
        Ok(())
    }
}
