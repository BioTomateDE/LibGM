use crate::gm_deserialize::{GMChunkElement, GMElement, DataReader};
use crate::gm_serialize::DataBuilder;

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
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.audios)
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.resolve_pointer(self)?;
        builder.write_usize(self.audio_data.len())?;
        builder.write_bytes(&self.audio_data);
        Ok(())
    }
}

