use crate::deserialize::chunk_reading::GMChunk;

#[derive(Debug, Clone)]
pub struct GMEmbeddedAudio {
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GMEmbeddedAudios {
    pub audios_by_index: Vec<GMEmbeddedAudio>,
}

pub fn parse_chunk_audo(chunk: &mut GMChunk) -> Result<GMEmbeddedAudios, String> {
    chunk.file_index = 0;
    let audios_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(audios_count);
    for _ in 0..audios_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut audios_by_index: Vec<GMEmbeddedAudio> = Vec::with_capacity(audios_count);
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

        let audio = GMEmbeddedAudio {
            raw_data: audio_raw.to_vec(),
        };
        audios_by_index.push(audio);
    }

    Ok(GMEmbeddedAudios{ audios_by_index })
}

