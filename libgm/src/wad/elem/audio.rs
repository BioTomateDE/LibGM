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
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let count = pointers.len();
        let mut elems = Vec::with_capacity(count);

        for (i, pointer) in pointers.into_iter().enumerate() {
            reader.assert_pos(pointer, "Audio")?;
            let size = reader.read_u32()?;
            let data: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
            elems.push(Audio { data: Blob(data) });
            if i != count - 1 {
                reader.align(4)?;
            }
        }

        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        let count = self.elems.len();
        writer.write_usize(count)?;
        let pointer_list_pos = writer.pos();
        for _ in 0..count {
            writer.write_u32(0xDEAD_C0DE);
        }

        for (idx, audio) in self.elems.iter().enumerate() {
            writer.overwrite_pointer_with_cur_pos(pointer_list_pos, idx)?;
            writer.write_usize(audio.data.len())?;
            writer.write_bytes(&audio.data);
            if idx != count - 1 {
                writer.align(4);
            }
        }

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
