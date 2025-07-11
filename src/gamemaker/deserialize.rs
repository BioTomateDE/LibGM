mod lists;
mod chunk;
mod resources;
mod numbers;
mod reader;

pub use chunk::GMChunk;
pub use reader::DataReader;
pub use resources::GMRef;

use crate::utility::Stopwatch;
use crate::gamemaker::detect_version::detect_gamemaker_version;
use crate::gamemaker::gm_version::GMVersion;
use crate::gamemaker::elements::animation_curves::GMAnimationCurves;
use crate::gamemaker::elements::audio_groups::GMAudioGroups;
use crate::gamemaker::elements::backgrounds::GMBackgrounds;
use crate::gamemaker::elements::code::GMCodes;
use crate::gamemaker::elements::data_files::GMDataFiles;
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudios;
use crate::gamemaker::elements::embedded_images::GMEmbeddedImages;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTextures;
use crate::gamemaker::elements::extensions::GMExtensions;
use crate::gamemaker::elements::feature_flags::GMFeatureFlags;
use crate::gamemaker::elements::filter_effects::GMFilterEffects;
use crate::gamemaker::elements::fonts::GMFonts;
use crate::gamemaker::elements::functions::GMFunctions;
use crate::gamemaker::elements::game_objects::GMGameObjects;
use crate::gamemaker::elements::scripts::GMScripts;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::variables::GMVariables;
use crate::gamemaker::elements::general_info::GMGeneralInfo;
use crate::gamemaker::elements::global_init::{GMGameEndScripts, GMGlobalInitScripts};
use crate::gamemaker::elements::languages::GMLanguageInfo;
use crate::gamemaker::elements::paths::GMPaths;
use crate::gamemaker::elements::rooms::GMRooms;
use crate::gamemaker::elements::sounds::GMSounds;
use crate::gamemaker::elements::sprites::GMSprites;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItems;
use crate::gamemaker::elements::options::GMOptions;
use crate::gamemaker::elements::particles::{GMParticleEmitters, GMParticleSystems};
use crate::gamemaker::elements::sequence::GMSequences;
use crate::gamemaker::elements::shaders::GMShaders;
use crate::gamemaker::elements::tags::GMTags;
use crate::gamemaker::elements::texture_group_info::GMTextureGroupInfos;
use crate::gamemaker::elements::ui_nodes::GMRootUINodes;
use crate::gamemaker::elements::timelines::GMTimelines;


#[derive(Debug, Clone)]
pub struct GMData {
    pub general_info: GMGeneralInfo,                    // GEN8
    pub strings: GMStrings,                             // STRG
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
    pub sequences: GMSequences,                         // SEQN
    pub particle_systems: GMParticleSystems,            // PSYS
    pub particle_emitters: GMParticleEmitters,          // PSEM
    pub language_info: GMLanguageInfo,                  // LANG
    pub extensions: GMExtensions,                       // EXTN
    pub audio_groups: GMAudioGroups,                    // AGRP
    pub global_init_scripts: GMGlobalInitScripts,       // GLOB
    pub game_end_scripts: GMGameEndScripts,             // GMEN
    pub shaders: GMShaders,                             // SHDR
    pub root_ui_nodes: GMRootUINodes,                   // UILR
    pub data_files: GMDataFiles,                        // DAFL
    pub timelines: GMTimelines,							// TMLN
    pub embedded_images: GMEmbeddedImages,              // EMBI
    pub texture_group_infos: GMTextureGroupInfos,       // TGIN
    pub tags: GMTags,                                   // TAGS
    pub feature_flags: GMFeatureFlags,                  // FEAT
    pub filter_effects: GMFilterEffects,                // FEDS
    pub animation_curves: GMAnimationCurves,            // ACRV

    /// Should not be edited; only set by `GMData::read_chunk_padding`.
    pub padding: usize,
    /// Size of the original data file; useful for approximating.
    pub original_data_size: usize,
}


pub fn parse_data_file(raw_data: &Vec<u8>, allow_unread_chunks: bool) -> Result<GMData, String> {
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
    
    reader.strings = reader.read_chunk_required("STRG")?;
    reader.general_info = reader.read_chunk_required("GEN8")?;
    
    let stopwatch2 = Stopwatch::start();
    let old_version: GMVersion = reader.general_info.version.clone();
    let detected_version_opt = detect_gamemaker_version(&mut reader)
        .map_err(|e| format!("{e}\n↳ while detecting gamemaker version"))?;
    log::trace!("Detecting GameMaker Version took {stopwatch2}");

    if let Some(detected_version) = detected_version_opt {
        log::info!("Data file specified GameMaker version {}; detected real version {}", old_version, detected_version);
    } else {
        log::info!("Data file specified GameMaker version {}", old_version);
    }

    let stopwatch2 = Stopwatch::start();
    let embedded_textures: GMEmbeddedTextures = reader.read_chunk_required("TXTR")?;
    let texture_page_items: GMTexturePageItems = reader.read_chunk_required("TPAG")?;
    let variables: GMVariables = reader.read_chunk_required("VARI")?;   // variables and functions have to be parsed before code
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

    let sequences: GMSequences = reader.read_chunk_optional("SEQN")?;
    let particle_systems: GMParticleSystems = reader.read_chunk_optional("PSYS")?;
    let particle_emitters: GMParticleEmitters = reader.read_chunk_optional("PSEM")?;
    let language_info: GMLanguageInfo = reader.read_chunk_optional("LANG")?;
    let extensions: GMExtensions = reader.read_chunk_optional("EXTN")?;
    let audio_groups: GMAudioGroups = reader.read_chunk_optional("AGRP")?;
    let global_init_scripts: GMGlobalInitScripts = reader.read_chunk_optional("GLOB")?;
    let game_end_scripts: GMGameEndScripts = reader.read_chunk_optional("GMEN")?;
    let shaders: GMShaders = reader.read_chunk_optional("SHDR")?;
    let root_ui_nodes: GMRootUINodes = reader.read_chunk_optional("UILR")?;
    let data_files: GMDataFiles = reader.read_chunk_optional("DAFL")?;
    let timelines: GMTimelines = reader.read_chunk_optional("TMLN")?;
    let embedded_images: GMEmbeddedImages = reader.read_chunk_optional("EMBI")?;
    let texture_group_infos: GMTextureGroupInfos = reader.read_chunk_optional("TGIN")?;
    let tags: GMTags = reader.read_chunk_optional("TAGS")?;
    let feature_flags: GMFeatureFlags = reader.read_chunk_optional("FEAT")?;
    let filter_effects: GMFilterEffects = reader.read_chunk_optional("FEDS")?;
    let animation_curves: GMAnimationCurves = reader.read_chunk_optional("ACRV")?;
    log::trace!("Parsing chunks took {stopwatch2}");
    
    // Throw error if not all chunks read to prevent silent data loss
    if !allow_unread_chunks && !reader.chunks.is_empty() {
        let chunks_str: String = reader.chunks.keys().cloned().collect::<Vec<_>>().join(", ");
        return Err(format!(
            "Not all chunks in the data file were read, which would lead to data loss when writing.\n\
            The following chunks are unknown or not supported: [{chunks_str}]"
        ))
    }

    let data = GMData {
        general_info: reader.general_info,
        strings: reader.strings,
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
        sequences,
        particle_systems,
        particle_emitters,
        language_info,
        extensions,
        audio_groups,
        global_init_scripts,
        game_end_scripts,
        shaders,
        root_ui_nodes,
        data_files,
        timelines,
        embedded_images,
        texture_group_infos,
        tags,
        feature_flags,
        filter_effects,
        animation_curves,
        padding: reader.padding,
        original_data_size: raw_data.len(),
    };

    log::trace!("Parsing data took {stopwatch}");
    Ok(data)
}

