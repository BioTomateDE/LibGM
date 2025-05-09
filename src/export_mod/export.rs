use crate::deserialize::all::GMData;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};
use crate::deserialize::backgrounds::GMBackground;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::code::GMCode;
use crate::deserialize::embedded_audio::GMEmbeddedAudio;
use crate::deserialize::fonts::GMFont;
use crate::deserialize::functions::GMFunction;
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::paths::GMPath;
use crate::deserialize::rooms::GMRoom;
use crate::deserialize::scripts::GMScript;
use crate::deserialize::sounds::GMSound;
use crate::deserialize::sprites::GMSprite;
use crate::deserialize::texture_page_items::GMTexture;
use crate::deserialize::variables::GMVariable;
use crate::export_mod::fonts::ModFont;
use crate::export_mod::unordered_list::{export_changes_unordered_list, AModUnorderedListChanges, GModUnorderedListChanges};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModUnorderedRef {
    Edit(usize),
    Add(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModOrderedRef {
    index: usize,
}


pub type ModWriter<'a> = ZipWriter<Cursor<&'a mut Vec<u8>>>;
pub const FILE_OPTIONS: LazyLock<FileOptions<()>> = LazyLock::new(|| 
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

fn zw_write_unordered_list_changes<A: Serialize>(zip_writer: &mut ModWriter, filename: &str, changes: &AModUnorderedListChanges<A>) -> Result<(), String> {
    let string: String = serde_json::to_string_pretty(changes)
        .map_err(|e| format!("Could not convert changes to json for zip file {filename}: {e}"))?;
    let data: &[u8] = string.as_bytes();
    zw_write_file(zip_writer, filename, data)
}


fn export_mod(original_data: &GMData, modified_data: &GMData, target_file: &Path) -> Result<(), String> {
    let mut data: Vec<u8> = Vec::new();
    let buff = Cursor::new(&mut data);
    let mut zip_writer = ZipWriter::new(buff);

    let gm_changes: GModData = export_changes_gamemaker(original_data, modified_data)?;
    let strings: AModUnorderedListChanges<String> = gm_changes.convert_strings(&gm_changes.strings)?;
    let fonts: AModUnorderedListChanges<ModFont> = gm_changes.convert_fonts(&gm_changes.fonts)?;
    // repeat ts for every element

    zw_write_unordered_list_changes(&mut zip_writer, "strings.json", &strings)?;
    zw_write_unordered_list_changes(&mut zip_writer, "fonts.json", &fonts)?;
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


#[derive(Debug, Clone)]
pub struct GModData<'o, 'm> {
    pub modified_data: &'m GMData,
    pub backgrounds: GModUnorderedListChanges<'o, 'm, GMBackground>,
    pub codes: GModUnorderedListChanges<'o, 'm, GMCode>,
    pub audios: GModUnorderedListChanges<'o, 'm, GMEmbeddedAudio>,
    pub textures: GModUnorderedListChanges<'o, 'm, GMTexture>,
    pub fonts: GModUnorderedListChanges<'o, 'm, GMFont>,
    pub functions: GModUnorderedListChanges<'o, 'm, GMFunction>,
    pub game_objects: GModUnorderedListChanges<'o, 'm, GMGameObject>,
    pub paths: GModUnorderedListChanges<'o, 'm, GMPath>,
    pub rooms: GModUnorderedListChanges<'o, 'm, GMRoom>,
    pub scripts: GModUnorderedListChanges<'o, 'm, GMScript>,
    pub sounds: GModUnorderedListChanges<'o, 'm, GMSound>,
    pub sprites: GModUnorderedListChanges<'o, 'm, GMSprite>,
    pub strings: GModUnorderedListChanges<'o, 'm, String>,
    pub variables: GModUnorderedListChanges<'o, 'm, GMVariable>,
}

fn export_changes_gamemaker<'o, 'm>(original_data: &'o GMData, modified_data: &'m GMData) -> Result<GModData<'o, 'm>, String> {
    let strings: GModUnorderedListChanges<String> = export_changes_unordered_list(
        &original_data.strings.strings_by_index,
        &modified_data.strings.strings_by_index,
    )?;

    let backgrounds: GModUnorderedListChanges<GMBackground> = export_changes_unordered_list(
        &original_data.backgrounds.backgrounds_by_index,
        &modified_data.backgrounds.backgrounds_by_index,
    )?;

    let codes: GModUnorderedListChanges<GMCode> = export_changes_unordered_list(
        &original_data.codes.codes_by_index,
        &modified_data.codes.codes_by_index,
    )?;

    let audios: GModUnorderedListChanges<GMEmbeddedAudio> = export_changes_unordered_list(
        &original_data.audios.audios_by_index,
        &modified_data.audios.audios_by_index,
    )?;

    let textures: GModUnorderedListChanges<GMTexture> = export_changes_unordered_list(
        &original_data.textures.textures_by_index,
        &modified_data.textures.textures_by_index,
    )?;

    let fonts: GModUnorderedListChanges<GMFont> = export_changes_unordered_list(
        &original_data.fonts.fonts_by_index,
        &modified_data.fonts.fonts_by_index,
    )?;

    let functions: GModUnorderedListChanges<GMFunction> = export_changes_unordered_list(
        &original_data.functions.functions_by_index,
        &modified_data.functions.functions_by_index,
    )?;

    let game_objects: GModUnorderedListChanges<GMGameObject> = export_changes_unordered_list(
        &original_data.game_objects.game_objects_by_index,
        &modified_data.game_objects.game_objects_by_index,
    )?;

    let paths: GModUnorderedListChanges<GMPath> = export_changes_unordered_list(
        &original_data.paths.paths_by_index,
        &modified_data.paths.paths_by_index,
    )?;

    let rooms: GModUnorderedListChanges<GMRoom> = export_changes_unordered_list(
        &original_data.rooms.rooms_by_index,
        &modified_data.rooms.rooms_by_index,
    )?;

    let scripts: GModUnorderedListChanges<GMScript> = export_changes_unordered_list(
        &original_data.scripts.scripts_by_index,
        &modified_data.scripts.scripts_by_index,
    )?;


    let sounds: GModUnorderedListChanges<GMSound> = export_changes_unordered_list(
        &original_data.sounds.sounds_by_index,
        &modified_data.sounds.sounds_by_index,
    )?;

    let sprites: GModUnorderedListChanges<GMSprite> = export_changes_unordered_list(
        &original_data.sprites.sprites_by_index,
        &modified_data.sprites.sprites_by_index,
    )?;

    let variables: GModUnorderedListChanges<GMVariable> = export_changes_unordered_list(
        &original_data.variables.variables,
        &modified_data.variables.variables,
    )?;

    Ok(GModData {
        modified_data,
        backgrounds,
        codes,
        audios,
        textures,
        fonts,
        functions,
        game_objects,
        paths,
        rooms,
        scripts,
        sounds,
        sprites,
        strings,
        variables,
    })
}


macro_rules! resolve_reference_fn {
    ($fn_name:ident, $field:ident, $type:ty, $lookup_field:ident) => {
        pub fn $fn_name(&self, reference: &GMRef<$type>) -> Result<ModUnorderedRef, String> {
            if self.$field.edits.contains_key(&reference.index) {
                return Ok(ModUnorderedRef::Edit(reference.index));
            }

            let target: &$type = reference.resolve(&self.modified_data.$field.$lookup_field)?;
            for (i, item) in self.$field.additions.iter().enumerate() {
                if item == target {
                    return Ok(ModUnorderedRef::Add(i));
                }
            }

            Err(format!(
                "Could not resolve {} reference for GameMaker index {} in edits \
                list with length {} or additions list with length {}.",
                stringify!($field),
                reference.index,
                self.$field.edits.len(),
                self.$field.additions.len()
            ))
        }
    };
}

impl<'o, 'm> GModData<'o, 'm> {
    resolve_reference_fn!(resolve_background_ref, backgrounds, GMBackground, backgrounds_by_index);
    resolve_reference_fn!(resolve_code_ref, codes, GMCode, codes_by_index);
    resolve_reference_fn!(resolve_audio_ref, audios, GMEmbeddedAudio, audios_by_index);
    resolve_reference_fn!(resolve_texture_ref, textures, GMTexture, textures_by_index);
    resolve_reference_fn!(resolve_font_ref, fonts, GMFont, fonts_by_index);
    resolve_reference_fn!(resolve_function_ref, functions, GMFunction, functions_by_index);
    resolve_reference_fn!(resolve_game_object_ref, game_objects, GMGameObject, game_objects_by_index);
    resolve_reference_fn!(resolve_path_ref, paths, GMPath, paths_by_index);
    resolve_reference_fn!(resolve_room_ref, rooms, GMRoom, rooms_by_index);
    resolve_reference_fn!(resolve_script_ref, scripts, GMScript, scripts_by_index);
    resolve_reference_fn!(resolve_sound_ref, sounds, GMSound, sounds_by_index);
    resolve_reference_fn!(resolve_sprite_ref, sprites, GMSprite, sprites_by_index);
    resolve_reference_fn!(resolve_string_ref, strings, String, strings_by_index);
    resolve_reference_fn!(resolve_variable_ref, variables, GMVariable, variables);
}


pub fn edit_field<'a, T: PartialEq + Clone>(original: &T, modified: &T) -> Option<T> {
    if original == modified {
        Some(modified.clone())
    } else {
        None
    }
}

pub fn edit_field_option<'a, T: PartialEq>(original: &Option<T>, modified: &'a Option<T>) -> &'a Option<T> {
    if original == modified {
        modified
    } else {
        &None
    }
}
