use std::collections::HashMap;
use std::fs;
use crate::deserialize::backgrounds::{parse_chunk_BGND, GMBackgrounds};
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::code::{parse_chunk_CODE, GMCode};
use crate::deserialize::embedded_audio::{parse_chunk_AUDO, GMEmbeddedAudios};
use crate::deserialize::embedded_textures::{parse_chunk_TXTR, GMEmbeddedTexture};
use crate::deserialize::fonts::{parse_chunk_FONT, GMFonts};
use crate::deserialize::functions::{parse_chunk_FUNC, GMCodeLocal, GMFunctions};
use crate::deserialize::game_objects::{parse_chunk_OBJT, GMGameObjects};
use crate::deserialize::general_info::{parse_chunk_GEN8, parse_chunk_OPTN};
use crate::deserialize::scripts::{parse_chunk_SCPT, GMScripts};
use crate::deserialize::strings::{parse_chunk_STRG, GMStrings};
use crate::deserialize::variables::{parse_chunk_VARI, GMVariable};
use crate::deserialize::general_info::{GMGeneralInfo, GMOptions};
use crate::deserialize::rooms::{parse_chunk_ROOM, GMRoom};
use crate::deserialize::sounds::{parse_chunk_SOND, GMSounds};
use crate::deserialize::sprites::{parse_chunk_SPRT, GMSprites};
use crate::deserialize::texture_page_items::{parse_chunk_TPAG, GMTextures};


#[derive(Debug, Clone)]
pub struct GMData {
    pub strings: GMStrings,                 // STRG
    pub general_info: GMGeneralInfo,        // GEN8
    pub options: GMOptions,                 // OPTN
    pub textures: GMTextures,               // TPAG  (and TXTR)
    pub backgrounds: GMBackgrounds,         // BGND
    pub sprites: GMSprites,                 // SPRT
    pub scripts: GMScripts,                 // SCPT
    pub variables: Vec<GMVariable>,         // VARI
    pub functions: GMFunctions,             // FUNC
    pub code_locals: Vec<GMCodeLocal>,      // FUNC
    pub code: Vec<GMCode>,                  // CODE
    pub fonts: GMFonts,                     // FONT
    pub audios: GMEmbeddedAudios,           // AUDO
    pub sounds: GMSounds,                   // SOND
    pub game_objects: GMGameObjects,        // OBJT
    pub rooms: Vec<GMRoom>,                 // ROOM
}

pub fn parse_data_file(raw_data: Vec<u8>) -> Result<GMData, String> {
    let mut all = GMChunk {
        name: "".to_string(),
        abs_pos: 0,
        data: &raw_data,
        file_index: 0
    };

    if all.read_chunk_name()? != "FORM" {
        return Err("Invalid or corrupted data.win file: 'FORM' chunk missing!".to_string());
    }

    // get chunks
    let raw_data_len: usize = all.read_usize()? + all.file_index;
    let mut chunks: HashMap<String, GMChunk> = HashMap::new();

    while all.file_index + 8 < raw_data_len {
        let chunk_name: String = all.read_chunk_name()?;
        let chunk_length: usize = all.read_usize()?;
        let chunk_data: &[u8] = match all.data.get(all.file_index .. all.file_index + chunk_length) {
            Some(bytes) => bytes,
            None => return Err(format!(
                "Chunk '{}' with specified length {} is out of bounds at absolute position {} while reading chunks: {} > {}.",
                chunk_name, chunk_length, all.file_index, all.file_index + chunk_length, all.data.len(),
            )),
        };
        // println!("{} {}", chunk_name, all.file_index);
        chunks.insert(
            chunk_name.clone(),
            GMChunk {
                name: chunk_name,
                abs_pos: all.file_index,
                data: &chunk_data,
                file_index: 0,
            },
        );
        all.file_index += chunk_length;
    }

    #[allow(non_snake_case)] let mut chunk_STRG: GMChunk = get_chunk(&chunks, "STRG")?;
    #[allow(non_snake_case)] let mut chunk_GEN8: GMChunk = get_chunk(&chunks, "GEN8")?;
    #[allow(non_snake_case)] let mut chunk_OPTN: GMChunk = get_chunk(&chunks, "OPTN")?;
    #[allow(non_snake_case)] let mut chunk_TXTR: GMChunk = get_chunk(&chunks, "TXTR")?;
    #[allow(non_snake_case)] let mut chunk_TPAG: GMChunk = get_chunk(&chunks, "TPAG")?;
    #[allow(non_snake_case)] let mut chunk_BGND: GMChunk = get_chunk(&chunks, "BGND")?;
    #[allow(non_snake_case)] let mut chunk_SPRT: GMChunk = get_chunk(&chunks, "SPRT")?;
    #[allow(non_snake_case)] let mut chunk_SCPT: GMChunk = get_chunk(&chunks, "SCPT")?;
    #[allow(non_snake_case)] let mut chunk_FUNC: GMChunk = get_chunk(&chunks, "FUNC")?;
    #[allow(non_snake_case)] let mut chunk_VARI: GMChunk = get_chunk(&chunks, "VARI")?;
    #[allow(non_snake_case)] let mut chunk_CODE: GMChunk = get_chunk(&chunks, "CODE")?;
    #[allow(non_snake_case)] let mut chunk_FONT: GMChunk = get_chunk(&chunks, "FONT")?;
    #[allow(non_snake_case)] let mut chunk_AUDO: GMChunk = get_chunk(&chunks, "AUDO")?;
    #[allow(non_snake_case)] let mut chunk_SOND: GMChunk = get_chunk(&chunks, "SOND")?;
    #[allow(non_snake_case)] let mut chunk_ROOM: GMChunk = get_chunk(&chunks, "ROOM")?;
    #[allow(non_snake_case)] let mut chunk_OBJT: GMChunk = get_chunk(&chunks, "OBJT")?;

    let strings: GMStrings = parse_chunk_STRG(&mut chunk_STRG)?;
    // dbg!(strings.get_string_by_pos(12028677).unwrap().resolve(&strings)?);
    let general_info: GMGeneralInfo = parse_chunk_GEN8(&mut chunk_GEN8, &strings)?;
    let bytecode14: bool = general_info.bytecode_version >= 14;
    let options: GMOptions = parse_chunk_OPTN(&mut chunk_OPTN)?;
    let texture_pages: Vec<GMEmbeddedTexture> = parse_chunk_TXTR(&mut chunk_TXTR, &general_info)?;
    let textures: GMTextures = parse_chunk_TPAG(&mut chunk_TPAG, texture_pages)?;
    let backgrounds: GMBackgrounds = parse_chunk_BGND(&mut chunk_BGND, &general_info, &strings, &textures)?;
    let sprites: GMSprites = parse_chunk_SPRT(&mut chunk_SPRT, &general_info, &strings, &textures)?;
    let scripts: GMScripts = parse_chunk_SCPT(&mut chunk_SCPT, &strings)?;
    let variables: Vec<GMVariable> = parse_chunk_VARI(&mut chunk_VARI, &strings)?;
    let (functions, code_locals): (GMFunctions, Vec<GMCodeLocal>) = parse_chunk_FUNC(&mut chunk_FUNC, &strings, &chunk_CODE)?;
    let code: Vec<GMCode> = parse_chunk_CODE(&mut chunk_CODE, bytecode14, &strings, &variables, &functions)?;
    let fonts: GMFonts = parse_chunk_FONT(&mut chunk_FONT, &general_info, &strings)?;
    let audios: GMEmbeddedAudios = parse_chunk_AUDO(&mut chunk_AUDO)?;
    let sounds: GMSounds = parse_chunk_SOND(&mut chunk_SOND, &general_info, &strings, &audios)?;
    let game_objects: GMGameObjects = parse_chunk_OBJT(&mut chunk_OBJT, &general_info, &strings)?;
    let rooms: Vec<GMRoom> = parse_chunk_ROOM(&mut chunk_ROOM, &general_info, &strings, &backgrounds, &game_objects)?;

    // for i in &rooms {for j in &i.backgrounds {j.print()}}
    // for i in &sounds.sounds_by_index { i.print(&strings)?; }
    // for i in &game_objects.game_objects_by_index { i.print(&strings)?; }

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
        code,
        fonts,
        audios,
        sounds,
        game_objects,
        rooms,
    };

    // println!("Total data length: {total_length} bytes");
    // println!("Chunk Sizes:");
    // for (chunk_name, chunk) in &data.chunks {
    //     println!("  {}: {} bytes", chunk_name, chunk.data.len());
    // }

    // ----- Testing -----
    // for (chunk_name, chunk) in &chunks {
    //     let path = format!("./_expdat/{chunk_name}.bin");
    //     match fs::write(path, chunk.data.clone()) {
    //         Ok(_) => (),
    //         Err(err) => eprintln!("Failed to write to file for {chunk_name}: {}", err),
    //     }
    // }
    // ----- ^^^^^^^^ -----

    Ok(data)
}


pub fn read_data_file(data_file_path: &str) -> Result<Vec<u8>, String> {
    match fs::read(data_file_path) {
        Ok(file) => Ok(file),
        Err(error) => {
            Err(format!("Could not read data file: {error}"))
        }
    }
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

