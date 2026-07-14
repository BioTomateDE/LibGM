// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionMask {
    pub rle_data: Blob<Vec<u8>>,
}

impl GMElement for CollisionMask {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let rle_length = reader.read_count("YYSWF Collision Mask RLE Data")?;
        let rle_data: Vec<u8> = reader
            .read_bytes_dyn(rle_length)
            .ctx("reading RLE Data of Timeline")?
            .to_vec();
        reader.align(4)?; // [From UndertaleModTool] "why it's not aligned before the data is beyond my brain"
        Ok(Self { rle_data: Blob(rle_data) })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        // From UTMT: "writing zero for empty table would probably be smart but the
        // padding handles it automatically?"
        // but you cant even have a yyswf sprite with a null rle data???
        if !self.rle_data.is_empty() {
            builder.write_usize(self.rle_data.len())?;
            builder.write_bytes(&self.rle_data);
        }
        builder.align(4);
        Ok(())
    }
}
