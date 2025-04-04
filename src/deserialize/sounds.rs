use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::embedded_audio::{UTEmbeddedAudioRef, UTEmbeddedAudios};
use crate::deserialize::general_info::UTGeneralInfo;
use crate::deserialize::strings::{UTStringRef, UTStrings};

#[derive(Debug, Clone)]
pub struct UTSound {
    pub name: UTStringRef,                          // e.g. "abc_123_a"
    pub flags: UTSoundFlags,                        // e.g. Regular
    pub audio_type: UTStringRef,                    // e.g. ".mp3"
    pub file: UTStringRef,                          // e.g. "abc_123_a.ogg"; doesn't have to actually be a real file in game files (rather embedded audio)
    pub effects: u32,                               // idk; always zero
    pub volume: f32,                                // e.g. 0.69
    pub pitch: f32,                                 // e.g. 4.20
    // pub audio_group: u32,                        // idk; type is wrong
    pub audio_file: Option<UTEmbeddedAudioRef>,     // e.g. UndertaleEmbeddedAudio#17
    pub audio_length: Option<f32>,                  // in seconds probably
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UTSoundRef {
    index: usize,
}
impl UTSoundRef {
    pub fn resolve<'a>(&self, sounds: &'a UTSounds) -> Result<&'a UTSound, String> {
        match sounds.sounds_by_index.get(self.index) {
            Some(sound) => Ok(sound),
            None => Err(format!(
                "Could not resolve sound with index {} in list with length {}.",
                self.index, sounds.sounds_by_index.len()
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UTSoundFlags {
    pub is_embedded: bool,
    pub is_compressed: bool,
    pub is_decompressed_on_load: bool,
    pub regular: bool,
}


#[derive(Debug, Clone)]
pub struct UTSounds {
    pub sounds_by_index: Vec<UTSound>,
}
impl UTSounds {
    pub fn get_sound_by_index(&self, index: usize) -> Option<UTSoundRef> {
        if index >= self.sounds_by_index.len() {
            return None;
        }
        Some(UTSoundRef {index})
    }
    pub fn len(&self) -> usize {
        self.sounds_by_index.len()
    }
}


#[allow(non_snake_case)]
pub fn parse_chunk_SOND(chunk: &mut UTChunk, general_info: &UTGeneralInfo, strings: &UTStrings, embedded_audios: &UTEmbeddedAudios) -> Result<UTSounds, String> {
    chunk.file_index = 0;
    let sounds_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(sounds_count);
    for _ in 0..sounds_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut sounds_by_index: Vec<UTSound> = Vec::with_capacity(sounds_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
        let name: UTStringRef = chunk.read_ut_string(strings)?;
        let flags: UTSoundFlags = parse_sound_flags(chunk.read_u32()?);
        let audio_type: UTStringRef = chunk.read_ut_string(strings)?;
        let file: UTStringRef = chunk.read_ut_string(strings)?;
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
        let audio_file: Option<UTEmbeddedAudioRef> =
            if audio_file == -1 { None }
            else { embedded_audios.get_audio_by_index(audio_file as usize) };

        let mut audio_length: Option<f32> = None;
        if general_info.is_version_at_least(2024, 6, 0, 0) {
            audio_length = Some(chunk.read_f32()?);
        }

        let sound: UTSound = UTSound {
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

    Ok(UTSounds{ sounds_by_index })
}


fn parse_sound_flags(raw: u32) -> UTSoundFlags {
    UTSoundFlags {
        is_embedded: 0 != raw & 0x1,
        is_compressed: 0 != raw & 0x2,
        is_decompressed_on_load: 3 == raw & 0x3,    // maybe??? UndertaleModTool doesn't know either
        regular: 0 != raw & 0x64,
    }
}

