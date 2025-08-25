mod lists;
mod chunk;
mod resources;
mod numbers;
mod reader;

pub use chunk::GMChunk;
pub use reader::DataReader;
pub use resources::GMRef;
use crate::gamemaker::data::GMData;
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
use crate::gamemaker::elements::functions::{GMCodeLocals, GMFunctions};
use crate::gamemaker::elements::game_objects::GMGameObjects;
use crate::gamemaker::elements::scripts::GMScripts;
use crate::gamemaker::elements::variables::GMVariables;
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


pub fn parse_data_file(raw_data: &Vec<u8>, allow_unread_chunks: bool) -> Result<GMData, String> {
    let stopwatch = Stopwatch::start();
    let mut reader = DataReader::new(&raw_data);

    let root_chunk_name: String = reader.read_chunk_name()?;
    reader.is_big_endian = match root_chunk_name.as_str() {
        "FORM" => false,
        "MROF" => true,
        _ => return Err(format!("Invalid data file: expected root chunk to be 'FORM' but found '{root_chunk_name}'"))
    };
    if reader.is_big_endian {
        log::warn!("Big endian format might not work, proceed with caution");
    }
    
    let total_data_len: usize = reader.read_usize()? + reader.cur_pos;
    if total_data_len != raw_data.len() {
        return Err(format!(
            "Specified FORM data length is {} but data is actually {} bytes long",
            total_data_len, raw_data.len(),
        ))
    }

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

    let specified_version: GMVersion = reader.general_info.version.clone();
    if specified_version.major >= 2 {
        let stopwatch2 = Stopwatch::start();
        detect_gamemaker_version(&mut reader).map_err(|e| format!("{e}\n↳ while detecting gamemaker version"))?;
        log::trace!("Detecting GameMaker Version took {stopwatch2}");
        log::info!(
            "Loaded game \"{}\" with gamemaker version {} [specified: {}] and bytecode version {}",
            reader.resolve_gm_str(reader.general_info.display_name)?,
            reader.general_info.version,
            specified_version,
            reader.general_info.bytecode_version,
        );
    } else {
        log::info!(
            "Loaded game \"{}\" with gamemaker version {} and bytecode version {}",
            reader.resolve_gm_str(reader.general_info.display_name)?,
            reader.general_info.version,
            reader.general_info.bytecode_version,
        );
    }

    let stopwatch2 = Stopwatch::start();
    
    let functions: GMFunctions;
    let codes: GMCodes;
    if check_yyc(&reader) {
        reader.variables = GMVariables { variables: vec![], b15_header: None, exists: false };
        functions = GMFunctions { functions: vec![], code_locals: GMCodeLocals { code_locals: vec![], exists: false }, exists: false };
        codes = GMCodes { codes: vec![], exists: false }
    } else {
        reader.variables = reader.read_chunk_required("VARI")?;
        functions = reader.read_chunk_required("FUNC")?;
        codes = reader.read_chunk_required("CODE")?;
    }

    let embedded_textures: GMEmbeddedTextures = reader.read_chunk_required("TXTR")?;
    let texture_page_items: GMTexturePageItems = reader.read_chunk_required("TPAG")?;
    let scripts: GMScripts = reader.read_chunk_required("SCPT")?;
    let fonts: GMFonts = reader.read_chunk_required("FONT")?;
    let sprites: GMSprites = reader.read_chunk_required("SPRT")?;
    let game_objects: GMGameObjects = reader.read_chunk_required("OBJT")?;
    let rooms: GMRooms = reader.read_chunk_required("ROOM")?;
    let backgrounds: GMBackgrounds = reader.read_chunk_required("BGND")?;
    let audios: GMEmbeddedAudios = reader.read_chunk_required("AUDO")?;
    let sounds: GMSounds = reader.read_chunk_required("SOND")?;
    // some of these chunks probably aren't actually required; make them optional when issues occur
    let paths: GMPaths = reader.read_chunk_optional("PATH")?;
    let options: GMOptions = reader.read_chunk_optional("OPTN")?;
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
        variables: reader.variables,
        functions,
        embedded_textures,
        texture_page_items,
        codes,
        scripts,
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
        
        chunk_padding: reader.chunk_padding,
        is_big_endian: reader.is_big_endian,
        original_data_size: raw_data.len(),
    };

    log::trace!("Parsing data took {stopwatch}");
    Ok(data)
}


/// Check whether this data file was generated with YYC (YoYoGames Compiler).
/// Should that be the case, the CODE, VARI and FUNC chunks will be empty (or not exist?).
fn check_yyc(reader: &DataReader) -> bool {
    let Some(chunk_code) = reader.chunks.get("CODE") else {return true};
    let Some(chunk_vari) = reader.chunks.get("VARI") else {return true};
    let Some(chunk_func) = reader.chunks.get("FUNC") else {return true};
    chunk_code.end_pos <= chunk_code.start_pos &&
    chunk_vari.end_pos <= chunk_vari.start_pos &&
    chunk_func.end_pos <= chunk_func.start_pos
}

