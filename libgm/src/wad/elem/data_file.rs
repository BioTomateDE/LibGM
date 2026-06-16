// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

/// This is the always-empty unused chunk `DAFL`.
///
/// It is unrelated to the `GMData` struct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMDataFiles {
    exists: bool,
}

gm_chunk!(DAFL, GMDataFiles);

// Very cool
impl GMElement for GMDataFiles {
    fn deserialize(_: &mut DataReader) -> Result<Self> {
        Ok(Self { exists: true })
    }

    fn serialize(&self, _: &mut DataBuilder) -> Result<()> {
        Ok(())
    }
}
