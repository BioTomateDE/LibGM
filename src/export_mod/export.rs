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
use crate::deserialize::fonts::GMFont;
use crate::deserialize::functions::GMFunction;
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::paths::GMPath;
use crate::deserialize::rooms::GMRoom;
use crate::deserialize::scripts::GMScript;
use crate::deserialize::sounds::GMSound;
use crate::deserialize::sprites::GMSprite;
use crate::deserialize::texture_page_items::GMTexturePageItem;
use crate::deserialize::variables::GMVariable;
use crate::export_mod::fonts::{AddFont, EditFont};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList, GModUnorderedListChanges};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModUnorderedRef {
    Data(usize),    // index in gamemaker data list; assumes element also exists in the data it will be loaded in
    Add(usize),     // element is being added by this mod; index of this mod's addition list
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

fn zw_write_unordered_list_changes<ADD: Serialize, EDIT: Serialize>(zip_writer: &mut ModWriter, filename: &str, changes: &EditUnorderedList<ADD, EDIT>) -> Result<(), String> {
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
    let strings: EditUnorderedList<String, String> = gm_changes.convert_strings(&gm_changes.strings)?;
    let fonts: EditUnorderedList<AddFont, EditFont> = gm_changes.convert_fonts(&gm_changes.fonts)?;
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
    pub original_data: &'o GMData,
    pub modified_data: &'m GMData,
    pub backgrounds: GModUnorderedListChanges<'o, 'm, GMBackground>,
    pub codes: GModUnorderedListChanges<'o, 'm, GMCode>,
    pub audios: GModUnorderedListChanges<'o, 'm, GMEmbeddedAudio>,
    pub texture_page_items: GModUnorderedListChanges<'o, 'm, GMTexturePageItem>,
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

    let texture_page_items: GModUnorderedListChanges<GMTexturePageItem> = export_changes_unordered_list(
        &original_data.texture_page_items.textures_by_index,
        &modified_data.texture_page_items.textures_by_index,
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
        texture_page_items,
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
            // If reference index out of bounds in modified data; throw error.
            // This should never happen in healthy gm data; just being cautious that the mod will be fully functional.
            if reference.index >= self.modified_data.$field.$lookup_field.len() {
                return Err(format!(
                    "Could not resolve {} reference for GameMaker index {} in edits \
                    list with length {} or additions list with length {}", 
                    stringify!($field), reference.index, self.$field.edits.len(), self.$field.additions.len(), 
                ))
            }
            
            let original_length = self.original_data.$field.$lookup_field.len();
            if reference.index >= original_length {
                // If reference index exists (isn't out of bounds) in modified data but not in original data,
                // then the element was newly added --> "Add" reference
                Ok(ModUnorderedRef::Add(reference.index - original_length))
            } else {
                // If reference index exists in original data (and modified data; assumes unordered lists never remove elements),
                // then the element is a reference to the gamemaker data the mod will later be loaded in.
                Ok(ModUnorderedRef::Data(reference.index))
            }
        }
    };
}

macro_rules! resolve_reference_optional_fn {
    ($fn_name:ident, $field:ident, $type:ty, $lookup_field:ident) => {
        pub fn $fn_name(&self, reference: &Option<GMRef<$type>>) -> Result<Option<ModUnorderedRef>, String> {
            let reference: &GMRef<$type> = match reference {
                None => return Ok(None),
                Some(x) => x,
            };

            if reference.index >= self.modified_data.$field.$lookup_field.len() {
                return Err(format!(
                    "Could not resolve optional {} reference for GameMaker index {} in edits \
                    list with length {} or additions list with length {}", 
                    stringify!($field), reference.index, self.$field.edits.len(), self.$field.additions.len(), 
                ))
            }
            
            let original_length = self.original_data.$field.$lookup_field.len();
            if reference.index >= original_length {
                Ok(Some(ModUnorderedRef::Add(reference.index - original_length)))
            } else {
                Ok(Some(ModUnorderedRef::Data(reference.index)))
            }
        }
    };
}

impl<'o, 'm> GModData<'o, 'm> {
    resolve_reference_fn!(resolve_background_ref, backgrounds, GMBackground, backgrounds_by_index);
    resolve_reference_fn!(resolve_code_ref, codes, GMCode, codes_by_index);
    resolve_reference_fn!(resolve_audio_ref, audios, GMEmbeddedAudio, audios_by_index);
    resolve_reference_fn!(resolve_texture_ref, texture_page_items, GMTexturePageItem, textures_by_index);
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

    resolve_reference_optional_fn!(resolve_optional_background_ref, backgrounds, GMBackground, backgrounds_by_index);
    resolve_reference_optional_fn!(resolve_optional_code_ref, codes, GMCode, codes_by_index);
    resolve_reference_optional_fn!(resolve_optional_audio_ref, audios, GMEmbeddedAudio, audios_by_index);
    resolve_reference_optional_fn!(resolve_optional_texture_ref, texture_page_items, GMTexturePageItem, textures_by_index);
    resolve_reference_optional_fn!(resolve_optional_font_ref, fonts, GMFont, fonts_by_index);
    resolve_reference_optional_fn!(resolve_optional_function_ref, functions, GMFunction, functions_by_index);
    resolve_reference_optional_fn!(resolve_optional_game_object_ref, game_objects, GMGameObject, game_objects_by_index);
    resolve_reference_optional_fn!(resolve_optional_path_ref, paths, GMPath, paths_by_index);
    resolve_reference_optional_fn!(resolve_optional_room_ref, rooms, GMRoom, rooms_by_index);
    resolve_reference_optional_fn!(resolve_optional_script_ref, scripts, GMScript, scripts_by_index);
    resolve_reference_optional_fn!(resolve_optional_sound_ref, sounds, GMSound, sounds_by_index);
    resolve_reference_optional_fn!(resolve_optional_sprite_ref, sprites, GMSprite, sprites_by_index);
    resolve_reference_optional_fn!(resolve_optional_string_ref, strings, String, strings_by_index);
    resolve_reference_optional_fn!(resolve_optional_variable_ref, variables, GMVariable, variables);
    
    
    // pub fn ts(&self) -> Result<EditUnorderedList<AddFont, EditFont>, String> {
    //     let changes = export_changes_unordered_list(&self.original_data.fonts.fonts_by_index, &self.modified_data.fonts.fonts_by_index)?;
    //     self.convert_edits(
    //         &changes,
    //         |i| self.convert_font_additions(changes.additions),
    //         |o, m| {
    //             Ok(EditFont {
    //                 name: edit_field(&self.resolve_string_ref(&o.name)?, &self.resolve_string_ref(&m.name)?),
    //                 display_name: edit_field(&self.resolve_string_ref(&o.display_name)?, &self.resolve_string_ref(&m.display_name)?),
    //                 em_size: edit_field(&o.em_size, &m.em_size),
    //                 bold: edit_field(&o.bold, &m.bold),
    //                 italic: edit_field(&o.italic, &m.italic),
    //                 range_start: edit_field(&o.range_start, &m.range_start),
    //                 charset: edit_field(&o.charset, &m.charset),
    //                 anti_alias: edit_field(&o.anti_alias, &m.anti_alias),
    //                 range_end: edit_field(&o.range_end, &m.range_end),
    //                 texture: edit_field(&o.texture, &m.texture),
    //                 scale_x: edit_field(&o.scale_x, &m.scale_x),
    //                 scale_y: edit_field(&o.scale_y, &m.scale_y),
    //                 ascender_offset: edit_field(&o.ascender_offset, &m.ascender_offset),
    //                 ascender: edit_field(&o.ascender, &m.ascender),
    //                 sdf_spread: edit_field(&o.sdf_spread, &m.sdf_spread),
    //                 line_height: edit_field(&o.line_height, &m.line_height),
    //                 glyphs: EditUnorderedList {TODO},
    //             })
    //         }
    //     )
    // }
}


pub fn flag_field(original: bool, modified: bool) -> Option<bool> {
    if original == modified {
        None
    } else {
        Some(modified)
    }
}

pub fn edit_field<'a, T: PartialEq + Clone>(original: &T, modified: &T) -> Option<T> {
    if original == modified {
        Some(modified.clone())
    } else {
        None
    }
}

pub fn edit_field_option<T: PartialEq + Clone>(original: &Option<T>, modified: &Option<T>) -> Option<T> {
    if original == modified {
        modified.clone()
    } else {
        None
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

