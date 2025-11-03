pub(crate) mod chunk;
mod lists;
mod numbers;
pub(crate) mod reader;
pub mod resources;

use crate::gamemaker::data::{Endianness, GMData};
use crate::gamemaker::deserialize::chunk::GMChunk;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::detect_version::detect_gamemaker_version;
use crate::gamemaker::elements::animation_curves::GMAnimationCurves;
use crate::gamemaker::elements::audio_groups::GMAudioGroups;
use crate::gamemaker::elements::backgrounds::GMBackgrounds;
use crate::gamemaker::elements::code::{GMCodes, check_yyc};
use crate::gamemaker::elements::data_files::GMDataFiles;
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudios;
use crate::gamemaker::elements::embedded_images::GMEmbeddedImages;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTextures;
use crate::gamemaker::elements::extensions::GMExtensions;
use crate::gamemaker::elements::feature_flags::GMFeatureFlags;
use crate::gamemaker::elements::filter_effects::GMFilterEffects;
use crate::gamemaker::elements::fonts::GMFonts;
use crate::gamemaker::elements::functions::GMFunctions;
use crate::gamemaker::elements::game_end::GMGameEndScripts;
use crate::gamemaker::elements::game_objects::GMGameObjects;
use crate::gamemaker::elements::global_init::GMGlobalInitScripts;
use crate::gamemaker::elements::languages::GMLanguageInfo;
use crate::gamemaker::elements::options::GMOptions;
use crate::gamemaker::elements::particle_emitters::GMParticleEmitters;
use crate::gamemaker::elements::particle_systems::GMParticleSystems;
use crate::gamemaker::elements::paths::GMPaths;
use crate::gamemaker::elements::rooms::GMRooms;
use crate::gamemaker::elements::scripts::GMScripts;
use crate::gamemaker::elements::sequence::GMSequences;
use crate::gamemaker::elements::shaders::GMShaders;
use crate::gamemaker::elements::sounds::GMSounds;
use crate::gamemaker::elements::sprites::GMSprites;
use crate::gamemaker::elements::tags::GMTags;
use crate::gamemaker::elements::texture_group_info::GMTextureGroupInfos;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItems;
use crate::gamemaker::elements::timelines::GMTimelines;
use crate::gamemaker::elements::ui_nodes::GMRootUINodes;
use crate::gamemaker::elements::variables::GMVariables;
use crate::gamemaker::gm_version::{GMVersion, LTSBranch};
use crate::prelude::*;
use crate::util::bench::Stopwatch;

/// Parse a GameMaker data file (`data.win`, `game.unx`, etc).
pub fn parse_data_file<T: AsRef<[u8]>>(raw_data: T) -> Result<GMData> {
    // Length assertion
    let raw_data: &[u8] = raw_data.as_ref();
    if raw_data.len() >= i32::MAX as usize {
        bail!("Data file is bigger than 2,147,483,647 bytes which will lead to bugs in LibGM and the runner")
    }

    let stopwatch = Stopwatch::start();
    let mut reader = DataReader::new(&raw_data);

    // Read root chunk and set endianness
    let root_chunk_name = reader.read_chunk_name()?;
    reader.endianness = match root_chunk_name.as_str() {
        "FORM" => Endianness::Little,
        "MROF" => Endianness::Big,
        _ => bail!("Invalid data file: expected root chunk to be 'FORM' but found '{root_chunk_name}'"),
    };
    if reader.endianness == Endianness::Big {
        log::warn!("Big endian format might not work, proceed with caution");
    }

    // Length assertion
    let total_data_len = reader.read_u32()? + reader.cur_pos;
    if total_data_len as usize != raw_data.len() {
        bail!(
            "Specified FORM data length is {} but data is actually {} bytes long",
            total_data_len,
            raw_data.len(),
        );
    }

    // Read chunks into HashMap (FORM)
    while reader.cur_pos + 8 < total_data_len {
        let name = reader.read_chunk_name()?;
        let chunk_length = reader.read_u32()?;
        let start_pos = reader.cur_pos;

        reader.cur_pos += chunk_length;
        if reader.cur_pos > total_data_len {
            bail!(
                "Trying to read chunk '{}' out of data bounds: specified length {} implies chunk \
                end position {}; which is greater than the total data length {}",
                name,
                chunk_length,
                reader.cur_pos,
                total_data_len,
            );
        }

        let is_last_chunk: bool = reader.cur_pos == total_data_len;
        let chunk = GMChunk { start_pos, end_pos: reader.cur_pos, is_last_chunk };

        integrity_assert! {
            !reader.chunks.contains_key(&name),
            "Chunk {name:?} is defined multiple times"
        }
        reader.chunks.insert(name, chunk);
    }

    // Properly initialize GEN8 version before reading chunks.
    reader.specified_version = reader.read_gen8_version()?;

    // The following chunk read order is required:
    // Required: STRG → GEN8 → all others
    //
    // Then (in any order):
    // • [FUNC, VARI] → CODE
    // • TPAG → [BGND, EMBI, FONT, OPTN, SPRT]

    reader.strings = reader.read_chunk()?;
    if reader.strings.is_empty() {
        bail!("STRG chunk does not exist or is empty");
    }
    reader.general_info = reader.read_chunk()?;
    if !reader.general_info.exists {
        bail!("GEN8 chunk does not exist");
    }

    const GMS2: GMVersion = GMVersion::new(2, 0, 0, 0, LTSBranch::PreLTS);
    if reader.specified_version == GMS2 {
        let stopwatch = Stopwatch::start();
        detect_gamemaker_version(&mut reader).context("detecting GameMaker version")?;
        log::trace!("Detecting GameMaker Version took {stopwatch}");
    }

    log::info!(
        "Loading {:?} (GM {}, Bytecode {})",
        reader.resolve_gm_str(reader.general_info.display_name)?,
        reader.general_info.version,
        reader.general_info.bytecode_version,
    );

    let is_yyc: bool = check_yyc(&reader).context("Checking YYC")?;
    let mut variables = GMVariables::default();
    let mut functions = GMFunctions::default();
    let mut codes = GMCodes::default();

    let texture_page_items: GMTexturePageItems = reader.read_chunk()?;

    let mut stopwatch2 = Stopwatch::start();
    if !is_yyc {
        variables = reader.read_chunk()?;
        functions = reader.read_chunk()?;
        stopwatch2 = Stopwatch::start();
        // TODO: builder pattern for parser:
        // - opt out of any integrity check (separated)
        // - enable threaded chunk reading (implement ts)
        codes = reader.read_chunk()?;
    }

    let embedded_textures: GMEmbeddedTextures = reader.read_chunk()?;
    let scripts: GMScripts = reader.read_chunk()?;
    let fonts: GMFonts = reader.read_chunk()?;
    let sprites: GMSprites = reader.read_chunk()?;
    let game_objects: GMGameObjects = reader.read_chunk()?;
    let rooms: GMRooms = reader.read_chunk()?;
    let backgrounds: GMBackgrounds = reader.read_chunk()?;
    let audios: GMEmbeddedAudios = reader.read_chunk()?;
    let sounds: GMSounds = reader.read_chunk()?;
    let paths: GMPaths = reader.read_chunk()?;
    let options: GMOptions = reader.read_chunk()?;
    let sequences: GMSequences = reader.read_chunk()?;
    let particle_systems: GMParticleSystems = reader.read_chunk()?;
    let particle_emitters: GMParticleEmitters = reader.read_chunk()?;
    let language_info: GMLanguageInfo = reader.read_chunk()?;
    let extensions: GMExtensions = reader.read_chunk()?;
    let audio_groups: GMAudioGroups = reader.read_chunk()?;
    let global_init_scripts: GMGlobalInitScripts = reader.read_chunk()?;
    let game_end_scripts: GMGameEndScripts = reader.read_chunk()?;
    let shaders: GMShaders = reader.read_chunk()?;
    let root_ui_nodes: GMRootUINodes = reader.read_chunk()?;
    let timelines: GMTimelines = reader.read_chunk()?;
    let embedded_images: GMEmbeddedImages = reader.read_chunk()?;
    let texture_group_infos: GMTextureGroupInfos = reader.read_chunk()?;
    let tags: GMTags = reader.read_chunk()?;
    let feature_flags: GMFeatureFlags = reader.read_chunk()?;
    let filter_effects: GMFilterEffects = reader.read_chunk()?;
    let animation_curves: GMAnimationCurves = reader.read_chunk()?;
    log::trace!("Reading independent chunks took {stopwatch2}");

    // This chunk is so useless that it is perfectly safe to throw it away.
    let _: GMDataFiles = reader.read_chunk()?;

    // Throw error if not all chunks read to prevent silent data loss
    if !reader.chunks.is_empty() {
        let chunks_str: String = reader.chunks.into_keys().join(", ");
        bail!(
            "Not all chunks in the data file were read, which would lead to data loss when writing.\n\
            The following chunks are unknown or not supported: [{chunks_str}]"
        );
    }

    let data = GMData {
        general_info: reader.general_info,
        strings: reader.strings,
        variables,
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
        timelines,
        embedded_images,
        texture_group_infos,
        tags,
        feature_flags,
        filter_effects,
        animation_curves,

        chunk_padding: reader.chunk_padding,
        endianness: reader.endianness,
        original_data_size: raw_data.len(),
    };

    log::trace!("Parsing data took {stopwatch}");
    Ok(data)
}
