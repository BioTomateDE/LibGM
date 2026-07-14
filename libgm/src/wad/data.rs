// SPDX-License-Identifier: GPL-3.0-only
//! The full GameMaker data struct, containing all information from a data file.

use std::fmt;
use std::path::PathBuf;

use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::wad::chunk::ChunkName;
use crate::wad::elem::animation_curve::AnimationCurves;
use crate::wad::elem::audio::Audios;
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
use crate::wad::elem::timeline::Timelines;
use crate::wad::elem::ui_node::UINodes;
use crate::wad::elem::validate_names;
use crate::wad::elem::variable::Variables;

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
#[derive(Clone, Default)]
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

impl fmt::Debug for GMData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GMData")
            .field("meta", &self.meta)
            .field("general_info", &self.general_info)
            .finish_non_exhaustive()
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
