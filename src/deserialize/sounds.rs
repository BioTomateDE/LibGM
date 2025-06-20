use crate::deserialize::chunk_reading::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::deserialize::embedded_audio::GMEmbeddedAudio;


#[derive(Debug, Clone)]
pub struct GMSounds {
    pub sounds: Vec<GMSound>,
    pub exists: bool,
}
impl GMChunkElement for GMSounds {
    fn empty() -> Self {
        Self { sounds: vec![], exists: false }
    }
}
impl GMElement for GMSounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let sounds: Vec<GMSound> = reader.read_pointer_list()?;
        Ok(Self { sounds, exists: true })
    }
}


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
impl GMElement for GMSound {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let flags = GMSoundFlags::deserialize(reader)?;
        let audio_type: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let file: GMRef<String> = reader.read_gm_string()?;
        let effects: u32 = reader.read_u32()?;
        let volume: f32 = reader.read_f32()?;
        let pitch: f32 = reader.read_f32()?;
        if flags.regular && reader.general_info.bytecode_version >= 14 {
            // audio group stuff {~~} TODO
        } else {
            // group id stuff {~~}
        }
        let _ = reader.read_u32()?;      // because we skipped group stuff
        let audio_file: Option<GMRef<GMEmbeddedAudio>> = reader.read_resource_by_id_option()?;

        let mut audio_length: Option<f32> = None;
        if reader.general_info.is_version_at_least((2024, 6, 0, 0)) {
            audio_length = Some(reader.read_f32()?);
        }

        Ok(GMSound {
            name,
            flags,
            audio_type,
            file,
            effects,
            volume,
            pitch,
            audio_file,
            audio_length,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSoundFlags {
    pub is_embedded: bool,
    pub is_compressed: bool,
    pub is_decompressed_on_load: bool,
    pub regular: bool,
}
impl GMElement for GMSoundFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let raw: u32 = reader.read_u32()?;
        Ok(GMSoundFlags {
            is_embedded: 0 != raw & 0x1,
            is_compressed: 0 != raw & 0x2,
            is_decompressed_on_load: 3 == raw & 0x3,    // maybe??? UndertaleModTool doesn't know either
            regular: 0 != raw & 0x64,
        })
    }
}

