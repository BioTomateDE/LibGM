use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::deserialize::sounds::{GMSound, GMSoundFlags};
use crate::export_mod::export::{edit_field, edit_field_option, flag_field, GModData, ModUnorderedRef};
use crate::export_mod::unordered_list::{ AModUnorderedListChanges, GModUnorderedListChanges};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSound {
    pub name: Option<ModUnorderedRef>,      // String
    pub flags: Option<ModSoundFlags>,
    pub audio_type: Option<ModUnorderedRef>,  // String
    pub file: Option<ModUnorderedRef>,      // String
    pub effects: Option<u32>,
    pub volume: Option<f32>,
    pub pitch: Option<f32>,
    pub audio_file: Option<ModUnorderedRef>,    // Embedded Audio
    pub audio_length: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSoundFlags {
    pub is_embedded: Option<bool>,
    pub is_compressed: Option<bool>,
    pub is_decompressed_on_load: Option<bool>,
    pub regular: Option<bool>,
}

impl GModData<'_, '_> {
    pub fn convert_sound_additions(&self, gm_sounds: &Vec<GMSound>) -> Result<Vec<ModSound>, String> {
        let mut mod_sounds: Vec<ModSound> = Vec::with_capacity(gm_sounds.len());

        for sound in gm_sounds {
            mod_sounds.push(ModSound {
                name: Some(self.resolve_string_ref(&sound.name)?),
                flags: Some(self.convert_sound_flag_additions(&sound.flags)?),
                audio_type: Some(self.resolve_string_ref(&sound.audio_type)?),
                file: Some(self.resolve_string_ref(&sound.file)?),
                effects: Some(sound.effects),
                volume: Some(sound.volume),
                pitch: Some(sound.pitch),
                audio_file: if let Some(ref audio_file) = sound.audio_file { Some(self.resolve_audio_ref(&audio_file)?) } else { None },
                audio_length: sound.audio_length,
            });
        }

        Ok(mod_sounds)
    }

    pub fn convert_sounds(&self, changes: &GModUnorderedListChanges<GMSound>) -> Result<AModUnorderedListChanges<ModSound>, String> {
        let additions: Vec<ModSound> = self.convert_sound_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModSound> = HashMap::new();

        // let fn_audio_not_set: fn(&GMSound) -> String = |sound: &GMSound| format!(
        //     "Audio data not set for Sound \"{}\"! AcornGM ",
        //     sound.name.display(&self.modified_data.strings),
        // );

        for (index, (original, modified)) in &changes.edits {
            let resolved_original_audio_file = original
                .audio_file
                .as_ref()
                .map(|def| self.resolve_audio_ref(def))
                .transpose()?; // This gives you an Option<T>, not Result<Option<T>>

            let resolved_modified_audio_file = modified
                .audio_file
                .as_ref()
                .map(|def| self.resolve_audio_ref(def))
                .transpose()?;

            edits.insert(*index, ModSound {
                name: edit_field(&self.resolve_string_ref(&original.name)?, &self.resolve_string_ref(&modified.name)?),
                flags: Some(self.convert_sound_flags(&original.flags, &modified.flags)?),
                audio_type: edit_field(&self.resolve_string_ref(&original.audio_type)?, &self.resolve_string_ref(&modified.audio_type)?),
                file: edit_field(&self.resolve_string_ref(&original.file)?, &self.resolve_string_ref(&modified.file)?),   // unnecessary ig
                effects: edit_field(&original.effects, &modified.effects),
                volume: edit_field(&original.volume, &modified.volume),
                pitch: edit_field(&original.pitch, &modified.pitch),
                audio_file: edit_field_option(&resolved_original_audio_file, &resolved_modified_audio_file).clone(),
                audio_length: *edit_field_option(&original.audio_length, &modified.audio_length),
            });
        }

        Ok(AModUnorderedListChanges { additions, edits })
    }

    pub fn convert_sound_flag_additions(&self, flags: &GMSoundFlags) -> Result<ModSoundFlags, String> {
        let flags = ModSoundFlags {
            is_embedded: Some(flags.is_embedded),
            is_compressed: Some(flags.is_compressed),
            is_decompressed_on_load: Some(flags.is_decompressed_on_load),
            regular: Some(flags.regular),
        };
        Ok(flags)
    }

    pub fn convert_sound_flags(&self, original: &GMSoundFlags, modified: &GMSoundFlags) -> Result<ModSoundFlags, String> {
        let flags = ModSoundFlags {
            is_embedded: flag_field(original.is_embedded, modified.is_embedded),
            is_compressed: flag_field(original.is_compressed, modified.is_compressed),
            is_decompressed_on_load: flag_field(original.is_decompressed_on_load, modified.is_decompressed_on_load),
            regular: flag_field(original.regular, modified.regular),
        };
        Ok(flags)
    }
}

