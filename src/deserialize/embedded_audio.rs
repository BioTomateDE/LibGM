use crate::deserialize::chunk_reading::UTChunk;

#[derive(Debug, Clone)]
pub struct UTEmbeddedAudio {
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UTEmbeddedAudioRef {
    pub index: usize,
}
impl UTEmbeddedAudioRef {
    pub fn resolve<'a>(&self, embedded_audios: &'a UTEmbeddedAudios) -> Result<&'a UTEmbeddedAudio, String> {
        match embedded_audios.audios_by_index.get(self.index) {
            Some(audio) => Ok(audio),
            None => Err(format!(
                "Could not resolve embedded audio with index {} in list with length {}.",
                self.index, embedded_audios.audios_by_index.len(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UTEmbeddedAudios {
    pub audios_by_index: Vec<UTEmbeddedAudio>,
}
impl UTEmbeddedAudios {
    pub fn get_audio_by_index(&self, index: usize) -> Option<UTEmbeddedAudioRef> {
        if index >= self.audios_by_index.len() {
            return None;
        }
        Some(UTEmbeddedAudioRef {index})
    }
    pub fn len(&self) -> usize {
        self.audios_by_index.len()
    }
}


#[allow(non_snake_case)]
pub fn parse_chunk_AUDO(chunk: &mut UTChunk) -> Result<UTEmbeddedAudios, String> {
    chunk.file_index = 0;
    let audios_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(audios_count);
    for _ in 0..audios_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut audios_by_index: Vec<UTEmbeddedAudio> = Vec::with_capacity(audios_count);
    for (i, start_position) in start_positions.iter().enumerate() {
        chunk.file_index = *start_position;
        let audio_raw_length: usize = chunk.read_usize()?;
        let audio_raw: &[u8] = match chunk.data.get(chunk.file_index .. chunk.file_index + audio_raw_length) {
            Some(bytes) => bytes,
            None => return Err(format!(
                "Trying to read raw audio out of bounds for embedded audio #{} at position {} in chunk 'AUDO': {} >= {}.",
                i, start_position, chunk.file_index + audio_raw_length, chunk.data.len(),
            )),
        };

        let audio = UTEmbeddedAudio {
            raw_data: audio_raw.to_vec(),
        };
        audios_by_index.push(audio);
    }

    Ok(UTEmbeddedAudios{ audios_by_index })
}

