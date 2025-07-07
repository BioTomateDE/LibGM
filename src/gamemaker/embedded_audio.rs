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
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMEmbeddedAudios {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let audios: Vec<GMEmbeddedAudio> = reader.read_pointer_list()?;
        Ok(Self { audios, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }

        let count: usize = self.audios.len();
        builder.write_usize(count)?;
        let pointer_list_start_pos: usize = builder.len();
        for _ in 0..count {
            builder.write_u32(0xDEADC0DE);
        }

        for (i, audio) in self.audios.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_start_pos + 4*i)?;
            audio.serialize(builder)?;
            if i != count - 1 {
                builder.align(4);
            }
        }

        Ok(())
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
        builder.write_usize(self.audio_data.len())?;
        builder.write_bytes(&self.audio_data);
        Ok(())
    }

    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<(), String> {
        if !is_last {
            reader.align(4)?;
        }
        Ok(())
    }

    fn serialize_post_padding(builder: &mut DataBuilder, is_last: bool) -> Result<(), String> {
        if !is_last {
            builder.align(4);
        }
        Ok(())
    }
}

