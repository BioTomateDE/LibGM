use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::deserialize::backgrounds::{parse_chunk_bgnd, GMBackgrounds};
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::code::{parse_chunk_code, GMCodes};
use crate::deserialize::embedded_audio::{parse_chunk_audo, GMEmbeddedAudios};
use crate::deserialize::embedded_textures::{parse_chunk_txtr, GMEmbeddedTexture};
use crate::deserialize::fonts::{parse_chunk_font, GMFonts};
use crate::deserialize::functions::{parse_chunk_func, GMCodeLocal, GMFunctions};
use crate::deserialize::game_objects::{parse_chunk_objt, GMGameObjects};
use crate::deserialize::general_info::{parse_chunk_gen8, parse_chunk_optn};
use crate::deserialize::scripts::{parse_chunk_scpt, GMScripts};
use crate::deserialize::strings::{parse_chunk_strg, GMStrings};
use crate::deserialize::variables::{parse_chunk_vari, GMVariables};
use crate::deserialize::general_info::{GMGeneralInfo, GMOptions};
use crate::deserialize::paths::{parse_chunk_path, GMPaths};
use crate::deserialize::rooms::{parse_chunk_room, GMRooms};
use crate::deserialize::sounds::{parse_chunk_sond, GMSounds};
use crate::deserialize::sprites::{parse_chunk_sprt, GMSprites};
use crate::deserialize::texture_page_items::{parse_chunk_tpag, GMTextures};


#[derive(Debug, Clone)]
pub struct GMData {
    pub strings: GMStrings,                 // STRG
    pub general_info: GMGeneralInfo,        // GEN8
    pub options: GMOptions,                 // OPTN
    pub textures: GMTextures,               // TPAG  (and TXTR)
    pub backgrounds: GMBackgrounds,         // BGND
    pub sprites: GMSprites,                 // SPRT
    pub scripts: GMScripts,                 // SCPT
    pub variables: GMVariables,             // VARI
    pub functions: GMFunctions,             // FUNC
    pub code_locals: Vec<GMCodeLocal>,      // FUNC
    pub codes: GMCodes,                     // CODE
    pub fonts: GMFonts,                     // FONT
    pub audios: GMEmbeddedAudios,           // AUDO
    pub sounds: GMSounds,                   // SOND
    pub game_objects: GMGameObjects,        // OBJT
    pub rooms: GMRooms,                     // ROOM
    pub paths: GMPaths,                     // PATH
}

pub fn parse_data_file(raw_data: Vec<u8>) -> Result<GMData, String> {
    let mut all = GMChunk {
        name: "".to_string(),
        abs_pos: 0,
        data: &raw_data,
        cur_pos: 0
    };

    if all.read_chunk_name()? != "FORM" {
        return Err("Invalid or corrupted data.win file: 'FORM' chunk missing!".to_string());
    }

    // get chunks
    let raw_data_len: usize = all.read_usize()? + all.cur_pos;
    let mut chunks: HashMap<String, GMChunk> = HashMap::new();

    while all.cur_pos + 8 < raw_data_len {
        let chunk_name: String = all.read_chunk_name()?;
        let chunk_length: usize = all.read_usize()?;
        let chunk_data: &[u8] = match all.data.get(all.cur_pos.. all.cur_pos + chunk_length) {
            Some(bytes) => bytes,
            None => return Err(format!(
                "Chunk '{}' with specified length {} is out of bounds at absolute position {} while reading chunks: {} > {}.",
                chunk_name, chunk_length, all.cur_pos, all.cur_pos + chunk_length, all.data.len(),
            )),
        };
        // println!("{} {}", chunk_name, all.file_index);
        chunks.insert(
            chunk_name.clone(),
            GMChunk {
                name: chunk_name,
                abs_pos: all.cur_pos,
                data: &chunk_data,
                cur_pos: 0,
            },
        );
        all.cur_pos += chunk_length;
    }

    let mut chunk_strg: GMChunk = get_chunk(&chunks, "STRG")?;
    let mut chunk_gen8: GMChunk = get_chunk(&chunks, "GEN8")?;
    let mut chunk_optn: GMChunk = get_chunk(&chunks, "OPTN")?;
    let mut chunk_txtr: GMChunk = get_chunk(&chunks, "TXTR")?;
    let mut chunk_tpag: GMChunk = get_chunk(&chunks, "TPAG")?;
    let mut chunk_bgnd: GMChunk = get_chunk(&chunks, "BGND")?;
    let mut chunk_sprt: GMChunk = get_chunk(&chunks, "SPRT")?;
    let mut chunk_scpt: GMChunk = get_chunk(&chunks, "SCPT")?;
    let mut chunk_func: GMChunk = get_chunk(&chunks, "FUNC")?;
    let mut chunk_vari: GMChunk = get_chunk(&chunks, "VARI")?;
    let mut chunk_code: GMChunk = get_chunk(&chunks, "CODE")?;
    let mut chunk_font: GMChunk = get_chunk(&chunks, "FONT")?;
    let mut chunk_audo: GMChunk = get_chunk(&chunks, "AUDO")?;
    let mut chunk_sond: GMChunk = get_chunk(&chunks, "SOND")?;
    let mut chunk_room: GMChunk = get_chunk(&chunks, "ROOM")?;
    let mut chunk_objt: GMChunk = get_chunk(&chunks, "OBJT")?;
    let mut chunk_path: GMChunk = get_chunk(&chunks, "PATH")?;

    let strings: GMStrings = parse_chunk_strg(&mut chunk_strg)?;
    // dbg!(strings.get_string_by_pos(12028677).unwrap().resolve(&strings)?);
    let general_info: GMGeneralInfo = parse_chunk_gen8(&mut chunk_gen8, &strings)?;
    let bytecode14: bool = general_info.bytecode_version <= 14;
    let options: GMOptions = parse_chunk_optn(&mut chunk_optn)?;
    let texture_pages: Vec<GMEmbeddedTexture> = parse_chunk_txtr(&mut chunk_txtr, &general_info)?;
    let textures: GMTextures = parse_chunk_tpag(&mut chunk_tpag, texture_pages)?;
    let backgrounds: GMBackgrounds = parse_chunk_bgnd(&mut chunk_bgnd, &general_info, &strings, &textures)?;
    let sprites: GMSprites = parse_chunk_sprt(&mut chunk_sprt, &general_info, &strings, &textures)?;
    let scripts: GMScripts = parse_chunk_scpt(&mut chunk_scpt, &strings)?;
    let variables: GMVariables = parse_chunk_vari(&mut chunk_vari, &strings, &general_info, &mut chunk_code)?;
    let (functions, code_locals): (GMFunctions, Vec<GMCodeLocal>) = parse_chunk_func(&mut chunk_func, &strings, &chunk_code)?;
    let codes: GMCodes = parse_chunk_code(&mut chunk_code, bytecode14, &strings, &variables, &functions)?;
    let fonts: GMFonts = parse_chunk_font(&mut chunk_font, &general_info, &strings)?;
    let audios: GMEmbeddedAudios = parse_chunk_audo(&mut chunk_audo)?;
    let sounds: GMSounds = parse_chunk_sond(&mut chunk_sond, &general_info, &strings)?;
    let game_objects: GMGameObjects = parse_chunk_objt(&mut chunk_objt, &general_info, &strings)?;
    let rooms: GMRooms = parse_chunk_room(&mut chunk_room, &general_info, &strings)?;
    let paths: GMPaths = parse_chunk_path(&mut chunk_path, &strings)?;

    let data = GMData {
        strings,
        general_info,
        options,
        textures,
        backgrounds,
        sprites,
        scripts,
        variables,
        functions,
        code_locals,
        codes,
        fonts,
        audios,
        sounds,
        game_objects,
        rooms,
        paths,
    };

    Ok(data)
}


pub fn read_data_file(data_file_path: &Path) -> Result<Vec<u8>, String> {
    fs::read(data_file_path)
        .map_err(|e| format!("Could not read data file with path \"{}\": {e}", data_file_path.display()))
}

fn get_chunk<'a>(chunks: &HashMap<String, GMChunk<'a>>, chunk_name: &str) -> Result<GMChunk<'a>, String> {
    match chunks.get(chunk_name) {
        None => Err(format!(
            "Chunk '{}' is missing in data file (chunk hashmap length: {}).",
            chunk_name,
            chunks.len()
        )),
        Some(chunk) => Ok(chunk.clone())
    }
}

