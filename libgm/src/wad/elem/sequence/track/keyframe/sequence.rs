// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::elem::GMElement;
use crate::wad::elem::sequence::GMSequence;
use crate::wad::reference::GMRef;
use crate::wad::build::builder::DataBuilder;
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub sequence: GMRef<GMSequence>,
}

impl GMElement for Sequence {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sequence: GMRef<GMSequence> = reader.read_resource_by_id()?;
        Ok(Self { sequence })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.sequence);
        Ok(())
    }
}
