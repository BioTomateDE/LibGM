use crate::deserialize::chunk_reading::GMChunk;

#[derive(Debug, Clone, PartialEq)]
pub struct GMEmbeddedAudio {
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GMEmbeddedAudios {
    pub audios_by_index: Vec<GMEmbeddedAudio>,
}

pub fn parse_chunk_audo(chunk: &mut GMChunk) -> Result<GMEmbeddedAudios, String> {
    chunk.cur_pos = 0;
    let audios_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(audios_count);
    for _ in 0..audios_count {
        start_positions.push(chunk.read_relative_pointer()?);
    }

    let mut audios_by_index: Vec<GMEmbeddedAudio> = Vec::with_capacity(audios_count);
    for (i, start_position) in start_positions.iter().enumerate() {
        chunk.cur_pos = *start_position;
        let audio_raw_length: usize = chunk.read_usize()?;
        let audio_raw: &[u8] = chunk.read_bytes_dyn(audio_raw_length)
            .map_err(|e| format!("Trying to read raw audio #{i} with length {audio_raw_length} {e}"))?;

        let audio = GMEmbeddedAudio {
            raw_data: audio_raw.to_vec(),
        };
        audios_by_index.push(audio);
    }

    Ok(GMEmbeddedAudios{ audios_by_index })
}

