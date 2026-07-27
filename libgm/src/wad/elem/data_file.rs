// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::chunk::ChunkName;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

/// This is the always-empty unused chunk `DAFL`.
///
/// It is unrelated to the `GMData` struct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataFiles;

impl GMChunk for DataFiles {
    const NAME: ChunkName = ChunkName::DAFL;
}

// Very cool
impl GMElement for DataFiles {
    fn deserialize(_: &mut DataReader) -> Result<Self> {
        Ok(Self)
    }

    fn serialize(&self, _: &mut DataWriter) -> Result<()> {
        Ok(())
    }
}
