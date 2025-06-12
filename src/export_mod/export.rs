use std::collections::HashMap;
use crate::deserialize::all::GMData;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};
use crate::deserialize::backgrounds::GMBackground;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::code::GMCode;
use crate::deserialize::embedded_audio::GMEmbeddedAudio;
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::sprites::GMSprite;
use crate::deserialize::texture_page_items::GMTexturePageItem;
use crate::export_mod::fonts::{AddFont, EditFont};
use crate::export_mod::sounds::{AddSound, EditSound};
use crate::export_mod::unordered_list::{EditUnorderedList, GModUnorderedListChanges};


pub fn export_mod(original_data: &GMData, modified_data: &GMData, target_file: &Path) -> Result<(), String> {
    let mut data: Vec<u8> = Vec::new();
    let buff = Cursor::new(&mut data);
    let mut zip_writer = ZipWriter::new(buff);

    let mod_exporter = ModExporter {original_data, modified_data};
    let fonts: EditUnorderedList<AddFont, EditFont> = mod_exporter.export_fonts()?;
    let sounds: EditUnorderedList<AddSound, EditSound> = mod_exporter.export_sounds()?;
    let strings: EditUnorderedList<String, String> = mod_exporter.export_strings()?;
    // repeat ts for every element

    zw_write_unordered_list_changes(&mut zip_writer, "fonts.json", &fonts)?;
    zw_write_unordered_list_changes(&mut zip_writer, "sounds.json", &sounds)?;
    zw_write_unordered_list_changes(&mut zip_writer, "strings.json", &strings)?;
    // repeat ts for every element

    // also export textures and audio separately

    zip_writer.finish()
        .map_err(|e| format!("Could not finish zip archive: {e}"))?;

    let mut file = File::create(target_file)
        .map_err(|e| format!("Could not create target mod file with path {target_file:?}: {e}"))?;
    file.write_all(data.as_slice())
        .map_err(|e| format!("Could not write zip archive data to target mod file with path {target_file:?}: {e}"))?;
    Ok(())
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModRef {
    Data(usize),    // index in gamemaker data list; assumes element also exists in the data it will be loaded in
    Add(usize),     // element is being added by this mod; index of this mod's addition list
}

pub struct ModExporter<'o, 'm> {
    pub original_data: &'o GMData,
    pub modified_data: &'m GMData,
}

impl ModExporter<'_, '_> {
    pub fn convert_audio_ref(&self, gm_audio_ref: GMRef<GMEmbeddedAudio>) -> Result<ModRef, String> {
        convert_reference(gm_audio_ref, &self.original_data.audios.audios_by_index, &self.modified_data.audios.audios_by_index)
    }
    pub fn convert_background_ref(&self, gm_background_ref: GMRef<GMBackground>) -> Result<ModRef, String> {
        convert_reference(gm_background_ref, &self.original_data.backgrounds.backgrounds_by_index, &self.modified_data.backgrounds.backgrounds_by_index)
    }
    // TODO continue
    pub fn convert_game_object_ref(&self, gm_game_object_ref: GMRef<GMGameObject>) -> Result<ModRef, String> {
        convert_reference(gm_game_object_ref, &self.original_data.game_objects.game_objects_by_index, &self.modified_data.game_objects.game_objects_by_index)
    }
    pub fn convert_sprite_ref(&self, gm_sprite_ref: GMRef<GMSprite>) -> Result<ModRef, String> {
        convert_reference(gm_sprite_ref, &self.original_data.sprites.sprites_by_index, &self.modified_data.sprites.sprites_by_index)
    }
    pub fn convert_string_ref(&self, gm_string_ref: GMRef<String>) -> Result<ModRef, String> {
        convert_reference(gm_string_ref, &self.original_data.strings.strings_by_index, &self.modified_data.strings.strings_by_index)
    }
    pub fn convert_texture_ref(&self, gm_texture_ref: GMRef<GMTexturePageItem>) -> Result<ModRef, String> {
        convert_reference(gm_texture_ref, &self.original_data.texture_page_items.textures_by_index, &self.modified_data.texture_page_items.textures_by_index)
    }

    pub fn convert_audio_ref_opt(&self, gm_audio_ref: Option<GMRef<GMEmbeddedAudio>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_audio_ref, &self.original_data.audios.audios_by_index, &self.modified_data.audios.audios_by_index)
    }
    pub fn convert_background_ref_opt(&self, gm_background_ref: Option<GMRef<GMBackground>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_background_ref, &self.original_data.backgrounds.backgrounds_by_index, &self.modified_data.backgrounds.backgrounds_by_index)
    }
    pub fn convert_code_ref_opt(&self, gm_code_ref: Option<GMRef<GMCode>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_code_ref, &self.original_data.codes.codes_by_index, &self.modified_data.codes.codes_by_index)
    }
    pub fn convert_game_object_ref_opt(&self, gm_game_object_ref: Option<GMRef<GMGameObject>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_game_object_ref, &self.original_data.game_objects.game_objects_by_index, &self.modified_data.game_objects.game_objects_by_index)
    }
}

fn convert_reference<GM>(gm_reference: GMRef<GM>, original_list: &[GM], modified_list: &[GM]) -> Result<ModRef, String> {
    // If reference index out of bounds in modified data; throw error.
    // This should never happen in healthy gm data; just being cautious that the mod will be fully functional.
    if gm_reference.index >= modified_list.len() {
        return Err(format!(
            "Could not resolve {} reference with GameMaker index {} in list with length {}; out of bounds",
            std::any::type_name_of_val(&gm_reference), gm_reference.index, modified_list.len(),
        ))
    }

    let original_length = original_list.len();
    if gm_reference.index >= original_length {
        // If reference index exists (isn't out of bounds) in modified data but not in original data,
        // then the element was newly added --> "Add" reference
        Ok(ModRef::Add(gm_reference.index - original_length))
    } else {
        // If reference index exists in original data (and modified data; assumes unordered lists never remove elements),
        // then the element is a reference to the gamemaker data the mod will later be loaded in.
        Ok(ModRef::Data(gm_reference.index))
    }
}

fn convert_reference_optional<GM>(gm_reference_optional: Option<GMRef<GM>>, original_list: &[GM], modified_list: &[GM]) -> Result<Option<ModRef>, String> {
    match gm_reference_optional {
        Some(gm_reference) => Ok(Some(convert_reference(gm_reference, original_list, modified_list)?)),
        None => Ok(None),
    }
}


pub type ModWriter<'a> = ZipWriter<Cursor<&'a mut Vec<u8>>>;
const FILE_OPTIONS: LazyLock<FileOptions<()>> = LazyLock::new(|| 
    SimpleFileOptions::default()
    .compression_method(CompressionMethod::Bzip2)
    .compression_level(Some(9))
);


fn zw_write_file(zip_writer: &mut ModWriter, filename: &str, data: &[u8]) -> Result<(), String> {
    zip_writer.start_file(filename, *FILE_OPTIONS)
        .map_err(|e| format!("Could not create {filename} file in zip archive: {e}"))?;
    zip_writer.write_all(data)
        .map_err(|e| format!("Could not write data to {filename} file in zip archive: {e}"))?;
    Ok(())
}

fn zw_write_unordered_list_changes<ADD: Serialize, EDIT: Serialize>(zip_writer: &mut ModWriter, filename: &str, changes: &EditUnorderedList<ADD, EDIT>) -> Result<(), String> {
    let string: String = serde_json::to_string_pretty(changes)
        .map_err(|e| format!("Could not convert changes to json for zip file {filename}: {e}"))?;
    let data: &[u8] = string.as_bytes();
    zw_write_file(zip_writer, filename, data)
}


pub fn flag_field(original: bool, modified: bool) -> Option<bool> {
    if original == modified {
        None
    } else {
        Some(modified)
    }
}

pub fn edit_field<'a, T: PartialEq + Clone>(original: &T, modified: &T) -> Option<T> {
    if original != modified {
        Some(modified.clone())
    } else {
        None
    }
}
/// TODO remove edit_field_option (impossible to tell whether it should be set to None or ignored; use two layers of Option instead)
pub fn edit_field_option<T: PartialEq + Clone>(original: &Option<T>, modified: &Option<T>) -> Option<T> {
    if original != modified {
        modified.clone()
    } else {
        None
    }
}
pub fn edit_field_convert<GM>(
    original: GMRef<GM>,
    modified: GMRef<GM>,
    converter: impl Fn(GMRef<GM>) -> Result<ModRef, String>,
) -> Result<Option<ModRef>, String> {
    if original.index != modified.index {
        Ok(Some(converter(modified)?))
    } else {
        Ok(None)
    }
}
pub fn edit_field_convert_option<GM: PartialEq>(
    original: Option<GMRef<GM>>,
    modified: Option<GMRef<GM>>,
    converter: impl Fn(Option<GMRef<GM>>) -> Result<Option<ModRef>, String>,
) -> Result<Option<Option<ModRef>>, String> {
    if original == modified {
        Ok(Some(converter(modified)?))
    } else {
        Ok(None)
    }
}

pub fn convert_edits<GM, ADD, EDIT>(
    changes: &GModUnorderedListChanges<GM>,
    map_additions: impl Fn(&[GM]) -> Result<Vec<ADD>, String>,
    map_edit: impl Fn(&GM, &GM) -> Result<EDIT, String>,
) -> Result<EditUnorderedList<ADD, EDIT>, String> {
    let additions: Vec<ADD> = map_additions(&changes.additions)?;
    let edits: HashMap<usize, EDIT> = changes.edits
        .iter()
        .map(|(i, (original, modified))| Ok((*i, map_edit(original, modified)?)))
        .collect::<Result<HashMap<_, _>, String>>()?;
    Ok(EditUnorderedList { additions, edits })
}

pub fn convert_additions<GM, ADD>(gm_elements: &[GM], map_addition: impl Fn(&GM) -> Result<ADD, String>) -> Result<Vec<ADD>, String> {
    gm_elements.iter().map(map_addition).collect()
}

