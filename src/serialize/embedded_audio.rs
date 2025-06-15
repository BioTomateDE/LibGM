use crate::deserialize::all::GMData;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_audo(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("AUDO")?;
    let len: usize = gm_data.audios.audios_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(GMPointer::Audio(i))?;
    }

    for (i, audio) in gm_data.audios.audios_by_index.iter().enumerate() {
        builder.resolve_pointer(GMPointer::Audio(i))?;
        builder.write_usize(audio.raw_data.len());
        builder.write_bytes(&audio.raw_data);

        // padding
        if i + 1 != len {
            builder.align(4);
        }
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

