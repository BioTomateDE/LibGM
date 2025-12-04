pub(crate) mod chunk;
mod lists;
mod numbers;
pub(crate) mod reader;
pub mod resources;

use std::path::Path;

use crate::{
    gamemaker::{
        data::{Endianness, GMData},
        deserialize::{
            chunk::{Chunks, GMChunk},
            reader::DataReader,
        },
        elements::{
            animation_curves::GMAnimationCurves,
            audio_groups::GMAudioGroups,
            backgrounds::GMBackgrounds,
            code::{GMCodes, check_yyc},
            data_files::GMDataFiles,
            embedded_audio::GMEmbeddedAudios,
            embedded_images::GMEmbeddedImages,
            embedded_textures::GMEmbeddedTextures,
            extensions::GMExtensions,
            feature_flags::GMFeatureFlags,
            filter_effects::GMFilterEffects,
            fonts::GMFonts,
            functions::GMFunctions,
            game_end::GMGameEndScripts,
            game_objects::GMGameObjects,
            global_init::GMGlobalInitScripts,
            languages::GMLanguageInfo,
            options::GMOptions,
            particle_emitters::GMParticleEmitters,
            particle_systems::GMParticleSystems,
            paths::GMPaths,
            rooms::GMRooms,
            scripts::GMScripts,
            sequence::GMSequences,
            shaders::GMShaders,
            sounds::GMSounds,
            sprites::GMSprites,
            strings::GMStrings,
            tags::GMTags,
            texture_group_info::GMTextureGroupInfos,
            texture_page_items::GMTexturePageItems,
            timelines::GMTimelines,
            ui_nodes::GMRootUINodes,
            variables::GMVariables,
        },
        gm_version::{GMVersion, LTSBranch},
        version_detection::detect_gamemaker_version,
    },
    prelude::*,
    util::bench::Stopwatch,
};

const ERR_TOO_BIG: &str =
    "Data file is bigger than 2,147,483,646 bytes which will lead to bugs in LibGM and the runner";

pub struct DataParser {
    options: ParserOptions,
}

pub(crate) struct ParserOptions {
    pub verify_alignment: bool,
    pub verify_constants: bool,
    pub allow_unknown_chunks: bool,
    pub parallel_processing: bool,
}

impl Default for DataParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DataParser {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            options: ParserOptions {
                verify_alignment: true,
                verify_constants: true,
                allow_unknown_chunks: false,
                parallel_processing: false,
            },
        }
    }

    /// When enabled, verifies that all pointer offsets and data structures
    /// are properly aligned according to their type requirements.
    ///
    /// This helps detect corrupted or malformed data files by ensuring
    /// all memory accesses occur on correct boundaries.
    ///
    /// Disable this flag if your data file has uncommon/malformed alignment
    /// (e.g. a port to a niche platform), but you want to try to parse it anyway.
    ///
    /// > Default: **true**
    #[must_use]
    pub const fn verify_alignment(mut self, enabled: bool) -> Self {
        self.options.verify_alignment = enabled;
        self
    }

    /// When enabled, validates that known constant values in the data format
    /// match their expected values (e.g., reserved fields that should be zero
    /// or deprecated values that are always the same for compatibility).
    ///
    /// This provides additional validation against data corruption or version mismatches.
    ///
    /// > Default: **true**
    #[must_use]
    pub const fn verify_constants(mut self, enabled: bool) -> Self {
        self.options.verify_constants = enabled;
        self
    }

    /// When **disabled**, requires that all data chunks in the file are processed during parsing.
    ///
    /// This prevents silent data loss when rebuilding data by ensuring
    /// no unrecognized or unsupported chunks are ignored. If any chunks
    /// remain unread after parsing completes, an error will be returned.
    ///
    /// > Default: **false**
    #[must_use]
    pub const fn allow_unknown_chunks(mut self, enabled: bool) -> Self {
        self.options.allow_unknown_chunks = enabled;
        self
    }

    /// When enabled, processes independent chunks in parallel using multiple threads.
    /// # Experimental.
    ///
    /// > Default: **false**
    #[must_use]
    pub const unsafe fn parallel_processing(mut self, enabled: bool) -> Self {
        self.options.parallel_processing = enabled;
        self
    }

    fn parse(&self, raw_data: impl AsRef<[u8]>) -> Result<GMData> {
        let stopwatch = Stopwatch::start();
        let mut reader: DataReader = parse_form(raw_data.as_ref())?;

        // Properly initialize GEN8 version before reading chunks.
        reader.specified_version = reader.read_gen8_version()?;

        // The following chunk read order is required:
        // Required: GEN8 --> all others
        //
        // Then (in any order):
        // • [FUNC, VARI, STRG] --> CODE
        // • TPAG --> [BGND, EMBI, FONT, OPTN, SPRT]

        reader.string_chunk = reader
            .chunks
            .get("STRG")
            .ok_or("Chunk STRG does not exist")?;
        reader.general_info = reader.read_chunk()?;
        if !reader.general_info.exists {
            bail!("GEN8 chunk does not exist");
        }

        #[allow(clippy::items_after_statements)]
        const GMS2: GMVersion = GMVersion::new(2, 0, 0, 0, LTSBranch::PreLTS);

        if reader.specified_version == GMS2 {
            let stopwatch = Stopwatch::start();
            detect_gamemaker_version(&mut reader).context("detecting GameMaker version")?;
            log::trace!("Detecting GameMaker Version took {stopwatch}");
        }

        log::info!(
            "Loading {:?} (GM {}, Bytecode {})",
            reader.general_info.display_name,
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
            reader.read_chunk::<GMStrings>()?; // Set `reader.strings`
            variables = reader.read_chunk()?; // Set `reader.variable_occurrences`
            functions = reader.read_chunk()?; // Set `reader.function_occurrences`
            stopwatch2 = Stopwatch::start();
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

        // This chunk is so useless that it is perfectly safe to throw it away.
        reader.read_chunk::<GMDataFiles>()?;

        log::trace!("Reading independent chunks took {stopwatch2}");

        if !options.exists {
            bail!("Required chunk OPTN does not exist");
        }

        handle_unread_chunks(&reader.chunks, self.options.allow_unknown_chunks)?;

        let data = GMData {
            chunk_padding: reader.chunk_padding,
            endianness: reader.endianness,
            original_data_size: reader.size(),

            general_info: reader.general_info,

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
            global_init_scripts,
            language_info,
            options,
            particle_emitters,
            particle_systems,
            paths,
            rooms,
            root_ui_nodes,
            scripts,
            sequences,
            shaders,
            sounds,
            sprites,
            tags,
            texture_group_infos,
            texture_page_items,
            timelines,
            embedded_textures,
            game_objects,
            variables,
        };

        log::trace!("Parsing data took {stopwatch}");
        Ok(data)
    }

    ///todo docstrings
    pub fn parse_bytes(&self, raw_data: impl AsRef<[u8]>) -> Result<GMData> {
        self.parse(raw_data).context("parsing GameMaker data")
    }

    /// Parse a GameMaker data file (`data.win`, `game.unx`, etc).
    pub fn parse_file(&self, data_file_path: impl AsRef<Path>) -> Result<GMData> {
        let path = data_file_path.as_ref();

        let meta = std::fs::metadata(path)
            .map_err(|e| e.to_string())
            .with_context(|| format!("reading metadata of data file {}", path.display()))?;

        if meta.len() >= i32::MAX as u64 {
            bail!("{ERR_TOO_BIG}");
        }

        let stopwatch = Stopwatch::start();
        let raw_data: Vec<u8> = std::fs::read(path)
            .map_err(|e| e.to_string())
            .with_context(|| format!("reading data file {}", path.display()))?;
        log::trace!("Reading data file took {stopwatch}");

        self.parse(raw_data)
            .with_context(|| format!("parsing GameMaker data file {}", path.display()))
    }
}

fn parse_form(raw_data: &'_ [u8]) -> Result<DataReader<'_>> {
    // Length assertion
    if raw_data.len() >= i32::MAX as usize {
        bail!("{ERR_TOO_BIG}");
    }

    let mut reader = DataReader::new(raw_data);

    // Read root chunk and set endianness
    let root_chunk_name = reader.read_chunk_name()?;
    reader.endianness = match root_chunk_name.as_str() {
        "FORM" => Endianness::Little,
        "MROF" => Endianness::Big,
        _ => bail!(
            "Invalid data file: expected root chunk to be 'FORM' but found '{root_chunk_name}'"
        ),
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

        reader.cur_pos = reader.cur_pos.checked_add(chunk_length)
            .filter(|&pos| pos <= total_data_len)
            .ok_or_else(|| format!(
                "Chunk '{name}' out of bounds: specified length {chunk_length} would exceed total length {total_data_len}"
            ))?;

        let end_pos = reader.cur_pos;
        if end_pos == total_data_len {
            reader.last_chunk.clone_from(&name);
        }

        let chunk = GMChunk { start_pos, end_pos };
        reader.chunks.push(name, chunk)?;
    }

    Ok(reader)
}

/// Verify all data chunks were processed to prevent data loss
fn handle_unread_chunks(chunks: &Chunks, allow_unknown_chunks: bool) -> Result<()> {
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
        "{count} unprocessed {noun} detected: {buffer}\n\
        These unknown chunks will be lost when rebuilding data.",
    );

    if allow_unknown_chunks {
        log::warn!("{message}");
        Ok(())
    } else {
        bail!("{message}");
    }
}

/// Parse a GameMaker data file (stored in a buffer) with default settings.
pub fn read_data_bytes(raw_data: impl AsRef<[u8]>) -> Result<GMData> {
    DataParser::new().parse_bytes(raw_data)
}

/// Parse a GameMaker data file (`data.win`, `game.unx`, etc.) with default settings.
pub fn read_data_file(data_file_path: impl AsRef<Path>) -> Result<GMData> {
    DataParser::new().parse_file(data_file_path)
}
