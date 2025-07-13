use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use crate::bench_export;
use crate::utility::Stopwatch;
use crate::export_mod::elements::backgrounds::{AddBackground, EditBackground};
use crate::export_mod::elements::code::{AddCode, EditCode};
use crate::export_mod::elements::fonts::{AddFont, EditFont};
use crate::export_mod::elements::functions::{AddFunction, EditFunction};
use crate::export_mod::elements::game_objects::{AddGameObject, EditGameObject};
use crate::export_mod::elements::general_info::EditGeneralInfo;
use crate::export_mod::elements::options::EditOptions;
use crate::export_mod::elements::paths::ModPath;
use crate::export_mod::elements::rooms::{AddRoom, EditRoom};
use crate::export_mod::elements::scripts::ModScript;
use crate::export_mod::elements::sounds::{AddSound, EditSound};
use crate::export_mod::elements::sprites::{AddSprite, EditSprite};
use crate::export_mod::unordered_list::EditUnorderedList;
use crate::export_mod::elements::variables::{AddVariable, EditVariable};
use crate::gamemaker::deserialize::{GMData, GMRef};

pub fn export_mod(original_data: &GMData, modified_data: &GMData, target_file_path: &Path) -> Result<(), String> {
    let stopwatch = Stopwatch::start();
    
    // initialize file and tarball
    let file = File::create(target_file_path)
        .map_err(|e| format!("Could not create archive file with path \"{}\": {e}", target_file_path.display()))?;
    let zstd_encoder = zstd::Encoder::new(file, 19)
        .map_err(|e| format!("Could not create ZStd encoder: {e}"))?;
    let mut tar = tar::Builder::new(zstd_encoder);

    let mod_exporter = ModExporter {original_data, modified_data};
    let audios: EditUnorderedList<Vec<u8>, Vec<u8>> = bench_export!("Audio", mod_exporter.export_audios())?;
    let backgrounds: EditUnorderedList<AddBackground, EditBackground> = bench_export!("Backgrounds", mod_exporter.export_backgrounds())?;
    let codes: EditUnorderedList<AddCode, EditCode> = bench_export!("Code", mod_exporter.export_codes())?;
    let fonts: EditUnorderedList<AddFont, EditFont> = bench_export!("Fonts", mod_exporter.export_fonts())?;
    let functions: EditUnorderedList<AddFunction, EditFunction> = bench_export!("Functions", mod_exporter.export_functions())?;
    let game_objects: EditUnorderedList<AddGameObject, EditGameObject> = bench_export!("Game Objects", mod_exporter.export_game_objects())?;
    let general_info: EditGeneralInfo = bench_export!("General Info", mod_exporter.export_general_info())?;
    let options: EditOptions = bench_export!("Options", mod_exporter.export_options())?;
    let paths: EditUnorderedList<ModPath, ModPath> = bench_export!("Paths", mod_exporter.export_paths())?;
    let rooms: EditUnorderedList<AddRoom, EditRoom> = bench_export!("Rooms", mod_exporter.export_rooms())?;
    let scripts: EditUnorderedList<ModScript, ModScript> = bench_export!("Scripts", mod_exporter.export_scripts())?;
    let sounds: EditUnorderedList<AddSound, EditSound> = bench_export!("Sounds", mod_exporter.export_sounds())?;
    let sprites: EditUnorderedList<AddSprite, EditSprite> = bench_export!("Sprites", mod_exporter.export_sprites())?;
    let strings: EditUnorderedList<String, String> = bench_export!("Strings", mod_exporter.export_strings())?;
    let (texture_page_items, images) = bench_export!("Textures", mod_exporter.export_textures())?;
    let variables: EditUnorderedList<AddVariable, EditVariable> = bench_export!("Variables", mod_exporter.export_variables())?;
    log::trace!("Exporting changes took {stopwatch}");

    let stopwatch2 = Stopwatch::start();
    tar_write_json_file(&mut tar, "backgrounds", backgrounds)?;
    tar_write_json_file(&mut tar, "codes", codes)?;
    tar_write_json_file(&mut tar, "fonts", fonts)?;
    tar_write_json_file(&mut tar, "functions", functions)?;
    tar_write_json_file(&mut tar, "game_objects", game_objects)?;
    tar_write_json_file(&mut tar, "general_info", general_info)?;
    tar_write_json_file(&mut tar, "paths", paths)?;
    tar_write_json_file(&mut tar, "options", options)?;
    tar_write_json_file(&mut tar, "rooms", rooms)?;
    tar_write_json_file(&mut tar, "scripts", scripts)?;
    tar_write_json_file(&mut tar, "sounds", sounds)?;
    tar_write_json_file(&mut tar, "sprites", sprites)?;
    tar_write_json_file(&mut tar, "strings", strings)?;
    tar_write_json_file(&mut tar, "textures", texture_page_items)?;
    tar_write_json_file(&mut tar, "variables", variables)?;
    log::trace!("Writing json files took {stopwatch2}");

    // export textures into textures/{i}.png
    let stopwatch2 = Stopwatch::start();
    let image_count = images.len();
    for (i, image) in images.into_iter().enumerate() {
        let file_path: String = format!("textures/{i}.png");
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| format!("Could not encode PNG image: {e}"))?;
        drop(image);
        tar_write_raw_file(&mut tar, &file_path, &buffer.into_inner())?;
    }
    log::trace!("Writing {image_count} images took {stopwatch2}");

    // export audio additions into audio_additions/{i}.wav
    let stopwatch2 = Stopwatch::start();
    let audio_addition_count = audios.additions.len();
    for (i, audio_data) in audios.additions.into_iter().enumerate() {
        tar_write_raw_file(&mut tar, &format!("audio_additions/{i}.wav"), &audio_data)?;
    }
    log::trace!("Writing {audio_addition_count} audio additions took {stopwatch2}");
    
    // export audio edits into audio_edits/{i}.wav
    let stopwatch2 = Stopwatch::start();
    let audio_edit_count = audios.edits.len();
    for (i, audio_data) in audios.edits {
        tar_write_raw_file(&mut tar, &format!("audio_edits/{i}.wav"), &audio_data)?;
    }
    log::trace!("Writing {audio_edit_count} audio edits took {stopwatch2}");

    let stopwatch2 = Stopwatch::start();
    // finalize
    tar.into_inner()
        .map_err(|e| format!("Could not get inner value of tarball: {e}"))?
        .finish()
        .map_err(|e| format!("Could not finish writing tarball: {e}"))?;
    log::trace!("Finalizing tarball took {stopwatch2}");
    
    log::trace!("Exporting changes and writing tarball took {stopwatch}");
    Ok(())
}

fn tar_write_json_file<J: Serialize>(tar: &mut tar::Builder<zstd::Encoder<File>>, name: &str, json_struct: J) -> Result<(), String> {
    let filename: String = format!("{name}.json");

    let data: Vec<u8> = serde_json::to_vec_pretty(&json_struct)
        .map_err(|e| format!("Could not serialize_old {name} changes to json: {e}"))?;

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
    Data(u32),    // index in gamemaker data list; assumes element also exists in the data it will be loaded in
    Add(u32),     // element is being added by this mod; index of this mod's addition list
}

pub struct ModExporter<'o, 'm> {
    pub original_data: &'o GMData,
    pub modified_data: &'m GMData,
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
pub fn edit_field_option<T: PartialEq + Clone>(original: &Option<T>, modified: &Option<T>) -> Option<Option<T>> {
    if original != modified {
        Some(modified.clone())
    } else {
        None
    }
}
pub fn edit_field_convert<GM>(
    original: &GMRef<GM>,
    modified: &GMRef<GM>,
    converter: impl Fn(&GMRef<GM>) -> Result<ModRef, String>,
) -> Result<Option<ModRef>, String> {
    if original.index != modified.index {
        Ok(Some(converter(modified)?))
    } else {
        Ok(None)
    }
}
pub fn edit_field_convert_option<GM: PartialEq, MOD>(
    original: &Option<GM>,
    modified: &Option<GM>,
    converter: impl Fn(&GM) -> Result<MOD, String>,
) -> Result<Option<Option<MOD>>, String> {
    if original == modified {
        if let Some(m) = modified {
            Ok(Some(Some(converter(m)?)))
        } else {
            Ok(Some(None))
        }
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditWrapper<A, E> {
    Add(A),
    Edit(E),
    None,
}

pub fn wrap_edit_option<G, A, E>(
    o: &Option<G>,
    m: &Option<G>,
    add_map: impl Fn(&G) -> Result<A, String>,
    edit_map: impl Fn(&G, &G) -> Result<E, String>,
) -> Result<Option<EditWrapper<A, E>>, String> {
    Ok(match (o, m) {
        (None, None) => None,
        (None, Some(m)) => Some(EditWrapper::Add(add_map(m)?)),
        (Some(_), None) => Some(EditWrapper::None),
        (Some(o), Some(m)) => Some(EditWrapper::Edit(edit_map(o, m)?)),
    })
}

pub fn convert_additions<GM, ADD>(gm_elements: &[GM], map_addition: impl Fn(&GM) -> Result<ADD, String>) -> Result<Vec<ADD>, String> {
    gm_elements.iter().map(map_addition).collect()
}

