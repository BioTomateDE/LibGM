use std::fmt;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Clone, PartialEq, Eq)]
pub struct CollisionMask {
    pub rle_data: Vec<u8>,
}

impl fmt::Debug for CollisionMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CollisionMask")
    }
}

impl GMElement for CollisionMask {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let rle_length = reader.read_count("YYSWF Collision Mask RLE Data")?;
        let rle_data: Vec<u8> = reader
            .read_bytes_dyn(rle_length)
            .context("reading RLE Data of Timeline")?
            .to_vec();
        reader.align(4)?; // [From UndertaleModTool] "why it's not aligned before the data is beyond my brain"
        Ok(Self { rle_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        // From UTMT: writing zero for empty table would probably be smart but the padding handles it automatically?
        //            but you cant even have a yyswf sprite with a null rle data???
        if !self.rle_data.is_empty() {
            builder.write_usize(self.rle_data.len())?;
            builder.write_bytes(&self.rle_data);
        }
        builder.align(4);
        Ok(())
    }
}
