use crate::gamemaker::chunk_reading::{GMChunk, GMChunkElement, GMElement, DataReader};

#[derive(Debug, Clone)]
pub struct GMEmbeddedAudios {
    pub audios: Vec<GMEmbeddedAudio>,
    pub exists: bool,
}
impl GMChunkElement for GMEmbeddedAudios {
    fn empty() -> Self {
        Self { audios: vec![], exists: false }
    }
}
impl GMElement for GMEmbeddedAudios {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let audios: Vec<GMEmbeddedAudio> = reader.read_pointer_list()?;
        Ok(Self { audios, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMEmbeddedAudio {
    pub audio_data: Vec<u8>,
}
impl GMElement for GMEmbeddedAudio {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let audio_data_length: usize = reader.read_usize()?;
        let audio_data: Vec<u8> = reader.read_bytes_dyn(audio_data_length)?.to_vec();
        Ok(Self { audio_data })
    }
}

