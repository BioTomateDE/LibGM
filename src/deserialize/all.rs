use crate::debug_utils::DurationExt;
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
use crate::bench_parse;

#[derive(Debug, Clone)]
pub struct GMData {
    pub strings: GMStrings,                 // STRG
    pub general_info: GMGeneralInfo,        // GEN8
    pub options: GMOptions,                 // OPTN
    pub texture_pages: Vec<GMEmbeddedTexture>,     // TPAG
    pub texture_page_items: GMTextures,     // TPAG
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
    let t_start = cpu_time::ProcessTime::now();
    
    let mut all = GMChunk {
        name: "".to_string(),
        abs_pos: 0,
        data: &raw_data,
        cur_pos: 0,
    };

    if all.read_chunk_name()? != "FORM" {
        return Err("Invalid or corrupted data.win file: 'FORM' chunk missing".to_string());
    }

    // get chunks
    let raw_data_len: usize = all.read_usize()? + all.cur_pos;
    let mut chunks: HashMap<String, GMChunk> = HashMap::new();

    while all.cur_pos + 8 < raw_data_len {
        let chunk_name: String = all.read_chunk_name()?;
        let chunk_length: usize = all.read_usize()?;
        let chunk_data: &[u8] = all.data.get(all.cur_pos.. all.cur_pos + chunk_length)
            .ok_or_else(|| format!(
                "Chunk '{}' with specified length {} is out of bounds at absolute position {} while reading chunks: {} > {}",
                chunk_name, chunk_length, all.cur_pos, all.cur_pos + chunk_length, all.data.len(),
            ))?;
        
        // {~~} padding stuff

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
    
    log::trace!("Parsing FORM took {}", t_start.elapsed().ms());

    let strings: GMStrings = bench_parse!("STRG", parse_chunk_strg(&mut chunk_strg)?);
    let general_info: GMGeneralInfo = bench_parse!("GEN8", parse_chunk_gen8(&mut chunk_gen8, &strings)?);
    let texture_pages: Vec<GMEmbeddedTexture> = bench_parse!("TXTR", parse_chunk_txtr(&mut chunk_txtr, &general_info)?);
    let texture_page_items: GMTextures = bench_parse!("TPAG", parse_chunk_tpag(&mut chunk_tpag)?);
    let backgrounds: GMBackgrounds = bench_parse!("BGND", parse_chunk_bgnd(&mut chunk_bgnd, &general_info, &strings, &texture_page_items)?);
    let sprites: GMSprites = bench_parse!("SPRT", parse_chunk_sprt(&mut chunk_sprt, &general_info, &strings, &texture_page_items)?);
    let scripts: GMScripts = bench_parse!("SCPT", parse_chunk_scpt(&mut chunk_scpt, &strings)?);
    let variables: GMVariables = bench_parse!("VARI", parse_chunk_vari(&mut chunk_vari, &strings, &general_info, &mut chunk_code)?);
    let (functions, code_locals): (GMFunctions, Vec<GMCodeLocal>) = bench_parse!("FUNC", parse_chunk_func(&mut chunk_func, &general_info, &strings, &mut chunk_code)?);
    let codes: GMCodes = bench_parse!("CODE", parse_chunk_code(&mut chunk_code, general_info.bytecode_version <= 14, &strings, &variables, &functions)?);
    let fonts: GMFonts = bench_parse!("FONT", parse_chunk_font(&mut chunk_font, &general_info, &strings, &texture_page_items)?);
    let audios: GMEmbeddedAudios = bench_parse!("AUDO", parse_chunk_audo(&mut chunk_audo)?);
    let sounds: GMSounds = bench_parse!("SOND", parse_chunk_sond(&mut chunk_sond, &general_info, &strings)?);
    let game_objects: GMGameObjects = bench_parse!("OBJT", parse_chunk_objt(&mut chunk_objt, &general_info, &strings)?);
    let rooms: GMRooms = bench_parse!("ROOM", parse_chunk_room(&mut chunk_room, &general_info, &strings)?);
    let paths: GMPaths = bench_parse!("PATH", parse_chunk_path(&mut chunk_path, &strings)?);
    let options: GMOptions = bench_parse!("OPTN", parse_chunk_optn(&mut chunk_optn, &strings, &texture_page_items)?);

    let data = GMData {
        strings,
        general_info,
        options,
        texture_pages,
        texture_page_items,
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
    chunks.get(chunk_name)
        .map(|i| i.to_owned())
        .ok_or_else(|| format!(
            "Chunk '{}' is missing in data file (chunk hashmap length: {})",
            chunk_name, chunks.len(), 
        ))
}
