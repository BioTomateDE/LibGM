use crate::deserialize::chunk_reading::UTChunk;
// use hound;

#[derive(Debug, Clone)]
pub struct UTEmbeddedAudio {
    raw_data: Vec<u8>,
}


pub fn parse_chunk_AUDO(chunk: &mut UTChunk) -> Result<Vec<UTEmbeddedAudio>, String> {
    chunk.file_index = 0;
    let audios_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(audios_count);
    for _ in 0..audios_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut audios: Vec<UTEmbeddedAudio> = Vec::with_capacity(audios_count);
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
        // chunk.file_index += audio_raw_length;    // unnecessary because set at start of for loop anyways

        // let mut wav_reader: hound::WavReader<&[u8]> = match hound::WavReader::new(audio_raw) {
        //     Ok(reader) => reader,
        //     Err(error) => return Err(format!(
        //         "Could not read WAV embedded audio #{} at position {} in chunk 'AUDO': \"{}\".",
        //         i, start_position, error
        //     )),
        // };

        // f32 might not be the best sample type? (for undertale)
        // let samples: hound::WavSamples<Vec<u8>, f32> = wav_reader.samples::<f32>();

        let audio = UTEmbeddedAudio {
            raw_data: audio_raw.to_vec(),
        };
        audios.push(audio);
    }

    Ok(audios)
}

