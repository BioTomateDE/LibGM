// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionMask {
    pub rle_data: Blob<Vec<u8>>,
}

impl GMElement for CollisionMask {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let rle_length = reader.read_count("YYSWF Collision Mask RLE Data")?;
        let rle_data: Vec<u8> =
            reader.read_bytes_dyn(rle_length).ctx("reading RLE Data of Timeline")?.to_vec();
        reader.align(4)?; // [From UndertaleModTool] "why it's not aligned before the data is beyond my brain"
        Ok(Self { rle_data: Blob(rle_data) })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        // From UTMT: "writing zero for empty table would probably be smart but the
        // padding handles it automatically?"
        // but you cant even have a yyswf sprite with a null rle data???
        if !self.rle_data.is_empty() {
            writer.write_usize(self.rle_data.len())?;
            writer.write_bytes(&self.rle_data);
        }
        writer.align(4);
        Ok(())
    }
}
