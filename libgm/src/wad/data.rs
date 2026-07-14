// SPDX-License-Identifier: GPL-3.0-only
//! The full GameMaker data struct, containing all information from a data file.
use std::path::PathBuf;

use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::wad::Blob;
use crate::wad::GMVersion;
use crate::wad::chunk::ChunkName;
use crate::wad::elem::animation_curve::AnimationCurves;
use crate::wad::elem::audio::Audios;
use crate::wad::elem::audio_group::AudioGroup;
use crate::wad::elem::audio_group::AudioGroups;
use crate::wad::elem::background::Tilesets;
use crate::wad::elem::code::Codes;
use crate::wad::elem::data_file::DataFiles;
use crate::wad::elem::embedded_image::EmbeddedImages;
use crate::wad::elem::extension::Extensions;
use crate::wad::elem::feature_flag::FeatureFlags;
use crate::wad::elem::filter_effect::FilterEffects;
use crate::wad::elem::font::Fonts;
use crate::wad::elem::function::Functions;
use crate::wad::elem::game_end::GameEndScripts;
use crate::wad::elem::game_object::GameObjects;
use crate::wad::elem::general_info::Flags;
use crate::wad::elem::general_info::FunctionClassifications;
use crate::wad::elem::general_info::GMS2Data;
use crate::wad::elem::general_info::GeneralInfo;
use crate::wad::elem::global_init::GlobalInitScripts;
use crate::wad::elem::language::LanguageInfo;
use crate::wad::elem::options::Constant;
use crate::wad::elem::options::OptionFlags;
use crate::wad::elem::options::Options;
use crate::wad::elem::particle_emitter::ParticleEmitters;
use crate::wad::elem::particle_system::ParticleSystems;
use crate::wad::elem::path::Paths;
use crate::wad::elem::room::Room;
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
use crate::wad::elem::timeline::Timelines;
use crate::wad::elem::ui_node::UINodes;
use crate::wad::elem::validate_names;
use crate::wad::elem::variable::ModernHeader;
use crate::wad::elem::variable::Variables;
use crate::wad::version::LtsBranch;

/// Byte order (endianness) for integers and chunk names in data files.
///
/// Most modern platforms use little-endian, which is the default.
/// Big-endian support exists for legacy platforms and **may be deprecated**.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Endianness {
    /// Little-endian byte order (reversed bytes).
    ///
    /// This is the standard for x86/x64 architectures and most modern
    /// platforms. All new projects should use this format.
    #[default]
    Little,

    /// Big-endian byte order (forward bytes).
    ///
    /// Supported for legacy compatibility with older platforms like PlayStation
    /// 3. This format is not thoroughly tested and may be removed in future
    /// versions.
    Big,
}

/// Some metadata about a [`GMData`] (GameMaker data file).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Metadata {
    /// The directory in which this data file is located.
    ///
    /// This can be used to find, read and edit the following:
    /// * Audio group files (e.g. `audiogroup1.dat`)
    /// * External sound files (e.g. `mus_st_him.ogg`)
    /// * JSON language files
    ///
    /// If you do not want these files to be available,
    /// you set this to `None` after parsing.
    pub location: Option<PathBuf>,

    /// Indicates the number of padding bytes (null bytes) between chunks.
    ///
    /// Note that the last chunk does not get padding.
    /// This padding is influenced by the data file's GameMaker Version, as well
    /// as target platform/architecture.
    pub chunk_padding: u32,

    /// Indicates the data's byte endianness.
    ///
    /// This affects byte order of integers and chunk names.
    /// In most cases (and assumed by default), this is set to little-endian.
    /// Big-endian is an edge case for certain target platforms (e.g. PS3 or
    /// Xbox 360) and its support may be removed in the future.
    pub endianness: Endianness,

    /// The size of the original data file; useful for
    /// approximating the size of the modified data file.
    ///
    /// This is a micro optimization. This field's value
    /// can be initialized to zero without any problems.
    pub original_data_size: u32,

    pub(crate) chunk_order: Vec<ChunkName>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            location: None,
            // Use 16 chunk padding by default for compatibility.
            chunk_padding: 16,
            endianness: Endianness::Little,
            original_data_size: 0,
            chunk_order: Vec::new(),
        }
    }
}

/// The full GameMaker data struct, containing all information from a data file.
#[derive(Clone, Debug)]
pub struct GMData {
    /// Some metadata about the GameMaker data file.
    ///
    /// This is purposefully stored in another struct to make it distinct
    /// from chunk elements in `GMData` and not clutter the namespace.
    pub meta: Metadata,

    pub animation_curves: AnimationCurves,      // ACRV
    pub audio_groups: AudioGroups,              // AGRP
    pub audios: Audios,                         // AUDO
    pub codes: Codes,                           // CODE
    pub data_files: DataFiles,                  // DAFL
    pub embedded_images: EmbeddedImages,        // EMBI
    pub extensions: Extensions,                 // EXTN
    pub feature_flags: FeatureFlags,            // FEAT
    pub filter_effects: FilterEffects,          // FEDS
    pub fonts: Fonts,                           // FONT
    pub functions: Functions,                   // FUNC
    pub game_end_scripts: GameEndScripts,       // GMEN
    pub game_objects: GameObjects,              // OBJT
    pub general_info: GeneralInfo,              // GEN8
    pub global_init_scripts: GlobalInitScripts, // GLOB
    pub language_info: LanguageInfo,            // LANG
    pub options: Options,                       // OPTN
    pub particle_emitters: ParticleEmitters,    // PSEM
    pub particle_systems: ParticleSystems,      // PSYS
    pub paths: Paths,                           // PATH
    pub rooms: Rooms,                           // ROOM
    pub ui_nodes: UINodes,                      // UILR
    pub scripts: Scripts,                       // SCPT
    pub sequences: Sequences,                   // SEQN
    pub shaders: Shaders,                       // SHDR
    pub sounds: Sounds,                         // SOND
    pub sprites: Sprites,                       // SPRT
    pub strings: Strings,                       // STRG
    pub tags: Tags,                             // TAGS
    pub texture_group_infos: TextureGroupInfos, // TGIN
    pub texture_page_items: TexturePageItems,   // TPAG
    pub texture_pages: TexturePages,            // TXTR
    pub tilesets: Tilesets,                     // BGND
    pub timelines: Timelines,                   // TMLN
    pub variables: Variables,                   // VARI
}

impl Default for GMData {
    fn default() -> Self {
        use ChunkName::*;

        let mut strings = Strings::default();
        strings.exists = true;

        let meta = Metadata {
            location: None,
            chunk_padding: 16,
            endianness: Endianness::Little,
            original_data_size: 5000,
            chunk_order: vec![
                GEN8, OPTN, LANG, EXTN, SOND, AGRP, SPRT, BGND, PATH, SCPT, GLOB, SHDR, FONT, TMLN,
                OBJT, FEDS, ACRV, SEQN, TAGS, ROOM, UILR, DAFL, EMBI, PSEM, PSYS, TPAG, TGIN, CODE,
                VARI, FUNC, FEAT, STRG, TXTR, AUDO,
            ],
        };

        let animation_curves = AnimationCurves { elems: Vec::new(), exists: true };
        let mut audio_groups = AudioGroups { elems: Vec::new(), exists: true };
        audio_groups.push(AudioGroup {
            name: strings.make("audiogroup_default"),
            path: strings.make("audiogroup_default.dat"),
        });
        let audios = Audios { elems: Vec::new(), exists: true };
        let codes = Codes { elems: Vec::new(), exists: true };
        let data_files = DataFiles { exists: true };
        let embedded_images = EmbeddedImages { elems: Vec::new(), exists: true };
        let extensions = Extensions { elems: Vec::new(), exists: true };
        let feature_flags = FeatureFlags { elems: Vec::new(), exists: true };
        let filter_effects = FilterEffects { elems: Vec::new(), exists: true };
        let fonts = Fonts { exists: true, ..Default::default() };
        let functions = Functions { exists: true, ..Default::default() };
        let game_end_scripts = GameEndScripts { elems: Vec::new(), exists: true };
        let game_objects = GameObjects { elems: Vec::new(), exists: true };
        let general_info = GeneralInfo {
            debugger_enabled: false,
            wad_version: 17,
            unknown_value: 0,
            game_file_name: strings.make("libgm_game"),
            config: strings.make("Default"),
            last_object_id: 100_000,
            last_tile_id: 10_000_000,
            game_id: 1337,
            directplay_guid: Blob([0u8; 16]),
            game_name: strings.make("LibGM"),
            version: GMVersion::new(2026, 0, 0, 0, LtsBranch::PostLts),
            window_width: 640,
            window_height: 480,
            flags: Flags::SCALE | Flags::SHOW_CURSOR,
            license_crc32: 1337,
            license_md5: Blob(*b"GnuPublicLicense"),
            creation_timestamp: Default::default(), // set this urself if you want to lol
            display_name: strings.make("LibGM: The Game"),
            function_classifications: FunctionClassifications::empty(),
            steam_appid: 0,
            debugger_port: 0,
            room_order: vec![GMRef::new(0)],
            gms2_data: Some(GMS2Data::default()),
            exists: true,
        };
        let global_init_scripts = GlobalInitScripts { elems: Vec::new(), exists: true };
        let language_info = LanguageInfo {
            unknown1: 1,
            exists: true,
            ..Default::default()
        };
        let options = Options {
            is_new_format: true,
            flags: OptionFlags::SHOW_CURSOR
                | OptionFlags::VARIABLE_ERRORS
                | OptionFlags::LEGACY_JSON_PARSING
                | OptionFlags::LEGACY_NUMBER_CONVERSION
                | OptionFlags::LEGACY_OTHER_BEHAVIOR
                | OptionFlags::AUDIO_ERROR_BEHAVIOR
                | OptionFlags::ALLOW_INSTANCE_CHANGE,
            window_scale: 0,
            window_color: 0,
            color_depth: 0,
            resolution: 0,
            frequency: 0,
            vertex_sync: 0,
            priority: 0,
            back_image: GMRef::none(),
            front_image: GMRef::none(),
            load_image: GMRef::none(),
            load_alpha: 0,
            constants: vec![
                Constant::new("@@SleepMargin", "0", &mut strings),
                Constant::new("@@DrawColour", "4294967295", &mut strings),
                Constant::new("@@VersionMajor", "1", &mut strings),
                Constant::new("@@VersionMinor", "0", &mut strings),
                Constant::new("@@VersionBuild", "", &mut strings),
                Constant::new("@@VersionRevision", "0", &mut strings),
            ],
            exists: true,
        };
        let particle_emitters = ParticleEmitters { elems: Vec::new(), exists: true };
        let particle_systems = ParticleSystems { elems: Vec::new(), exists: true };
        let paths = Paths { elems: Vec::new(), exists: true };
        let mut rooms = Rooms { elems: Vec::new(), exists: true };
        rooms.push(Room {
            name: strings.make("room0"),
            ..Default::default()
        });
        let ui_nodes = UINodes { elems: Vec::new(), exists: true };
        let scripts = Scripts { elems: Vec::new(), exists: true };
        let sequences = Sequences { elems: Vec::new(), exists: true };
        let shaders = Shaders { elems: Vec::new(), exists: true };
        let sounds = Sounds { elems: Vec::new(), exists: true };
        let sprites = Sprites { elems: Vec::new(), exists: true };
        let tags = Tags { exists: true, ..Default::default() };
        let texture_group_infos = TextureGroupInfos { elems: Vec::new(), exists: true };
        let texture_page_items = TexturePageItems { elems: Vec::new(), exists: true };
        let texture_pages = TexturePages { elems: Vec::new(), exists: true };
        let tilesets = Tilesets {
            elems: Vec::new(),
            align: true,
            exists: true,
        };
        let timelines = Timelines { elems: Vec::new(), exists: true };
        let variables = Variables {
            elems: Vec::new(),
            modern_header: Some(ModernHeader {
                var_count1: 0,
                var_count2: 0,
                max_local_var_count: 0,
            }),
            exists: true,
        };

        GMData {
            meta,
            animation_curves,
            audio_groups,
            audios,
            codes,
            data_files,
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
        }
    }
}

impl GMData {
    /// Validates all names of all named root elements.
    /// This checks for duplicates as well as name charset.
    pub fn validate_names(&self) -> Result<()> {
        let stopwatch = Stopwatch::start();
        validate_names(&self.animation_curves, &self.strings)?;
        validate_names(&self.audio_groups, &self.strings)?;
        validate_names(&self.codes, &self.strings)?;
        validate_names(&self.filter_effects, &self.strings)?;
        validate_names(&self.fonts, &self.strings)?;
        validate_names(&self.functions, &self.strings)?;
        validate_names(&self.game_objects, &self.strings)?;
        validate_names(&self.particle_systems, &self.strings)?;
        validate_names(&self.paths, &self.strings)?;
        validate_names(&self.rooms, &self.strings)?;
        validate_names(&self.scripts, &self.strings)?;
        validate_names(&self.sequences, &self.strings)?;
        validate_names(&self.shaders, &self.strings)?;
        validate_names(&self.sounds, &self.strings)?;
        validate_names(&self.sprites, &self.strings)?;
        validate_names(&self.texture_group_infos, &self.strings)?;
        validate_names(&self.tilesets, &self.strings)?;
        validate_names(&self.timelines, &self.strings)?;
        self.variables.validate_names(&self.strings)?;
        log::trace!("Validating all names took {stopwatch}");
        Ok(())
    }

    /// Deserializes all embedded texture pages, turning their underlying image
    /// data into [`DynamicImage`].
    ///
    /// This single-threaded implementation may take quite a while.
    /// If you care about performance, I would recommend making a custom
    /// multithreaded implementation (perhaps using the `rayon` crate).
    ///
    /// [`DynamicImage`]: image::DynamicImage
    pub fn deserialize_all_textures(&mut self) -> Result<()> {
        let stopwatch = Stopwatch::start();

        for texture_page in self.texture_pages.elements_mut() {
            let Some(image) = &mut texture_page.image else {
                continue;
            };
            let error: &str = "deserializing all embedded texture pages";
            image.deserialize().ctx(error)?;
        }

        log::trace!(
            "Deserializing all {} texture pages took {}",
            self.texture_pages.len(),
            stopwatch,
        );
        Ok(())
    }

    /// Runs some actions to fully verify integrity and
    /// to prepare the data file for editing.
    pub fn post_deserialize(&mut self) -> Result<()> {
        self.validate_names()?;
        self.deserialize_all_textures()?;
        self.optimize_memory();
        Ok(())
    }
}
