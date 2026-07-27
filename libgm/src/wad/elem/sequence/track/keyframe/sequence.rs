// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::sequence::Sequence;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;
#[derive(Debug, Clone, PartialEq)]
pub struct KSequence {
    pub sequence: GMRef<Sequence>,
}

impl GMElement for KSequence {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sequence: GMRef<Sequence> = reader.read_resource_by_id()?;
        Ok(Self { sequence })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_resource_id(self.sequence);
        Ok(())
    }
}
