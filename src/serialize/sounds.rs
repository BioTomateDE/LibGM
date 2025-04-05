use crate::deserialize::all::UTData;
use crate::deserialize::sounds::{UTSound, UTSoundFlags, UTSoundRef};
use crate::serialize::all::{DataBuilder, UTRef};
use crate::serialize::chunk_writing::ChunkBuilder;

#[allow(non_snake_case)]
pub fn build_chunk_SOND(data_builder: &mut DataBuilder, ut_data: &UTData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "SOND", abs_pos: data_builder.len() };
    let len: usize = ut_data.sounds.len();
    builder.write_usize(len)?;

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, UTRef::Sound(UTSoundRef { index: i }))?;
    }

    for i in 0..len {
        data_builder.push_pointing_to(&mut builder, UTRef::Sound(UTSoundRef { index: i }))?;
        let sound: UTSoundRef = ut_data.sounds.get_sound_by_index(i).expect("Sound out of bounds while building.");
        let sound: &UTSound = sound.resolve(&ut_data.sounds)?;
        builder.write_ut_string(&sound.name, &ut_data.strings)?;
        builder.write_u32(build_sound_flags(&sound.flags))?;
        builder.write_ut_string(&sound.audio_type, &ut_data.strings)?;
        builder.write_ut_string(&sound.file, &ut_data.strings)?;
        builder.write_u32(sound.effects)?;
        builder.write_f32(sound.volume)?;
        builder.write_f32(sound.pitch)?;
        // {~~} audio group stuff idk
        builder.write_i32(-1)?;
        let audio_file: Option<UTRef> = match &sound.audio_file { Some(x) => Some(UTRef::Audio(x.clone())), None => None };
        data_builder.push_pointer_position_maybe(&mut builder, audio_file)?;
        if ut_data.general_info.is_version_at_least(2024, 6, 0, 0) {
            builder.write_f32(sound.audio_length.expect("Sound Audio length is None."))?;
        }
    }

    Ok(())
}


fn build_sound_flags(flags: &UTSoundFlags) -> u32 {
    let mut raw: u32 = 0;
    if flags.is_embedded { raw |= 0x1 };
    if flags.is_compressed { raw |= 0x2 };
    if flags.is_decompressed_on_load { raw |= 0x3 };
    if flags.regular { raw |= 0x64 };
    raw
}

