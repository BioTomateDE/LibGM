use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::embedded_audio::{GMEmbeddedAudioRef, GMEmbeddedAudios};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::{GMStringRef, GMStrings};

#[derive(Debug, Clone)]
pub struct GMSound {
    pub name: GMStringRef,                          // e.g. "abc_123_a"
    pub flags: GMSoundFlags,                        // e.g. Regular
    pub audio_type: GMStringRef,                    // e.g. ".mp3"
    pub file: GMStringRef,                          // e.g. "abc_123_a.ogg"; doesn't have to actually be a real file in game files (rather embedded audio)
    pub effects: u32,                               // idk; always zero
    pub volume: f32,                                // e.g. 0.69
    pub pitch: f32,                                 // e.g. 4.20
    // pub audio_group: u32,                        // idk; type is wrong
    pub audio_file: Option<GMEmbeddedAudioRef>,     // e.g. UndertaleEmbeddedAudio#17
    pub audio_length: Option<f32>,                  // in seconds probably
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GMSoundRef {
    pub index: usize,
}
impl GMSoundRef {
    pub fn resolve<'a>(&self, sounds: &'a GMSounds) -> Result<&'a GMSound, String> {
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
impl GMSounds {
    pub fn get_sound_by_index(&self, index: usize) -> Option<GMSoundRef> {
        if index >= self.sounds_by_index.len() {
            return None;
        }
        Some(GMSoundRef {index})
    }
    pub fn len(&self) -> usize {
        self.sounds_by_index.len()
    }
}


#[allow(non_snake_case)]
pub fn parse_chunk_SOND(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings, embedded_audios: &GMEmbeddedAudios) -> Result<GMSounds, String> {
    chunk.file_index = 0;
    let sounds_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(sounds_count);
    for _ in 0..sounds_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut sounds_by_index: Vec<GMSound> = Vec::with_capacity(sounds_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
        let name: GMStringRef = chunk.read_gm_string(strings)?;
        let flags: GMSoundFlags = parse_sound_flags(chunk.read_u32()?);
        let audio_type: GMStringRef = chunk.read_gm_string(strings)?;
        let file: GMStringRef = chunk.read_gm_string(strings)?;
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
        let audio_file: Option<GMEmbeddedAudioRef> =
            if audio_file == -1 { None }
            else { embedded_audios.get_audio_by_index(audio_file as usize) };

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

