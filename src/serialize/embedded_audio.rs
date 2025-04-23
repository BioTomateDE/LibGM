use crate::deserialize::all::GMData;
use crate::deserialize::embedded_audio::{GMEmbeddedAudio, GMEmbeddedAudioRef};
use crate::serialize::all::{DataBuilder, GMRef};
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_chunk_audo(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "AUDO", abs_pos: data_builder.len() };
    let len: usize = gm_data.audios.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, GMRef::Audio(GMEmbeddedAudioRef { index: i }))?;
    }

    for i in 0..len {
        data_builder.push_pointing_to(&mut builder, GMRef::Audio(GMEmbeddedAudioRef { index: i }))?;
        let audio: GMEmbeddedAudioRef = gm_data.audios.get_audio_by_index(i).expect("Audio out of bounds while building.");
        let audio: &GMEmbeddedAudio = audio.resolve(&gm_data.audios)?;
        builder.write_usize(audio.raw_data.len());
        builder.write_bytes(&audio.raw_data);
    }

    Ok(())
}

