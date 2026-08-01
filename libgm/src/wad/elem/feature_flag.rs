// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FeatureFlags {
    pub elems: Vec<GMRef<String>>,
}

gm_list_chunk!(FEAT, FeatureFlags, GMRef<String>);

impl GMElement for FeatureFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let elems: Vec<GMRef<String>> = reader.read_simple_list()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.align(4);
        writer.write_simple_list(&self.elems)?;
        Ok(())
    }
}
