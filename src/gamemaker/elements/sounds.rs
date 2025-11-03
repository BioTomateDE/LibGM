use std::ops::{Deref, DerefMut};
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::audio_groups::GMAudioGroup;
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudio;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::GMVersion;
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct GMSounds {
    pub sounds: Vec<GMSound>,
    pub exists: bool,
}

impl Deref for GMSounds {
    type Target = Vec<GMSound>;
    fn deref(&self) -> &Self::Target {
        &self.sounds
    }
}

impl DerefMut for GMSounds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sounds
    }
}

impl GMChunkElement for GMSounds {
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMSounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sounds: Vec<GMSound> = reader.read_pointer_list()?;
        Ok(Self { sounds, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.sounds)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSound {
    pub name: GMRef<String>,
    pub flags: GMSoundFlags,
    pub audio_type: Option<GMRef<String>>,
    pub file: GMRef<String>,
    pub effects: u32,
    pub volume: f32,
    pub pitch: f32,
    pub audio_group: GMRef<GMAudioGroup>,
    pub audio_file: Option<GMRef<GMEmbeddedAudio>>,
    pub audio_length: Option<f32>,
}

impl GMElement for GMSound {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let flags = GMSoundFlags::deserialize(reader)?;
        let audio_type: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let file: GMRef<String> = reader.read_gm_string()?;
        let effects = reader.read_u32()?;
        let volume = reader.read_f32()?;
        let pitch = reader.read_f32()?;
        let mut audio_group: GMRef<GMAudioGroup> = GMRef::new(get_builtin_sound_group_id(&reader.general_info.version));
        if flags.regular && reader.general_info.bytecode_version >= 14 {
            audio_group = reader.read_resource_by_id()?;
        } else {
            let preload = reader.read_bool32()?;
            if !preload {
                bail!(
                    "Preload is unexpectedly set to false for sound {:?}; please report this error",
                    reader.display_gm_str(name)
                );
            }
        }
        let audio_file: Option<GMRef<GMEmbeddedAudio>> = reader.read_resource_by_id_opt()?;
        let audio_length: Option<f32> = reader.deserialize_if_gm_version((2024, 6))?;
        Ok(GMSound {
            name,
            flags,
            audio_type,
            file,
            effects,
            volume,
            pitch,
            audio_group,
            audio_file,
            audio_length,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
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
            builder.write_bool32(true); // Preload   
        }
        builder.write_resource_id_opt(&self.audio_file);
        self.audio_length
            .serialize_if_gm_ver(builder, "Audio Length", (2024, 6))?;
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw = reader.read_u32()?;
        Ok(GMSoundFlags {
            is_embedded: 0 != raw & 0x1,
            is_compressed: 0 != raw & 0x2,
            is_decompressed_on_load: 3 == raw & 0x3, // Maybe??? UndertaleModTool doesn't know either
            regular: 0 != raw & 0x64,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut raw: u32 = 0;
        raw |= self.is_embedded as u32 * 0x1;
        raw |= self.is_compressed as u32 * 0x2;
        raw |= self.is_decompressed_on_load as u32 * 0x3;
        raw |= self.regular as u32 * 0x64;
        builder.write_u32(raw);
        Ok(())
    }
}

fn get_builtin_sound_group_id(gm_version: &GMVersion) -> u32 {
    let is_ver = |req| gm_version.is_version_at_least(req); // Small closure for concision
    // ver >= 1.0.0.1250 || (ver >= 1.0.0.161 && ver < 1.0.0.1000)
    if is_ver((1, 0, 0, 1250)) || is_ver((1, 0, 0, 161)) && !is_ver((1, 0, 0, 1000)) {
        0
    } else {
        1
    }
}
