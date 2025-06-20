use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::embedded_audio::GMEmbeddedAudio;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMSound {
    pub name: GMRef<String>,                         // e.g. "abc_123_a"
    pub flags: GMSoundFlags,                         // e.g. Regular
    pub audio_type: Option<GMRef<String>>,           // e.g. ".mp3"
    pub file: GMRef<String>,                         // e.g. "abc_123_a.ogg"; doesn't have to actually be a real file in game files (rather embedded audio)
    pub effects: u32,                                // idk; always zero
    pub volume: f32,                                 // e.g. 0.69
    pub pitch: f32,                                  // e.g. 4.20
    // pub audio_group: u32,                         // idk; type is wrong
    pub audio_file: Option<GMRef<GMEmbeddedAudio>>,  // e.g. UndertaleEmbeddedAudio#17
    pub audio_length: Option<f32>,                   // in seconds probably
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSoundFlags {
    pub is_embedded: bool,
    pub is_compressed: bool,
    pub is_decompressed_on_load: bool,
    pub regular: bool,
}


#[derive(Debug, Clone)]
pub struct GMSounds {
    pub sounds_by_index: Vec<GMSound>,
}


pub fn parse_chunk_sond(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMSounds, String> {
    chunk.cur_pos = 0;
    let sounds_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(sounds_count);
    for _ in 0..sounds_count {
        start_positions.push(chunk.read_relative_pointer()?);
    }

    let mut sounds_by_index: Vec<GMSound> = Vec::with_capacity(sounds_count);
    for start_position in start_positions {
        chunk.cur_pos = start_position;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let flags: GMSoundFlags = parse_sound_flags(chunk.read_u32()?);
        let audio_type: Option<GMRef<String>> = chunk.read_gm_string_optional(strings)?;
        let file: GMRef<String> = chunk.read_gm_string(strings)?;
        let effects: u32 = chunk.read_u32()?;
        let volume: f32 = chunk.read_f32()?;
        let pitch: f32 = chunk.read_f32()?;
        if flags.regular && general_info.bytecode_version >= 14 {
            // audio group stuff {~~}
        } else {
            // group id stuff {~~}
        }
        let _ = chunk.read_u32()?;      // because we skipped group stuff
        let audio_file: i32 = chunk.read_i32()?;
        let audio_file: Option<GMRef<GMEmbeddedAudio>> =
            if audio_file == -1 { None }
            else { Some(GMRef::new(audio_file as usize)) };

        let mut audio_length: Option<f32> = None;
        if general_info.is_version_at_least(2024, 6, 0, 0) {
            audio_length = Some(chunk.read_f32()?);
        }

        let sound: GMSound = GMSound {
            name,
            flags,
            audio_type,
            file,
            effects,
            volume,
            pitch,
            audio_file,
            audio_length,
        };
        sounds_by_index.push(sound);
    }

    Ok(GMSounds{ sounds_by_index })
}


fn parse_sound_flags(raw: u32) -> GMSoundFlags {
    GMSoundFlags {
        is_embedded: 0 != raw & 0x1,
        is_compressed: 0 != raw & 0x2,
        is_decompressed_on_load: 3 == raw & 0x3,    // maybe??? UndertaleModTool doesn't know either
        regular: 0 != raw & 0x64,
    }
}

