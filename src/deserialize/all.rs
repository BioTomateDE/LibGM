use crate::debug_utils::Stopwatch;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::deserialize::backgrounds::{parse_chunk_bgnd, GMBackgrounds};
use crate::deserialize::chunk_reading::{GMChunk, DataReader, GMElement, GMChunkElement};
use crate::deserialize::code::{parse_chunk_code, GMCodes};
use crate::deserialize::embedded_audio::{parse_chunk_audo, GMEmbeddedAudios};
use crate::deserialize::embedded_textures::{parse_chunk_txtr, GMEmbeddedTexture, GMEmbeddedTextures};
use crate::deserialize::fonts::{parse_chunk_font, GMFonts};
use crate::deserialize::functions::{parse_chunk_func, GMCodeLocal, GMFunctions};
use crate::deserialize::game_objects::{parse_chunk_objt, GMGameObjects};
use crate::deserialize::general_info::{parse_chunk_gen8, GMVersion};
use crate::deserialize::scripts::{parse_chunk_scpt, GMScripts};
use crate::deserialize::strings::{parse_chunk_strg, GMStrings};
use crate::deserialize::variables::{parse_chunk_vari, GMVariables};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::paths::{parse_chunk_path, GMPaths};
use crate::deserialize::rooms::{parse_chunk_room, GMRooms};
use crate::deserialize::sounds::{parse_chunk_sond, GMSounds};
use crate::deserialize::sprites::{parse_chunk_sprt, GMSprites};
use crate::deserialize::texture_page_items::{parse_chunk_tpag, GMTexturePageItems};
use crate::bench_parse;
use crate::deserialize::detect_version::detect_gamemaker_version;
use crate::deserialize::irrelevant::{parse_chunk_agrp, parse_chunk_extn, parse_chunk_glob, parse_chunk_lang, GMAudioGroups, GMExtensions, GMGameEndScripts, GMGlobalInitScripts, GMLanguageInfo};
use crate::deserialize::options::{parse_chunk_optn, GMOptions};
use crate::deserialize::particles::{parse_chunk_psem, parse_chunk_psys, GMParticleEmitters, GMParticleSystems};

#[derive(Debug, Clone)]
pub struct GMData {
    pub strings: GMStrings,                             // STRG
    pub general_info: GMGeneralInfo,                    // GEN8
    pub options: GMOptions,                             // OPTN
    pub texture_pages: Vec<GMEmbeddedTexture>,          // TPAG
    pub texture_page_items: GMTexturePageItems,                 // TPAG
    pub backgrounds: GMBackgrounds,                     // BGND
    pub sprites: GMSprites,                             // SPRT
    pub scripts: GMScripts,                             // SCPT
    pub variables: GMVariables,                         // VARI
    pub functions: GMFunctions,                         // FUNC
    pub code_locals: Vec<GMCodeLocal>,                  // FUNC
    pub codes: GMCodes,                                 // CODE
    pub fonts: GMFonts,                                 // FONT
    pub audios: GMEmbeddedAudios,                       // AUDO
    pub sounds: GMSounds,                               // SOND
    pub game_objects: GMGameObjects,                    // OBJT
    pub rooms: GMRooms,                                 // ROOM
    pub paths: GMPaths,                                 // PATH
    pub particle_systems: GMParticleSystems,            // PSYS
    pub particle_emitters: GMParticleEmitters,          // PSEM
    pub language_root: Option<GMLanguageInfo>,          // LANG
    pub extensions: GMExtensions,                       // EXTN
    pub audio_groups: GMAudioGroups,                    // AGRP
    pub global_inits: GMGlobalInitScripts,                    // GLOB
}

pub fn parse_data_file(raw_data: Vec<u8>) -> Result<GMData, String> {
    let stopwatch = Stopwatch::start();
    let mut strings = GMStrings::empty();
    let mut general_info = GMGeneralInfo::empty();
    let mut reader = DataReader::new(&raw_data, &general_info, &strings);

    if reader.read_chunk_name()? != "FORM" {
        return Err("Invalid or corrupted data.win file: 'FORM' chunk missing".to_string());
    }
    let total_data_len: usize = reader.read_usize()? + reader.cur_pos;

    // let strings: GMStrings; 
    // let general_info: GMGeneralInfo; 
    // let mut texture_pages: Option<GMEmbeddedTextures> = None; 
    // let mut texture_page_items: Option<GMTextures> = None;
    // let mut backgrounds: Option<GMBackgrounds> = None;
    // let mut sprites: Option<GMSprites> = None;
    // let mut scripts: Option<GMScripts> = None;
    // let mut variables: Option<GMVariables> = None;
    // let mut functions: Option<GMFunctions> = None; 
    // let mut code_locals: Option<GMCodeLocals> = None; 
    // let mut codes: Option<GMCodes> = None;
    // let mut fonts: Option<GMFonts> = None;
    // let mut audios: Option<GMEmbeddedAudios> = None;
    // let mut sounds: Option<GMSounds> = None;
    // let mut game_objects: Option<GMGameObjects> = None;
    // let mut rooms: Option<GMRooms> = None;
    // let mut paths: Option<GMPaths> = None;
    // let mut options: Option<GMOptions> = None;
    
    while reader.cur_pos + 8 < total_data_len { 
        let name: String = reader.read_chunk_name()?;
        let chunk_length: usize = reader.read_usize()?;
        let start_pos: usize = reader.cur_pos;
        reader.cur_pos += chunk_length;
        let chunk = GMChunk {
            name: name.clone(),
            start_pos,
            end_pos: reader.cur_pos,
        };
        if let Some(old_chunk) = reader.chunks.insert(name.clone(), chunk.clone()) {
            return Err(format!(
                "Chunk '{}' is defined multiple times: old data range {}..{}; new data range {}..{}",
                name, old_chunk.start_pos, old_chunk.end_pos, chunk.start_pos, chunk.end_pos,
            ))
        }
    }
    
    strings = reader.read_chunk_required("STRG")?;
    general_info = reader.read_chunk_required("GEN8")?;
    let stopwatch2 = Stopwatch::start();
    if let Some(detected_version) = detect_gamemaker_version(&chunks)? {
        log::info!("General info specified incorrect GameMaker version {}; automatically detected real version {}", general_info.version, detected_version);
        general_info.version = detected_version;
    }
    log::trace!("Detecting GameMaker Version took {stopwatch2}");
    
    let embedded_textures: GMEmbeddedTextures = reader.read_chunk_required("TXTR")?;
    let texture_pages: GMTexturePageItems = reader.read_chunk_required("TPAG")?;
    let variables: GMVariables = reader.read_chunk_required("VARI")?;
    let functions: GMFunctions = reader.read_chunk_required("FUNC")?;
    let scripts: GMScripts = reader.read_chunk_required("SCPT")?;
    let codes: GMCodes = reader.read_chunk_required("CODE")?;
    let fonts: GMFonts = reader.read_chunk_required("FONT")?;
    let sprites: GMSprites = reader.read_chunk_required("SPRT")?;
    let game_objects: GMGameObjects = reader.read_chunk_required("OBJT")?;
    let rooms: GMRooms = reader.read_chunk_required("ROOM")?;
    let rooms: GMRooms = reader.read_chunk_required("ROOM")?;
    let backgrounds: GMBackgrounds = reader.read_chunk_required("BGND")?;
    let paths: GMPaths = reader.read_chunk_required("PATH")?;
    let audios: GMEmbeddedAudios = reader.read_chunk_required("AUDO")?;
    let options: GMOptions = reader.read_chunk_required("OPTN")?;
    // some of these probably aren't actually required; make optional when issue occur
    
    let particle_systems: GMParticleSystems = reader.read_chunk_optional("PSYS")?;
    let particle_emitters: GMParticleEmitters = reader.read_chunk_optional("PSEM")?;
    let language_info: GMLanguageInfo = reader.read_chunk_optional("LANG")?;
    let extensions: GMExtensions = reader.read_chunk_optional("EXTN")?;
    let audio_groups: GMAudioGroups = reader.read_chunk_optional("AGRP")?;
    let global_init_scripts: GMGlobalInitScripts = reader.read_chunk_optional("GLOB")?;
    let game_end_scripts: GMGameEndScripts = reader.read_chunk_optional("GMEN")?;
    // TODO implement all other chunks
    
    log::trace!("Parsing FORM took {stopwatch}");

    let strings: GMStrings = bench_parse!("STRG", parse_chunk_strg(&mut chunk_strg)?);
    let mut general_info: GMGeneralInfo = bench_parse!("GEN8", parse_chunk_gen8(&mut chunk_gen8, &strings)?);
    
    

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
        particle_systems,
        particle_emitters,
        language_root,
        extensions,
        audio_groups,
        global_inits,
    };

    log::trace!("Parsing data took {stopwatch}");
    Ok(data)
}


pub fn read_data_file(data_file_path: &Path) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let data: Vec<u8> = fs::read(data_file_path)
        .map_err(|e| format!("Could not read data file with path \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Reading data file took {stopwatch}");
    Ok(data)
}

fn get_chunk<'a>(chunks: &HashMap<String, GMChunk<'a>>, chunk_name: &str) -> Result<GMChunk<'a>, String> {
    chunks.get(chunk_name)
        .map(|i| i.to_owned())  // does not clone chunk data, only metadata
        .ok_or_else(|| format!(
            "Chunk '{}' is missing in data file (chunk hashmap length: {})",
            chunk_name, chunks.len(), 
        ))
}
