use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::sounds::GMSoundFlags;
use crate::modding::export::{edit_field, edit_field_convert, edit_field_convert_option, flag_field, ModExporter, ModRef};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSound {
    pub name: ModRef,      // String
    pub flags: AddSoundFlags,
    pub audio_type: Option<ModRef>,  // String
    pub file: ModRef,      // String
    pub effects: u32,
    pub volume: f32,
    pub pitch: f32,
    pub audio_file: Option<ModRef>,    // Embedded Audio
    pub audio_length: Option<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSoundFlags {
    pub is_embedded: bool,
    pub is_compressed: bool,
    pub is_decompressed_on_load: bool,
    pub regular: bool,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSound {
    pub name: Option<ModRef>,      // String
    pub flags: EditSoundFlags,
    pub audio_type: Option<ModRef>,  // String
    pub filename: Option<ModRef>,      // String
    pub effects: Option<u32>,
    pub volume: Option<f32>,
    pub pitch: Option<f32>,
    pub audio_group: Option<ModRef>,    // Audio Group
    pub audio_data: Option<ModRef>,  // Embedded Audio
    pub audio_length: Option<f32>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSoundFlags {
    // TODO: remove (some of) these??
    pub is_embedded: Option<bool>,
    pub is_compressed: Option<bool>,
    pub is_decompressed_on_load: Option<bool>,
    pub regular: Option<bool>,
}


impl ModExporter<'_, '_> {
    pub fn export_sounds(&self) -> Result<Vec<DataChange<AddSound, EditSound>>, String> {
        export_changes_ordered_list(
            &self.original_data.sounds.sounds,
            &self.modified_data.sounds.sounds,
            |i| Ok(AddSound {
                name: self.convert_string_ref(&i.name)?,
                flags: add_sound_flags(&i.flags),
                audio_type: self.convert_string_ref_opt(&i.audio_type)?,
                file: self.convert_string_ref(&i.file)?,
                effects: i.effects,
                volume: i.volume,
                pitch: i.pitch,
                audio_file: self.convert_audio_ref_opt(&i.audio_file)?,
                audio_length: i.audio_length,
            }),
            |o, m| Ok(EditSound {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                flags: edit_sound_flags(&o.flags, &m.flags),
                audio_type: edit_field_convert_option(&o.audio_type, &m.audio_type, |r| self.convert_string_ref(r))?.flatten(),
                filename: edit_field_convert(&o.file, &m.file, |r| self.convert_string_ref(r))?,
                effects: edit_field(&o.effects, &m.effects),
                volume: edit_field(&o.volume, &m.volume),
                pitch: edit_field(&o.pitch, &m.pitch),
                audio_group: edit_field_convert(&o.audio_group, &m.audio_group, |r| self.convert_audio_group_ref(r))?,
                audio_data: edit_field_convert_option(&o.audio_file, &m.audio_file, |r| self.convert_audio_ref(r))?.flatten(),
                audio_length: edit_field(&o.audio_length, &m.audio_length).flatten(),
            }),
        )
    }
}

fn add_sound_flags(i: &GMSoundFlags) -> AddSoundFlags {
    AddSoundFlags {
        is_embedded: i.is_embedded,
        is_compressed: i.is_compressed,
        is_decompressed_on_load: i.is_decompressed_on_load,
        regular: i.regular,
    }
}

fn edit_sound_flags(o: &GMSoundFlags, m: &GMSoundFlags) -> EditSoundFlags {
    EditSoundFlags {
        is_embedded: flag_field(o.is_embedded, m.is_embedded),
        is_compressed: flag_field(o.is_compressed, m.is_compressed),
        is_decompressed_on_load: flag_field(o.is_decompressed_on_load, m.is_decompressed_on_load),
        regular: flag_field(o.regular, m.regular),
    }
}


