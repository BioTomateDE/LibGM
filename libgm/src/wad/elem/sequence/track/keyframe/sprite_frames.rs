// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteFrames {
    pub value: i32,
}

impl GMElement for SpriteFrames {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let value = reader.read_i32()?;
        Ok(Self { value })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i32(self.value);
        Ok(())
    }
}
