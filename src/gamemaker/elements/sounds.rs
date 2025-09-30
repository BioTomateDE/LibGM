use crate::gamemaker::elements::audio_groups::GMAudioGroup;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudio;
use crate::gamemaker::gm_version::GMVersion;
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;

#[derive(Debug, Clone)]
pub struct GMSounds {
    pub sounds: Vec<GMSound>,
    pub exists: bool,
}

impl GMChunkElement for GMSounds {
    fn stub() -> Self {
        Self { sounds: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMSounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let sounds: Vec<GMSound> = reader.read_pointer_list()?;
        Ok(Self { sounds, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.sounds)?;
        Ok(())
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
    pub audio_group: GMRef<GMAudioGroup>,            // Bytecode14+
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
        let mut audio_group: GMRef<GMAudioGroup> = GMRef::new(get_builtin_sound_group_id(&reader.general_info.version));
        if flags.regular && reader.general_info.bytecode_version >= 14 {
            audio_group = reader.read_resource_by_id()?;
        } else {
            let preload: bool = reader.read_bool32()?;
            if !preload {
                return Err(format!("Preload is unexpectedly set to false for sound \"{}\"; please report this error", reader.display_gm_str(name)))
            }
        }
        let audio_file: Option<GMRef<GMEmbeddedAudio>> = reader.read_resource_by_id_opt()?;
        let audio_length: Option<f32> = reader.deserialize_if_gm_version((2024, 6))?;
        Ok(GMSound { name, flags, audio_type, file, effects, volume, pitch, audio_group, audio_file, audio_length })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        self.flags.serialize(builder)?;
        builder.write_gm_string_opt(&self.audio_type)?;
        builder.write_gm_string(&self.file)?;
        builder.write_u32(self.effects);
        builder.write_f32(self.volume);
        builder.write_f32(self.pitch);
        if self.flags.regular && builder.bytecode_version() >= 14 {
            builder.write_resource_id(&self.audio_group);
        } else {
            builder.write_bool32(true);   // Preload   
        }
        builder.write_resource_id_opt(&self.audio_file);
        self.audio_length.serialize_if_gm_ver(builder, "Audio Length", (2024, 6))?;
        Ok(())
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let mut raw: u32 = 0;

        if self.is_embedded { raw |= 0x1 };
        if self.is_compressed { raw |= 0x2 };
        if self.is_decompressed_on_load { raw |= 0x3 };
        if self.regular { raw |= 0x64 };

        builder.write_u32(raw);
        Ok(())
    }
}


fn get_builtin_sound_group_id(gm_version: &GMVersion) -> u32 {
    let is_ver = |i| gm_version.is_version_at_least(i);  // small closure for concision
    // ver >= 1.0.0.1250 || (ver >= 1.0.0.161 && ver < 1.0.0.1000)
    if is_ver((1, 0, 0, 1250)) || is_ver((1, 0, 0, 161)) && !is_ver((1, 0, 0, 1000)) {
        0
    } else {
        1
    }
}

