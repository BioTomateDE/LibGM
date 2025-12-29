use macros::list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[list_chunk("AUDO")]
pub struct GMEmbeddedAudios {
    pub audios: Vec<GMEmbeddedAudio>,
    pub exists: bool,
}

impl GMElement for GMEmbeddedAudios {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let audios: Vec<GMEmbeddedAudio> = reader.read_pointer_list()?;
        Ok(Self { audios, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.audios)?;
        Ok(())
    }
}

/// An embedded audio entry in a data file.
#[derive(Debug, Clone, PartialEq)]
pub struct GMEmbeddedAudio {
    /// The raw audio data of the embedded audio entry.
    /// This can be either WAV or OGG.
    pub audio_data: Vec<u8>,
}

impl GMElement for GMEmbeddedAudio {
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
