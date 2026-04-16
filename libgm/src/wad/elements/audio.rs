use std::fmt;

use macros::list_chunk;

use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;

#[list_chunk("AUDO")]
pub struct GMAudios {
    pub audios: Vec<GMAudio>,
    pub exists: bool,
}

impl GMElement for GMAudios {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let audios: Vec<GMAudio> = reader.read_pointer_list()?;
        Ok(Self { audios, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.audios)?;
        Ok(())
    }
}

/// An embedded audio entry in a data file.
#[derive(Clone, PartialEq)]
pub struct GMAudio {
    /// The raw audio data of the embedded audio entry.
    /// This can be either WAV or OGG.
    pub audio_data: Vec<u8>,
}

impl fmt::Debug for GMAudio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GMAudio").finish_non_exhaustive()
    }
}

impl GMElement for GMAudio {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let audio_data_length = reader.read_u32()?;
        let audio_data: Vec<u8> = reader.read_bytes_dyn(audio_data_length)?.to_vec();
        Ok(Self { audio_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.audio_data.len())?;
        builder.write_bytes(&self.audio_data);
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
