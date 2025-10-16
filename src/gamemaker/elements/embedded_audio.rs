use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct GMEmbeddedAudios {
    pub audios: Vec<GMEmbeddedAudio>,
    pub exists: bool,
}

impl GMChunkElement for GMEmbeddedAudios {
    fn stub() -> Self {
        Self { audios: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMEmbeddedAudios {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let audios: Vec<GMEmbeddedAudio> = reader.read_pointer_list()?;
        Ok(Self { audios, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if !self.exists {
            return Ok(());
        }
        builder.write_pointer_list(&self.audios)?;
        Ok(())
    }
}

/// An embedded audio entry in a data file.
#[derive(Debug, Clone, PartialEq)]
pub struct GMEmbeddedAudio {
    /// The raw WAV audio data of the embedded audio entry.
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
