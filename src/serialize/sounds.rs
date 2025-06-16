use crate::deserialize::all::GMData;
use crate::deserialize::sounds::{GMSound, GMSoundFlags};
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_sond(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("SOND")?;
    let len: usize = gm_data.sounds.sounds_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(GMPointer::Sound(i))?;
    }

    for i in 0..len {
        builder.resolve_pointer(GMPointer::Sound(i))?;
        let sound: &GMSound = &gm_data.sounds.sounds_by_index[i];
        builder.write_gm_string(&sound.name)?;
        builder.write_u32(build_sound_flags(&sound.flags));
        builder.write_gm_string_optional(&sound.audio_type)?;
        builder.write_gm_string(&sound.file)?;
        builder.write_u32(sound.effects);
        builder.write_f32(sound.volume);
        builder.write_f32(sound.pitch);
        builder.write_i32(-1);    // {~~} audio group stuff idk   TODO check if -1 is a valid stub
        match &sound.audio_file {
            Some(file) => builder.write_usize(file.index),
            None => builder.write_i32(-1),
        }
        if gm_data.general_info.is_version_at_least(2024, 6, 0, 0) {
            builder.write_f32(sound.audio_length.expect("Sound Audio length is None"));
        }
    }

    builder.finish_chunk(&gm_data.general_info)?;
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

