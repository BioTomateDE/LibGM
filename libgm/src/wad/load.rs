// SPDX-License-Identifier: GPL-3.0-only
//! Functions and writer structs related to parsing GameMaker data files.
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
use crate::util::unwind::catch;
use crate::wad::chunk::ChunkName;
use crate::wad::data::Endianness;
use crate::wad::data::GMData;
use crate::wad::data::Metadata;
use crate::wad::elem::animation_curve::AnimationCurves;
use crate::wad::elem::audio::Audios;
use crate::wad::elem::audio_group::AudioGroups;
use crate::wad::elem::code::Codes;
use crate::wad::elem::code::check_yyc;
use crate::wad::elem::data_file::DataFiles;
use crate::wad::elem::embedded_image::EmbeddedImages;
use crate::wad::elem::extension::Extensions;
use crate::wad::elem::feature_flag::FeatureFlags;
use crate::wad::elem::filter_effect::FilterEffects;
use crate::wad::elem::font::Fonts;
use crate::wad::elem::function::Functions;
use crate::wad::elem::game_end::GameEndScripts;
use crate::wad::elem::game_object::GameObjects;
use crate::wad::elem::general_info::GeneralInfo;
use crate::wad::elem::global_init::GlobalInitScripts;
use crate::wad::elem::language::LanguageInfo;
use crate::wad::elem::options::Options;
use crate::wad::elem::particle_emitter::ParticleEmitters;
use crate::wad::elem::particle_system::ParticleSystems;
use crate::wad::elem::path::Paths;
use crate::wad::elem::room::Rooms;
use crate::wad::elem::script::Scripts;
use crate::wad::elem::sequence::Sequences;
use crate::wad::elem::shader::Shaders;
use crate::wad::elem::sound::Sounds;
use crate::wad::elem::sprite::Sprites;
use crate::wad::elem::string::Strings;
use crate::wad::elem::tag::Tags;
use crate::wad::elem::texture_group_info::TextureGroupInfos;
use crate::wad::elem::texture_page::TexturePages;
use crate::wad::elem::texture_page_item::TexturePageItems;
use crate::wad::elem::tileset::Tilesets;
use crate::wad::elem::timeline::Timelines;
use crate::wad::elem::ui_node::UINodes;
use crate::wad::elem::variable::Variables;
use crate::wad::load::chunk::ChunkBounds;
use crate::wad::load::reader::DataReader;
use crate::wad::version_detection::detect_format_version;

const ERR_MASSIVE: &str = "Data file size exceeds the signed 32-bit integer limit which will lead \
                           to bugs in LibGM and the runner";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadingOptions {
    /// See [`LoadingOptions::verify_alignment`].
    pub verify_alignment: bool,

    /// See [`LoadingOptions::verify_constants`].
    pub verify_constants: bool,
}

#[deprecated(since = "0.8.0", note = "renamed to LoadingOptions")]
pub type ParsingOptions = LoadingOptions;

impl Default for LoadingOptions {
    /// Strict by default
    fn default() -> Self {
        Self::STRICT
    }
}

impl LoadingOptions {
    pub const LENIENT: Self = Self { verify_alignment: false, verify_constants: false };
    pub const STRICT: Self = Self { verify_alignment: true, verify_constants: true };

    /// Creates a new [`LoadingOptions`] with default settings
    /// (corresponds to [`LoadingOptions::STRICT`].
    #[inline]
    #[must_use]
    #[deprecated(since = "0.8.0", note = "use LoadingOptions::default instead")]
    pub const fn new() -> Self {
        Self::STRICT
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

    /// Deserializes a GameMaker data file (stored in memory) with the specified options.
    ///
    /// If you want to deserialize a data file stored on disk, check out [`LoadingOptions::load_file`].
    ///
    /// For more information on the data file format, see [`crate::wad`].
    pub fn load_bytes(&self, raw_data: impl AsRef<[u8]>) -> Result<GMData> {
        let raw_data = raw_data.as_ref();
        catch(|| deserialize(raw_data, self)).ctx("parsing GameMaker data bytes")
    }

    #[deprecated(since = "0.8.0", note = "renamed to load_bytes")]
    pub fn parse_bytes(&self, raw_data: impl AsRef<[u8]>) -> Result<GMData> {
        self.load_bytes(raw_data)
    }

    /// Deseerializes a GameMaker data file (stored on disk) with the specified options.
    ///
    /// If you want to deserialize a data file stored in memory, check out [`LoadingOptions::load_bytes`].
    ///
    /// For more information on the data file format, see [`crate::wad`].
    pub fn load_file(&self, data_file_path: impl AsRef<Path>) -> Result<GMData> {
        let path = data_file_path.as_ref();

        let meta = std::fs::metadata(path)
            .ctx_any(|| format!("reading metadata of data file {}", path.display()))?;

        if meta.len() > i32::MAX as u64 {
            bail!("{ERR_MASSIVE}");
        }

        let stopwatch = Stopwatch::start();
        let raw_data: Vec<u8> =
            std::fs::read(path).ctx_any(|| format!("reading data file {}", path.display()))?;
        log::trace!("Reading data file bytes took {stopwatch}");

        let mut gm_data = catch(|| deserialize(&raw_data, self))
            .ctx(|| format!("deserializing GameMaker data file {}", path.display()))?;

        gm_data.meta.location = Some(path.to_path_buf());
        Ok(gm_data)
    }

    #[deprecated(since = "0.8.0", note = "renamed to load_file")]
    pub fn parse_file(&self, data_file_path: impl AsRef<Path>) -> Result<GMData> {
        self.load_file(data_file_path)
    }

    /// Deserializes a GameMaker audio group file (stored in memory) with the specified options.
    pub fn load_audiogroup_bytes(&self, raw_data: impl AsRef<[u8]>) -> Result<Audios> {
        let mut reader: DataReader = read_form(raw_data.as_ref(), self.clone())?;
        reader.read_chunk()
    }
}

/// Deserializes a GameMaker data file (stored in memory) with default options.
///
/// If you want to customize parsing options, check out [`LoadingOptions`].
/// If you want to parse a data file stored on disk, check out [`load_file`].
///
/// For more information on the data file format, see [`crate::wad`].
pub fn load_bytes(raw_data: impl AsRef<[u8]>) -> Result<GMData> {
    LoadingOptions::default().load_bytes(raw_data)
}

#[deprecated(since = "0.8.0", note = "renamed to load_bytes")]
pub fn parse_bytes(raw_data: impl AsRef<[u8]>) -> Result<GMData> {
    load_bytes(raw_data)
}

/// Deserializes a GameMaker data file (stored on disk) with default options.
///
/// If you want to customize parsing options or load a data file
/// stored in memory, check out [`LoadingOptions::load_bytes`].
pub fn load_file(data_file_path: impl AsRef<Path>) -> Result<GMData> {
    LoadingOptions::default().load_file(data_file_path)
}

#[deprecated(since = "0.8.0", note = "renamed to load_file")]
pub fn parse_file(data_file_path: impl AsRef<Path>) -> Result<GMData> {
    load_file(data_file_path)
}

/// Deserializes a GameMaker audio group file (stored on disk) with default options.
///
/// If you want to customize parsing options or load a audio
/// group data stored in memory, check out [`LoadingOptions::load_audiogroup_bytes`].
pub fn load_audiogroup_file(path: impl AsRef<Path>) -> Result<Audios> {
    let path = path.as_ref();
    let stopwatch = Stopwatch::start();
    let raw_data: Vec<u8> =
        std::fs::read(path).ctx_any(|| format!("reading audio group file {}", path.display()))?;
    log::trace!("Reading audio group file bytes took {stopwatch}");
    LoadingOptions::default().load_audiogroup_bytes(raw_data)
}

// ================ Actual logic here ================

fn read_form(raw_data: &'_ [u8], options: LoadingOptions) -> Result<DataReader<'_>> {
    let root = raw_data.get(0..4);
    let endianness = match root {
        Some(b"FORM") => Endianness::Little,
        Some(b"MROF") => Endianness::Big,
        _ => bail!("This is not a GameMaker data file"),
    };

    // Length assertion
    if raw_data.len() >= i32::MAX as usize {
        bail!("{ERR_MASSIVE}");
    }

    let mut reader = DataReader::new(raw_data);
    reader.options = options;
    reader.endianness = endianness;
    reader.cur_pos = 4;

    if endianness == Endianness::Big {
        log::warn!("Big endian format might not work, proceed with caution");
    }

    // Length assertion
    let remaining_data_len = reader.read_u32().ctx("reading total data length")?;
    let total_data_len = remaining_data_len + reader.cur_pos /* 8 */;
    if total_data_len as usize != raw_data.len() {
        bail!(
            "Specified FORM data length is {} but data is actually {} bytes long",
            total_data_len,
            raw_data.len(),
        );
    }

    // Read chunks into HashMap (FORM)
    while reader.cur_pos + 8 < total_data_len {
        let mut known = true;

        let mut bytes: [u8; 4] = *reader.read_bytes_const()?;
        if reader.endianness == Endianness::Big {
            bytes.reverse();
        }
        let name = match ChunkName::from_bytes(bytes) {
            Ok(c) => c,
            Err(e) if reader.options.verify_constants => return Err(e),
            Err(e) => {
                log::warn!("{e}");
                known = false;
                ChunkName::DAFL //stub
            }
        };

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
                    "Chunk {name} out of bounds: specified length {chunk_length} would exceed \
                     total length {total_data_len}"
                )
            })?;

        let end_pos = reader.cur_pos;
        reader.last_chunk = name;

        if known {
            let chunk_bounds = ChunkBounds { start_pos, end_pos };
            reader.chunks.push(name, chunk_bounds)?;
            reader.chunk_order.push(name);
        }
    }

    Ok(reader)
}

fn read_bytecode_chunks(reader: &mut DataReader) -> Result<(Codes, Functions, Variables)> {
    let is_yyc: bool = match check_yyc(reader) {
        Ok(yyc) => yyc,
        Err(e) if reader.options.verify_constants => return Err(e).ctx("Checking YYC"),
        Err(e) => {
            log::warn!("YYC integrity check failed: {e}");
            false
        }
    };

    if is_yyc {
        log::warn!("YYC is untested, issues may occur");
        Ok((Codes::default(), Functions::default(), Variables::default()))
    } else {
        let variables = reader.read_chunk()?; // Sets `reader.variable_occurrences`
        let functions = reader.read_chunk()?; // Sets `reader.function_occurrences`
        let codes = reader.read_chunk()?;
        Ok((codes, functions, variables))
    }
}

#[expect(clippy::too_many_lines)]
fn deserialize(raw_data: &[u8], options: &LoadingOptions) -> Result<GMData> {
    let stopwatch = Stopwatch::start();
    let mut reader: DataReader = read_form(raw_data, options.clone()).ctx("parsing FORM")?;

    if !reader.chunks.contains(ChunkName::STRG) {
        bail!("Chunk STRG does not exist");
    }
    if !reader.chunks.contains(ChunkName::GEN8) {
        bail!("Chunk GEN8 does not exist");
    }
    if !reader.chunks.contains(ChunkName::OPTN) {
        bail!("Chunk OPTN does not exist");
    }

    // Set the STRG chunk so that the reader can force-read strings.
    reader.string_chunk = reader.chunks.get(ChunkName::STRG).ok_or("Chunk STRG does not exist")?;

    let stopwatch2 = Stopwatch::start();
    reader.version = detect_format_version(reader.clone()).ctx("detecting format version")?;
    log::debug!("Detecting format version took {:.2?}", stopwatch2.elapsed());

    // The following chunk read order is required:
    // Required: STRG --> most others
    // Then (in any order):
    // * [FUNC, VARI] --> CODE
    // * TPAG --> [BGND, EMBI, FONT, OPTN, SPRT]

    let strings: Strings = reader.read_chunk()?;
    let texture_page_items: TexturePageItems = reader.read_chunk()?;

    let codes: Codes;
    let functions: Functions;
    let variables: Variables;
    (codes, functions, variables) = read_bytecode_chunks(&mut reader)?;

    // Read all other chunks. This is allowed to be executed arbitrary order.
    let stopwatch2 = Stopwatch::start();

    let texture_pages: TexturePages = reader.read_chunk()?;
    let scripts: Scripts = reader.read_chunk()?;
    let fonts: Fonts = reader.read_chunk()?;
    let sprites: Sprites = reader.read_chunk()?;
    let game_objects: GameObjects = reader.read_chunk()?;
    let general_info: GeneralInfo = reader.read_chunk()?;
    let rooms: Rooms = reader.read_chunk()?;
    let tilesets: Tilesets = reader.read_chunk()?;
    let audios: Audios = reader.read_chunk()?;
    let sounds: Sounds = reader.read_chunk()?;
    let paths: Paths = reader.read_chunk()?;
    let options: Options = reader.read_chunk()?;
    let sequences: Sequences = reader.read_chunk()?;
    let particle_systems: ParticleSystems = reader.read_chunk()?;
    let particle_emitters: ParticleEmitters = reader.read_chunk()?;
    let language_info: LanguageInfo = reader.read_chunk()?;
    let extensions: Extensions = reader.read_chunk()?;
    let audio_groups: AudioGroups = reader.read_chunk()?;
    let global_init_scripts: GlobalInitScripts = reader.read_chunk()?;
    let game_end_scripts: GameEndScripts = reader.read_chunk()?;
    let shaders: Shaders = reader.read_chunk()?;
    let ui_nodes: UINodes = reader.read_chunk()?;
    let timelines: Timelines = reader.read_chunk()?;
    let embedded_images: EmbeddedImages = reader.read_chunk()?;
    let texture_group_infos: TextureGroupInfos = reader.read_chunk()?;
    let tags: Tags = reader.read_chunk()?;
    let feature_flags: FeatureFlags = reader.read_chunk()?;
    let filter_effects: FilterEffects = reader.read_chunk()?;
    let animation_curves: AnimationCurves = reader.read_chunk()?;
    reader.read_chunk::<DataFiles>()?;

    log::trace!("Reading independent chunks took {stopwatch2}");

    let meta = Metadata {
        version: reader.version,
        location: None,
        chunk_padding: reader.chunk_padding,
        endianness: reader.endianness,
        original_data_size: reader.size(),
        chunks: reader.chunk_order,
    };

    let data = GMData {
        meta,
        animation_curves,
        audio_groups,
        audios,
        codes,
        embedded_images,
        extensions,
        feature_flags,
        filter_effects,
        fonts,
        functions,
        game_end_scripts,
        game_objects,
        general_info,
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
        strings,
        tags,
        texture_group_infos,
        texture_page_items,
        texture_pages,
        tilesets,
        timelines,
        variables,
    };

    log::trace!("Parsing data took {stopwatch}");
    Ok(data)
}
