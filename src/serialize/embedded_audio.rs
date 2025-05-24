use crate::deserialize::all::GMData;
use crate::deserialize::embedded_audio::GMEmbeddedAudio;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_audo(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "AUDO");
    let len: usize = gm_data.audios.audios_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.write_pointer_placeholder(&mut builder, GMPointer::Audio(i))?;
    }

    for i in 0..len {
        data_builder.resolve_pointer(&mut builder, GMPointer::Audio(i))?;
        let audio: &GMEmbeddedAudio = &gm_data.audios.audios_by_index[i];
        builder.write_usize(audio.raw_data.len());
        builder.write_bytes(&audio.raw_data);
    }

    builder.finish(data_builder)?;
    Ok(())
}

