use crate::deserialize::all::UTData;
use crate::deserialize::embedded_audio::{UTEmbeddedAudio, UTEmbeddedAudios, UTEmbeddedAudioRef};
use crate::serialize::all::{DataBuilder, UTRef};
use crate::serialize::chunk_writing::ChunkBuilder;

#[allow(non_snake_case)]
pub fn build_chunk_AUDO(data_builder: &mut DataBuilder, ut_data: &UTData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "AUDO", abs_pos: data_builder.len() };
    let len: usize = ut_data.audios.len();
    builder.write_usize(len)?;

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, UTRef::Audio(UTEmbeddedAudioRef { index: i }))?;
    }

    for i in 0..len {
        data_builder.push_pointing_to(&mut builder, UTRef::Audio(UTEmbeddedAudioRef { index: i }))?;
        let audio: UTEmbeddedAudioRef = ut_data.audios.get_audio_by_index(i).expect("Sound out of bounds while building.");
        let audio: &UTEmbeddedAudio = audio.resolve(&ut_data.audios)?;
        builder.write_usize(audio.raw_data.len())?;
        builder.write_bytes(&audio.raw_data)?;
    }

    Ok(())
}

