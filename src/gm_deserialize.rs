use crate::debug_utils::Stopwatch;
use std::fs;
use std::path::Path;
use crate::gamemaker::backgrounds::{GMBackgrounds};
use crate::gamemaker::chunk_reading::{GMChunk, DataReader};
use crate::gamemaker::code::GMCodes;
use crate::gamemaker::embedded_audio::GMEmbeddedAudios;
use crate::gamemaker::embedded_textures::GMEmbeddedTextures;
use crate::gamemaker::fonts::GMFonts;
use crate::gamemaker::functions::GMFunctions;
use crate::gamemaker::game_objects::GMGameObjects;
use crate::gamemaker::scripts::GMScripts;
use crate::gamemaker::strings::GMStrings;
use crate::gamemaker::variables::GMVariables;
use crate::gamemaker::general_info::GMGeneralInfo;
use crate::gamemaker::paths::GMPaths;
use crate::gamemaker::rooms::GMRooms;
use crate::gamemaker::sounds::GMSounds;
use crate::gamemaker::sprites::GMSprites;
use crate::gamemaker::texture_page_items::GMTexturePageItems;
use crate::gamemaker::options::GMOptions;
use crate::detect_version::detect_gamemaker_version;
use crate::gamemaker::irrelevant::{GMAudioGroups, GMExtensions, GMGameEndScripts, GMGlobalInitScripts, GMLanguageInfo};
use crate::gamemaker::particles::{GMParticleEmitters, GMParticleSystems};

#[derive(Debug, Clone)]
pub struct GMData {
    pub strings: GMStrings,                             // STRG
    pub general_info: GMGeneralInfo,                    // GEN8
    pub embedded_textures: GMEmbeddedTextures,          // TXTR
    pub texture_page_items: GMTexturePageItems,         // TPAG
    pub variables: GMVariables,                         // VARI
    pub functions: GMFunctions,                         // FUNC
    pub scripts: GMScripts,                             // SCPT
    pub codes: GMCodes,                                 // CODE
    pub fonts: GMFonts,                                 // FONT
    pub sprites: GMSprites,                             // SPRT
    pub game_objects: GMGameObjects,                    // OBJT
    pub rooms: GMRooms,                                 // ROOM
    pub backgrounds: GMBackgrounds,                     // BGND
    pub paths: GMPaths,                                 // PATH
    pub audios: GMEmbeddedAudios,                       // AUDO
    pub sounds: GMSounds,                               // SOND
    pub options: GMOptions,                             // OPTN
    pub particle_systems: GMParticleSystems,            // PSYS
    pub particle_emitters: GMParticleEmitters,          // PSEM
    pub language_info: GMLanguageInfo,                  // LANG
    pub extensions: GMExtensions,                       // EXTN
    pub audio_groups: GMAudioGroups,                    // AGRP
    pub global_init_scripts: GMGlobalInitScripts,       // GLOB
    pub game_end_scripts: GMGameEndScripts,             // GMEN
    
    /// Should not be edited; only set by `GMData::read_chunk_padding`.
    pub padding: usize,
}

pub fn parse_data_file(raw_data: Vec<u8>) -> Result<GMData, String> {
    let stopwatch = Stopwatch::start();
    let mut reader = DataReader::new(&raw_data);

    if reader.read_chunk_name()? != "FORM" {
        return Err("Invalid or corrupted data.win file: 'FORM' chunk missing".to_string());
    }
    let total_data_len: usize = reader.read_usize()? + reader.cur_pos;
    
    while reader.cur_pos + 8 < total_data_len { 
        let name: String = reader.read_chunk_name()?;
        let chunk_length: usize = reader.read_usize()?;
        let start_pos: usize = reader.cur_pos;
        
        reader.cur_pos += chunk_length;
        if reader.cur_pos > raw_data.len() {
            return Err(format!(
                "Trying to read chunk '{}' out of data bounds: specified length {} implies chunk \
                end position {}; which is greater than the total data length {}",
                name, chunk_length, reader.cur_pos, raw_data.len(),
            ))
        }
        
        let is_last_chunk: bool = reader.cur_pos == raw_data.len();
        let chunk = GMChunk {
            name: name.clone(),
            start_pos,
            end_pos: reader.cur_pos,
            is_last_chunk,
        };
        
        if let Some(old_chunk) = reader.chunks.insert(name.clone(), chunk.clone()) {
            return Err(format!(
                "Chunk '{}' is defined multiple times: old data range {}..{}; new data range {}..{}",
                name, old_chunk.start_pos, old_chunk.end_pos, chunk.start_pos, chunk.end_pos,
            ))
        }
    }
    log::trace!("Parsing FORM took {stopwatch}");
    
    reader.strings = reader.read_chunk_required("STRG")?;      // FIXME: hopefully this also updates reader.strings
    reader.general_info = reader.read_chunk_required("GEN8")?;
    
    let stopwatch2 = Stopwatch::start();
    if let Some(detected_version) = detect_gamemaker_version(&mut reader)? {
        log::info!("General info specified incorrect GameMaker version {}; automatically detected real version {}", reader.general_info.version, detected_version);
    }
    log::trace!("Detecting GameMaker Version took {stopwatch2}");

    let stopwatch2 = Stopwatch::start();
    let embedded_textures: GMEmbeddedTextures = reader.read_chunk_required("TXTR")?;
    let texture_page_items: GMTexturePageItems = reader.read_chunk_required("TPAG")?;
    let variables: GMVariables = reader.read_chunk_required("VARI")?;
    let functions: GMFunctions = reader.read_chunk_required("FUNC")?;
    let scripts: GMScripts = reader.read_chunk_required("SCPT")?;
    let codes: GMCodes = reader.read_chunk_required("CODE")?;
    let fonts: GMFonts = reader.read_chunk_required("FONT")?;
    let sprites: GMSprites = reader.read_chunk_required("SPRT")?;
    let game_objects: GMGameObjects = reader.read_chunk_required("OBJT")?;
    let rooms: GMRooms = reader.read_chunk_required("ROOM")?;
    let backgrounds: GMBackgrounds = reader.read_chunk_required("BGND")?;
    let paths: GMPaths = reader.read_chunk_required("PATH")?;
    let audios: GMEmbeddedAudios = reader.read_chunk_required("AUDO")?;
    let sounds: GMSounds = reader.read_chunk_required("SOND")?;
    let options: GMOptions = reader.read_chunk_required("OPTN")?;
    // some of these chunks probably aren't actually required; make them optional when issues occur

    let particle_systems: GMParticleSystems = reader.read_chunk_optional("PSYS")?;
    let particle_emitters: GMParticleEmitters = reader.read_chunk_optional("PSEM")?;
    let language_info: GMLanguageInfo = reader.read_chunk_optional("LANG")?;
    let extensions: GMExtensions = reader.read_chunk_optional("EXTN")?;
    let audio_groups: GMAudioGroups = reader.read_chunk_optional("AGRP")?;
    let global_init_scripts: GMGlobalInitScripts = reader.read_chunk_optional("GLOB")?;
    let game_end_scripts: GMGameEndScripts = reader.read_chunk_optional("GMEN")?;
    // TODO implement all other chunks
    
    log::trace!("Parsing chunks took {stopwatch2}");


    let data = GMData {
        strings: reader.strings,
        general_info: reader.general_info,
        embedded_textures,
        texture_page_items,
        variables,
        functions,
        scripts,
        codes,
        fonts,
        sprites,
        game_objects,
        rooms,
        backgrounds,
        paths,
        audios,
        sounds,
        options,
        particle_systems,
        particle_emitters,
        language_info,
        extensions,
        audio_groups,
        global_init_scripts,
        game_end_scripts,
        padding: reader.padding,
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

