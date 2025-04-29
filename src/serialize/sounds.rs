use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::sounds::{GMSound, GMSoundFlags};
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_sond(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "SOND", abs_pos: data_builder.len() };
    let len: usize = gm_data.sounds.sounds_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::sound(i))?;
    }

    for i in 0..len {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::sound(i))?;
        let sound: &GMSound = &gm_data.sounds.sounds_by_index[i];
        builder.write_gm_string(data_builder, &sound.name)?;
        builder.write_u32(build_sound_flags(&sound.flags));
        builder.write_gm_string(data_builder, &sound.audio_type)?;
        builder.write_gm_string(data_builder, &sound.file)?;
        builder.write_u32(sound.effects);
        builder.write_f32(sound.volume);
        builder.write_f32(sound.pitch);
        // {~~} audio group stuff idk
        builder.write_i32(-1);
        match &sound.audio_file {
            Some(file) => data_builder.push_pointer_placeholder(&mut builder, GMPointer::audio(file.index))?,
            None => builder.write_i32(-1),
        }
        if gm_data.general_info.is_version_at_least(2024, 6, 0, 0) {
            builder.write_f32(sound.audio_length.expect("Sound Audio length is None."));
        }
    }

    Ok(())
}


fn build_sound_flags(flags: &GMSoundFlags) -> u32 {
    let mut raw: u32 = 0;
    if flags.is_embedded { raw |= 0x1 };
    if flags.is_compressed { raw |= 0x2 };
    if flags.is_decompressed_on_load { raw |= 0x3 };
    if flags.regular { raw |= 0x64 };
    raw
}

