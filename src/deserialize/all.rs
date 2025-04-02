use std::collections::HashMap;
use std::fs;
use crate::deserialize::backgrounds::{parse_chunk_BGND, UTBackgrounds};
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::code::{parse_chunk_CODE, UTCode};
use crate::deserialize::embedded_audio::{parse_chunk_AUDO, UTEmbeddedAudios};
use crate::deserialize::embedded_textures::{parse_chunk_TXTR, UTEmbeddedTexture};
use crate::deserialize::fonts::{parse_chunk_FONT, UTFont};
use crate::deserialize::functions::{parse_chunk_FUNC, UTCodeLocal, UTFunctions};
use crate::deserialize::game_objects::{parse_chunk_OBJT, UTGameObjects};
use crate::deserialize::general_info::{parse_chunk_GEN8, parse_chunk_OPTN};
use crate::deserialize::scripts::{parse_chunk_SCPT, UTScript};
use crate::deserialize::strings::{parse_chunk_STRG, UTStrings};
use crate::deserialize::variables::{parse_chunk_VARI, UTVariable};
use crate::deserialize::general_info::{UTGeneralInfo, UTOptions};
use crate::deserialize::rooms::{parse_chunk_ROOM, UTRoom};
use crate::deserialize::sounds::{parse_chunk_SOND, UTSounds};
use crate::deserialize::sprites::{parse_chunk_SPRT, UTSprites};
use crate::deserialize::texture_page_item::{parse_chunk_TPAG, UTTextures};


#[derive(Debug, Clone)]
pub struct UTData {
    pub strings: UTStrings,                 // STRG
    pub general_info: UTGeneralInfo,        // GEN8
    pub options: UTOptions,                 // OPTN
    pub textures: UTTextures,               // TPAG  (and TXTR)
    pub backgrounds: UTBackgrounds,         // BGND
    pub sprites: UTSprites,                 // SPRT
    pub scripts: Vec<UTScript>,             // SCPT
    pub variables: Vec<UTVariable>,         // VARI
    pub functions: UTFunctions,             // FUNC
    pub code_locals: Vec<UTCodeLocal>,      // FUNC
    pub code: Vec<UTCode>,                  // CODE
    pub fonts: Vec<UTFont>,                 // FONT
    pub audios: UTEmbeddedAudios,           // AUDO
    pub sounds: UTSounds,                   // SOND
    pub game_objects: UTGameObjects,        // OBJT
    pub rooms: Vec<UTRoom>,                 // ROOM
}

pub fn parse_data_file(raw_data: Vec<u8>) -> Result<UTData, String> {
    let mut all = UTChunk {
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
    let mut chunks: HashMap<String, UTChunk> = HashMap::new();

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
            UTChunk {
                name: chunk_name,
                abs_pos: all.file_index,
                data: &chunk_data,
                file_index: 0,
            },
        );
        all.file_index += chunk_length;
    }

    #[allow(non_snake_case)] let mut chunk_STRG: UTChunk = get_chunk(&chunks, "STRG")?;
    #[allow(non_snake_case)] let mut chunk_GEN8: UTChunk = get_chunk(&chunks, "GEN8")?;
    #[allow(non_snake_case)] let mut chunk_OPTN: UTChunk = get_chunk(&chunks, "OPTN")?;
    #[allow(non_snake_case)] let mut chunk_TXTR: UTChunk = get_chunk(&chunks, "TXTR")?;
    #[allow(non_snake_case)] let mut chunk_TPAG: UTChunk = get_chunk(&chunks, "TPAG")?;
    #[allow(non_snake_case)] let mut chunk_BGND: UTChunk = get_chunk(&chunks, "BGND")?;
    #[allow(non_snake_case)] let mut chunk_SPRT: UTChunk = get_chunk(&chunks, "SPRT")?;
    #[allow(non_snake_case)] let mut chunk_SCPT: UTChunk = get_chunk(&chunks, "SCPT")?;
    #[allow(non_snake_case)] let mut chunk_FUNC: UTChunk = get_chunk(&chunks, "FUNC")?;
    #[allow(non_snake_case)] let mut chunk_VARI: UTChunk = get_chunk(&chunks, "VARI")?;
    #[allow(non_snake_case)] let mut chunk_CODE: UTChunk = get_chunk(&chunks, "CODE")?;
    #[allow(non_snake_case)] let mut chunk_FONT: UTChunk = get_chunk(&chunks, "FONT")?;
    #[allow(non_snake_case)] let mut chunk_AUDO: UTChunk = get_chunk(&chunks, "AUDO")?;
    #[allow(non_snake_case)] let mut chunk_SOND: UTChunk = get_chunk(&chunks, "SOND")?;
    #[allow(non_snake_case)] let mut chunk_ROOM: UTChunk = get_chunk(&chunks, "ROOM")?;
    #[allow(non_snake_case)] let mut chunk_OBJT: UTChunk = get_chunk(&chunks, "OBJT")?;

    let strings: UTStrings = parse_chunk_STRG(&mut chunk_STRG)?;
    // dbg!(strings.get_string_by_pos(12028677).unwrap().resolve(&strings)?);
    let general_info: UTGeneralInfo = parse_chunk_GEN8(&mut chunk_GEN8, &strings)?;
    let bytecode14: bool = general_info.bytecode_version >= 14;
    let options: UTOptions = parse_chunk_OPTN(&mut chunk_OPTN)?;
    let texture_pages: Vec<UTEmbeddedTexture> = parse_chunk_TXTR(&mut chunk_TXTR, &general_info)?;
    let textures: UTTextures = parse_chunk_TPAG(&mut chunk_TPAG, texture_pages)?;
    let backgrounds: UTBackgrounds = parse_chunk_BGND(&mut chunk_BGND, &general_info, &strings, &textures)?;
    let sprites: UTSprites = parse_chunk_SPRT(&mut chunk_SPRT, &general_info, &strings, &textures)?;
    let scripts: Vec<UTScript> = parse_chunk_SCPT(&mut chunk_SCPT, &strings)?;
    let variables: Vec<UTVariable> = parse_chunk_VARI(&mut chunk_VARI, &strings)?;
    let (functions, code_locals): (UTFunctions, Vec<UTCodeLocal>) = parse_chunk_FUNC(&mut chunk_FUNC, &strings, &chunk_CODE)?;
    let code: Vec<UTCode> = parse_chunk_CODE(&mut chunk_CODE, bytecode14, &strings, &variables, &functions)?;
    let fonts: Vec<UTFont> = parse_chunk_FONT(&mut chunk_FONT, &general_info, &strings)?;
    let audios: UTEmbeddedAudios = parse_chunk_AUDO(&mut chunk_AUDO)?;
    let sounds: UTSounds = parse_chunk_SOND(&mut chunk_SOND, &general_info, &strings, &audios)?;
    let game_objects: UTGameObjects = parse_chunk_OBJT(&mut chunk_OBJT, &general_info, &strings)?;
    let rooms: Vec<UTRoom> = parse_chunk_ROOM(&mut chunk_ROOM, &general_info, &strings, &backgrounds, &game_objects)?;

    // for i in &rooms {for j in &i.backgrounds {j.print()}}
    // for i in &sounds.sounds_by_index { i.print(&strings)?; }
    // for i in &game_objects.game_objects_by_index { i.print(&strings)?; }

    let data = UTData {
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


fn get_chunk<'a>(chunks: &HashMap<String, UTChunk<'a>>, chunk_name: &str) -> Result<UTChunk<'a>, String> {
    match chunks.get(chunk_name) {
        None => Err(format!(
            "Chunk '{}' is missing in data file (chunk hashmap length: {}).",
            chunk_name,
            chunks.len()
        )),
        Some(chunk) => Ok(chunk.clone())
    }
}

