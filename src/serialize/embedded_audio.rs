use crate::deserialize::all::GMData;
use crate::deserialize::embedded_audio::GMEmbeddedAudio;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_audo(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("AUDO")?;
    let len: usize = gm_data.audios.audios_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(GMPointer::Audio(i))?;
    }

    for i in 0..len {
        builder.resolve_pointer(GMPointer::Audio(i))?;
        let audio: &GMEmbeddedAudio = &gm_data.audios.audios_by_index[i];
        builder.write_usize(audio.raw_data.len());
        builder.write_bytes(&audio.raw_data);
    }

    builder.finish_chunk()?;
    Ok(())
}

