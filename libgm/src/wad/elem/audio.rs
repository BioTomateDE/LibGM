// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Audios {
    pub elems: Vec<Audio>,
    pub exists: bool,
}

gm_list_chunk!(AUDO, Audios, Audio, direct);

impl GMElement for Audios {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Audio> = reader.read_pointer_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.elems)?;
        Ok(())
    }
}

/// An embedded audio entry in a data file.
#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    /// The raw audio data of the embedded audio entry.
    /// This can be either WAV or OGG.
    pub data: Blob<Vec<u8>>,
}


impl GMElement for Audio {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let size = reader.read_u32()?;
        let data: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
        Ok(Self { data: Blob(data) })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.data.len())?;
        builder.write_bytes(&self.data);
        Ok(())
    }

    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<()> {
        if !is_last {
            reader.align(4)?;
        }
        Ok(())
    }

    fn serialize_post_padding(&self, builder: &mut DataBuilder, is_last: bool) -> Result<()> {
        if !is_last {
            builder.align(4);
        }
        Ok(())
    }
}
