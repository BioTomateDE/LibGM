//! Functions and builder structs related to parsing GameMaker data files.
//!
//! Some of these functions are also re-exported at the crate root.

pub(super) mod chunk;
pub mod integrity;
mod lists;
mod numbers;
pub(crate) mod reader;
pub(super) mod resources;

use std::path::Path;

use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::wad::data::Endianness;
use crate::wad::data::GMData;
use crate::wad::data::Metadata;
use crate::wad::deserialize::chunk::ChunkBounds;
use crate::wad::deserialize::chunk::ChunkMap;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::animation_curve::GMAnimationCurves;
use crate::wad::elements::audio::GMAudios;
use crate::wad::elements::audio_group::GMAudioGroups;
use crate::wad::elements::background::GMBackgrounds;
use crate::wad::elements::code::GMCodes;
use crate::wad::elements::code::check_yyc;
use crate::wad::elements::data_file::GMDataFiles;
use crate::wad::elements::embedded_image::GMEmbeddedImages;
use crate::wad::elements::extension::GMExtensions;
use crate::wad::elements::feature_flag::GMFeatureFlags;
use crate::wad::elements::filter_effect::GMFilterEffects;
use crate::wad::elements::font::GMFonts;
use crate::wad::elements::function::GMFunctions;
use crate::wad::elements::game_end::GMGameEndScripts;
use crate::wad::elements::game_object::GMGameObjects;
use crate::wad::elements::global_init::GMGlobalInitScripts;
use crate::wad::elements::language::GMLanguageInfo;
use crate::wad::elements::options::GMOptions;
use crate::wad::elements::particle_emitter::GMParticleEmitters;
use crate::wad::elements::particle_system::GMParticleSystems;
use crate::wad::elements::path::GMPaths;
use crate::wad::elements::room::GMRooms;
use crate::wad::elements::script::GMScripts;
use crate::wad::elements::sequence::GMSequences;
use crate::wad::elements::shader::GMShaders;
use crate::wad::elements::sound::GMSounds;
use crate::wad::elements::sprite::GMSprites;
use crate::wad::elements::string::GMStrings;
use crate::wad::elements::tag::GMTags;
use crate::wad::elements::texture_group_info::GMTextureGroupInfos;
use crate::wad::elements::texture_page::GMTexturePages;
use crate::wad::elements::texture_page_item::GMTexturePageItems;
use crate::wad::elements::timeline::GMTimelines;
use crate::wad::elements::ui_node::GMRootUINodes;
use crate::wad::elements::variable::GMVariables;
use crate::wad::version::GMVersion;
use crate::wad::version_detection::detect_gamemaker_version;

const ERR_TOO_BIG: &str =
    "Data file is bigger than 2,147,483,646 bytes which will lead to bugs in LibGM and the runner";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsingOptions {
    /// See [`ParsingOptions::verify_alignment`].
    pub verify_alignment: bool,

    /// See [`ParsingOptions::verify_constants`].
    pub verify_constants: bool,

    /// See [`ParsingOptions::allow_unknown_chunks`].
    pub allow_unknown_chunks: bool,
}

impl Default for ParsingOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ParsingOptions {
    /// Creates a new [`ParsingOptions`] with default settings.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            verify_alignment: true,
            verify_constants: true,
            allow_unknown_chunks: false,
        }
    }

    /// When enabled, verifies that all pointer offsets and data structures
    /// are properly aligned according to their type requirements.
    ///
    /// This helps detect corrupted or malformed data files by ensuring
    /// all memory accesses occur on correct boundaries.
    ///
    /// Disable this flag if your data file has uncommon/malformed alignment
    /// (e.g. a port to a niche platform), but you want to try to parse it
    /// anyway.
    ///
    /// > Default: **true**
    #[inline]
    #[must_use]
    pub const fn verify_alignment(mut self, enabled: bool) -> Self {
        self.verify_alignment = enabled;
        self
    }

    /// When enabled, validates that known constant values in the data format
    /// match their expected values (e.g., reserved fields that should be zero
    /// or deprecated values that are always the same for compatibility).
    ///
    /// This provides additional validation against data corruption or version
    /// mismatches.
    ///
    /// > Default: **true**
    #[inline]
    #[must_use]
    pub const fn verify_constants(mut self, enabled: bool) -> Self {
        self.verify_constants = enabled;
        self
    }

    /// When **disabled**, requires that all data chunks in the file are
    /// processed during parsing.
    ///
    /// This prevents silent data loss when rebuilding data by ensuring
    /// no unrecognized or unsupported chunks are ignored. If any chunks
    /// remain unread after parsing completes, an error will be returned.
    ///
    /// > Default: **false**
    #[inline]
    #[must_use]
    pub const fn allow_unknown_chunks(mut self, enabled: bool) -> Self {
        self.allow_unknown_chunks = enabled;
        self
    }

    /// Parses a GameMaker data file (stored in memory) with the specified
    /// options.
    ///
    /// If you want to parse a data file stored on disk, check out
    /// [`ParsingOptions::parse_file`].
    ///
    /// For more information on the data file format, see [`crate::wad`].
    pub fn parse_bytes(&self, raw_data: impl AsRef<[u8]>) -> Result<GMData> {
        self.parse(raw_data.as_ref())
            .context("parsing GameMaker data bytes")
    }

    /// Parses a GameMaker data file (stored on disk) with the specified
    /// options.
    ///
    /// If you want to parse a data file stored in memory, check out
    /// [`ParsingOptions::parse_bytes`].
    ///
    /// For more information on the data file format, see [`crate::wad`].
    pub fn parse_file(&self, data_file_path: impl AsRef<Path>) -> Result<GMData> {
        let path = data_file_path.as_ref();

        let meta = std::fs::metadata(path)
            .with_context_src(|| format!("reading metadata of data file {}", path.display()))?;

        if meta.len() >= i32::MAX as u64 {
            bail!("{ERR_TOO_BIG}");
        }

        let stopwatch = Stopwatch::start();
        let raw_data: Vec<u8> = std::fs::read(path)
            .with_context_src(|| format!("reading data file {}", path.display()))?;
        log::trace!("Reading data file bytes took {stopwatch}");

        let mut gm_data = self
            .parse(&raw_data)
            .with_context(|| format!("parsing GameMaker data file {}", path.display()))?;

        gm_data.meta.location = Some(path.to_path_buf());
        Ok(gm_data)
    }

    fn parse(&self, raw_data: &[u8]) -> Result<GMData> {
        if cfg!(feature = "catch-panic") {
            crate::util::panic::catch(|| parse(raw_data, self))
        } else {
            parse(raw_data, self)
        }
    }
}

/// Parses a GameMaker data file (stored in memory) with default options.
///
/// If you want to customize parsing options, check out [`ParsingOptions`].
/// If you want to parse a data file stored on disk, check out [`parse_file`].
///
/// For more information on the data file format, see [`crate::wad`].
pub fn parse_bytes(raw_data: impl AsRef<[u8]>) -> Result<GMData> {
    ParsingOptions::new().parse_bytes(raw_data)
}

/// Parses a GameMaker data file (stored on disk) with default options
///
/// If you want to customize parsing options, check out [`ParsingOptions`].
/// If you want to parse a data file stored in memory, check out
/// [`parse_bytes`].
///
/// For more information on the data file format, see [`crate::wad`].
pub fn parse_file(data_file_path: impl AsRef<Path>) -> Result<GMData> {
    ParsingOptions::new().parse_file(data_file_path)
}

// ================ Actual logic here ================

/// This can later be reused for audiogroup files.
fn parse_form(raw_data: &'_ [u8]) -> Result<DataReader<'_>> {
    // Length assertion
    if raw_data.len() >= i32::MAX as usize {
        bail!("{ERR_TOO_BIG}");
    }

    let mut reader = DataReader::new(raw_data);

    // Read root chunk and set endianness
    let root_chunk_name = reader.read_chunk_name().context("reading root chunk")?;
    reader.endianness = match root_chunk_name.as_str() {
        "FORM" => Endianness::Little,
        "MROF" => Endianness::Big,
        _ => bail!("Expected root chunk to be 'FORM' but found '{root_chunk_name}'"),
    };
    if reader.endianness == Endianness::Big {
        log::warn!("Big endian format might not work, proceed with caution");
    }

    // Length assertion
    let remaining_data_len = reader.read_u32()?;
    let total_data_len = remaining_data_len + reader.cur_pos;
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

        // Skip `chunk_length` bytes; moving to the end of the chunk.
        // Additional checks for integer overflows.
        reader.cur_pos = reader
            .cur_pos
            .checked_add(chunk_length)
            .filter(|&pos| pos <= total_data_len)
            .ok_or_else(|| {
                format!(
                    "Chunk '{name}' out of bounds: specified length {chunk_length} would exceed \
                     total length {total_data_len}"
                )
            })?;

        let end_pos = reader.cur_pos;
        reader.last_chunk = name;

        let chunk_bounds = ChunkBounds { start_pos, end_pos };
        reader.chunks.push(name, chunk_bounds)?;
    }

    Ok(reader)
}

#[allow(clippy::too_many_lines)]
/// Parse GameMaker data
fn parse(raw_data: &[u8], options: &ParsingOptions) -> Result<GMData> {
    let stopwatch = Stopwatch::start();
    let mut reader: DataReader = parse_form(raw_data)?;
    reader.options = options.clone();

    // Properly initialize GEN8 version before reading chunks.
    reader.specified_version = reader.read_gen8_version()?;

    // The following chunk read order is required:
    // Required: GEN8 --> all others
    //
    // Then (in any order):
    // * [FUNC, VARI, STRG] --> CODE
    // * TPAG --> [BGND, EMBI, FONT, OPTN, SPRT]

    reader.string_chunk = reader
        .chunks
        .get("STRG")
        .ok_or("Chunk STRG does not exist")?;
    reader.general_info = reader.read_chunk()?;
    if !reader.general_info.exists {
        bail!("GEN8 chunk does not exist");
    }

    if reader.specified_version == GMVersion::GMS2 {
        let stopwatch = Stopwatch::start();
        detect_gamemaker_version(&mut reader).context("detecting GameMaker version")?;
        log::trace!("Detecting GameMaker Version took {stopwatch}");
    }

    log::info!(
        "Loading {:?} (GM {}, WAD {})",
        reader.general_info.game_name,
        reader.general_info.version,
        reader.general_info.wad_version,
    );

    let texture_page_items: GMTexturePageItems = reader.read_chunk()?;

    let mut variables = GMVariables::default();
    let mut functions = GMFunctions::default();
    let mut codes = GMCodes::default();

    let is_yyc: bool = match check_yyc(&reader) {
        Ok(yyc) => yyc,
        Err(e) if reader.options.verify_constants => {
            log::warn!("YYC integrity check failed: {e}");
            false
        }
        Err(e) => return Err(e).context("Checking YYC"),
    };

    let stopwatch2 = if is_yyc {
        log::warn!("YYC is untested, issues may occur");
        // Need to remove STRG to not throw "unread chunk" error
        reader.chunks.remove("STRG");
        Stopwatch::start()
    } else {
        reader.read_chunk::<GMStrings>()?; // Sets `reader.strings`
        variables = reader.read_chunk()?; // Sets `reader.variable_occurrences`
        functions = reader.read_chunk()?; // Sets `reader.function_occurrences`
        let st = Stopwatch::start();
        codes = reader.read_chunk()?;
        st
    };

    // Read all other chunks. This is allowed to be executed arbitrary order.
    let texture_pages: GMTexturePages = reader.read_chunk()?;
    let scripts: GMScripts = reader.read_chunk()?;
    let fonts: GMFonts = reader.read_chunk()?;
    let sprites: GMSprites = reader.read_chunk()?;
    let game_objects: GMGameObjects = reader.read_chunk()?;
    let rooms: GMRooms = reader.read_chunk()?;
    let backgrounds: GMBackgrounds = reader.read_chunk()?;
    let audios: GMAudios = reader.read_chunk()?;
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
    let ui_nodes: GMRootUINodes = reader.read_chunk()?;
    let timelines: GMTimelines = reader.read_chunk()?;
    let embedded_images: GMEmbeddedImages = reader.read_chunk()?;
    let texture_group_infos: GMTextureGroupInfos = reader.read_chunk()?;
    let tags: GMTags = reader.read_chunk()?;
    let feature_flags: GMFeatureFlags = reader.read_chunk()?;
    let filter_effects: GMFilterEffects = reader.read_chunk()?;
    let animation_curves: GMAnimationCurves = reader.read_chunk()?;

    // This chunk is so useless that it is perfectly safe to throw it away.
    reader.read_chunk::<GMDataFiles>()?;

    log::trace!("Reading independent chunks took {stopwatch2}");

    if !options.exists {
        bail!("Required chunk OPTN does not exist");
    }

    handle_unread_chunks(&reader.chunks, reader.options.allow_unknown_chunks)?;

    let meta = Metadata {
        location: None,
        chunk_padding: reader.chunk_padding,
        endianness: reader.endianness,
        original_data_size: reader.size(),
    };

    let data = GMData {
        meta,

        animation_curves,
        audio_groups,
        audios,
        backgrounds,
        codes,
        embedded_images,
        extensions,
        feature_flags,
        filter_effects,
        fonts,
        functions,
        game_end_scripts,
        general_info: reader.general_info,
        global_init_scripts,
        language_info,
        options,
        particle_emitters,
        particle_systems,
        paths,
        rooms,
        ui_nodes,
        scripts,
        sequences,
        shaders,
        sounds,
        sprites,
        tags,
        texture_group_infos,
        texture_page_items,
        timelines,
        texture_pages,
        game_objects,
        variables,
    };

    log::trace!("Parsing data took {stopwatch}");
    Ok(data)
}

/// Verify all data chunks were processed to prevent data loss
fn handle_unread_chunks(chunks: &ChunkMap, allow_unknown: bool) -> Result<()> {
    if chunks.is_empty() {
        return Ok(());
    }

    let count: usize = chunks.count();

    let mut buffer = String::with_capacity(count * 6);
    for chunk_name in chunks.chunk_names() {
        buffer.push_str(chunk_name.as_str());
        buffer.push_str(", ");
    }

    // Remove last trailing comma and space
    buffer.pop();
    buffer.pop();

    let noun: &str = if count == 1 { "chunk" } else { "chunks" };

    let message = format!(
        "{count} unprocessed {noun} detected: {buffer}\nThese unknown chunks will be lost when \
         rebuilding data.",
    );

    if allow_unknown {
        log::warn!("{message}");
        Ok(())
    } else {
        bail!("{message}");
    }
}
