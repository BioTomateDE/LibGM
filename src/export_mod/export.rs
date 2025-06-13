use crate::deserialize::all::GMData;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use crate::deserialize::backgrounds::GMBackground;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::code::GMCode;
use crate::deserialize::embedded_audio::GMEmbeddedAudio;
use crate::deserialize::functions::GMFunction;
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::sprites::GMSprite;
use crate::deserialize::texture_page_items::GMTexturePageItem;
use crate::deserialize::variables::GMVariable;
use crate::export_mod::code::{AddCode, EditCode};
use crate::export_mod::fonts::{AddFont, EditFont};
use crate::export_mod::functions::{AddFunction, EditFunction};
use crate::export_mod::rooms::{AddRoom, EditRoom};
use crate::export_mod::sounds::{AddSound, EditSound};
use crate::export_mod::textures::{AddTexturePageItem, EditTexturePageItem};
use crate::export_mod::unordered_list::EditUnorderedList;


pub fn export_mod(original_data: &GMData, modified_data: &GMData, target_file_path: &Path) -> Result<(), String> {
    // initialize file and tarball
    let file = File::create(target_file_path)
        .map_err(|e| format!("Could not create archive file with path \"{}\": {e}", target_file_path.display()))?;
    let zstd_encoder = zstd::Encoder::new(file, 19)
        .map_err(|e| format!("Could not create Zstd encoder: {e}"))?;
    let mut tar = tar::Builder::new(zstd_encoder);

    let mod_exporter = ModExporter {original_data, modified_data};
    let codes: EditUnorderedList<AddCode, EditCode> = mod_exporter.export_codes()?;
    let fonts: EditUnorderedList<AddFont, EditFont> = mod_exporter.export_fonts()?;
    let functions: EditUnorderedList<AddFunction, EditFunction> = mod_exporter.export_functions()?;
    let rooms: EditUnorderedList<AddRoom, EditRoom> = mod_exporter.export_rooms()?;
    let sounds: EditUnorderedList<AddSound, EditSound> = mod_exporter.export_sounds()?;
    let strings: EditUnorderedList<String, String> = mod_exporter.export_strings()?;
    let (texture_page_items, images): (EditUnorderedList<AddTexturePageItem, EditTexturePageItem>, Vec<DynamicImage>) = mod_exporter.export_textures()?;
    // repeat ts for every element {~~}

    tar_write_json_file(&mut tar, "codes", &codes)?;
    tar_write_json_file(&mut tar, "fonts", &fonts)?;
    tar_write_json_file(&mut tar, "functions", &functions)?;
    tar_write_json_file(&mut tar, "sounds", &sounds)?;
    tar_write_json_file(&mut tar, "rooms", &rooms)?;
    tar_write_json_file(&mut tar, "strings", &strings)?;
    tar_write_json_file(&mut tar, "textures", &texture_page_items)?;
    // repeat ts for every element {~~}

    // export textures into textures/{i}.png
    for (i, image) in images.iter().enumerate() {
        let file_path: String = format!("textures/{i}.png");
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| format!("Could not encode PNG image: {e}"))?;
        tar_write_raw_file(&mut tar, &file_path, &buffer.into_inner())?;
    }

    // export audio into audios/{i}.wav
    // ^ TODO

    // finalize
    tar.into_inner()
        .map_err(|e| format!("Could not get inner value of tarball: {e}"))?
        .finish()
        .map_err(|e| format!("Could not finish writing tarball: {e}"))?;
    Ok(())
}

fn tar_write_json_file<ADD: Serialize, EDIT: Serialize>(
    tar: &mut tar::Builder<zstd::Encoder<File>>,
    name: &str,
    changes: &EditUnorderedList<ADD, EDIT>,
) -> Result<(), String> {
    let filename: String = format!("{name}.json");

    let data: Vec<u8> = serde_json::to_vec_pretty(changes)
        .map_err(|e| format!("Could not serialize {name} changes to json: {e}"))?;

    let mut header = tar::Header::new_gnu();
    header.set_path(&filename)
        .map_err(|e| format!("Could not set tar file path for json file \"{filename}\": {e}"))?;
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(get_current_unix_time());
    header.set_cksum();   // has to be called last
    tar.append(&header, data.as_slice())
        .map_err(|e| format!("Could not append json file \"{filename}\" to tarball: {e}"))?;
    Ok(())
}

fn tar_write_raw_file(tar: &mut tar::Builder<zstd::Encoder<File>>, file_path: &str, data: &[u8]) -> Result<(), String> {
    let mut header = tar::Header::new_gnu();
    header.set_path(&file_path)
        .map_err(|e| format!("Could not set tar file path for raw file \"{file_path}\": {e}"))?;
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(get_current_unix_time());
    header.set_cksum();   // has to be called last
    tar.append(&header, data)
        .map_err(|e| format!("Could not append raw file \"{file_path}\" to tarball: {e}"))?;
    Ok(())
}

fn get_current_unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time travellers are not allowed")
        .as_secs()
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
    pub fn convert_function_ref(&self, gm_function_ref: GMRef<GMFunction>) -> Result<ModRef, String> {
        convert_reference(gm_function_ref, &self.original_data.functions.functions_by_index, &self.modified_data.functions.functions_by_index)
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
    pub fn convert_variable_ref(&self, gm_variable_ref: GMRef<GMVariable>) -> Result<ModRef, String> {
        convert_reference(gm_variable_ref, &self.original_data.variables.variables, &self.modified_data.variables.variables)
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
    pub fn convert_string_ref_opt(&self, gm_string_ref: Option<GMRef<String>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_string_ref, &self.original_data.strings.strings_by_index, &self.modified_data.strings.strings_by_index)
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
pub fn edit_field_option<T: PartialEq + Clone>(original: Option<T>, modified: Option<T>) -> Option<Option<T>> {
    if original != modified {
        Some(modified)
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

pub fn convert_additions<GM, ADD>(gm_elements: &[GM], map_addition: impl Fn(&GM) -> Result<ADD, String>) -> Result<Vec<ADD>, String> {
    gm_elements.iter().map(map_addition).collect()
}

