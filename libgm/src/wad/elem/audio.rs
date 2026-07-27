// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Audios {
    pub elems: Vec<Audio>,
}

gm_list_chunk!(AUDO, Audios, Audio, direct);

impl GMElement for Audios {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Audio> = reader.read_pointer_list()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_pointer_list(&self.elems)?;
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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_usize(self.data.len())?;
        writer.write_bytes(&self.data);
        Ok(())
    }

    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<()> {
        if !is_last {
            reader.align(4)?;
        }
        Ok(())
    }

    fn serialize_post_padding(&self, writer: &mut DataWriter, is_last: bool) -> Result<()> {
        if !is_last {
            writer.align(4);
        }
        Ok(())
    }
}
