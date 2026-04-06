//! The full GameMaker data struct, containing all information from a data file.

use std::fmt;
use std::path::PathBuf;

use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::wad::elements::animation_curve::GMAnimationCurves;
use crate::wad::elements::audio::GMAudios;
use crate::wad::elements::audio_group::GMAudioGroups;
use crate::wad::elements::background::GMBackgrounds;
use crate::wad::elements::code::GMCodes;
use crate::wad::elements::embedded_image::GMEmbeddedImages;
use crate::wad::elements::extension::GMExtensions;
use crate::wad::elements::feature_flag::GMFeatureFlags;
use crate::wad::elements::filter_effect::GMFilterEffects;
use crate::wad::elements::font::GMFonts;
use crate::wad::elements::function::GMFunctions;
use crate::wad::elements::game_end::GMGameEndScripts;
use crate::wad::elements::game_object::GMGameObjects;
use crate::wad::elements::general_info::GMGeneralInfo;
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
use crate::wad::elements::tag::GMTags;
use crate::wad::elements::texture_group_info::GMTextureGroupInfos;
use crate::wad::elements::texture_page::GMTexturePages;
use crate::wad::elements::texture_page_item::GMTexturePageItems;
use crate::wad::elements::timeline::GMTimelines;
use crate::wad::elements::ui_node::GMRootUINodes;
use crate::wad::elements::validate_names;
use crate::wad::elements::variable::GMVariables;

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
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            location: None,
            // Use 16 chunk padding by default for compatibility.
            chunk_padding: 16,
            endianness: Endianness::Little,
            original_data_size: 0,
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

    pub animation_curves: GMAnimationCurves,      // ACRV
    pub audio_groups: GMAudioGroups,              // AGRP
    pub audios: GMAudios,                         // AUDO
    pub backgrounds: GMBackgrounds,               // BGND
    pub codes: GMCodes,                           // CODE
    pub embedded_images: GMEmbeddedImages,        // EMBI
    pub extensions: GMExtensions,                 // EXTN
    pub feature_flags: GMFeatureFlags,            // FEAT
    pub filter_effects: GMFilterEffects,          // FEDS
    pub fonts: GMFonts,                           // FONT
    pub functions: GMFunctions,                   // FUNC
    pub game_end_scripts: GMGameEndScripts,       // GMEN
    pub game_objects: GMGameObjects,              // OBJT
    pub general_info: GMGeneralInfo,              // GEN8
    pub global_init_scripts: GMGlobalInitScripts, // GLOB
    pub language_info: GMLanguageInfo,            // LANG
    pub options: GMOptions,                       // OPTN
    pub particle_emitters: GMParticleEmitters,    // PSEM
    pub particle_systems: GMParticleSystems,      // PSYS
    pub paths: GMPaths,                           // PATH
    pub rooms: GMRooms,                           // ROOM
    pub ui_nodes: GMRootUINodes,                  // UILR
    pub scripts: GMScripts,                       // SCPT
    pub sequences: GMSequences,                   // SEQN
    pub shaders: GMShaders,                       // SHDR
    pub sounds: GMSounds,                         // SOND
    pub sprites: GMSprites,                       // SPRT
    pub tags: GMTags,                             // TAGS
    pub texture_group_infos: GMTextureGroupInfos, // TGIN
    pub texture_page_items: GMTexturePageItems,   // TPAG
    pub texture_pages: GMTexturePages,            // TXTR
    pub timelines: GMTimelines,                   // TMLN
    pub variables: GMVariables,                   // VARI
}

impl fmt::Debug for GMData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        validate_names(&self.animation_curves)?;
        validate_names(&self.audio_groups)?;
        validate_names(&self.backgrounds)?;
        validate_names(&self.codes)?;
        validate_names(&self.embedded_images)?;
        validate_names(&self.filter_effects)?;
        validate_names(&self.fonts)?;
        validate_names(&self.functions)?;
        validate_names(&self.game_objects)?;
        validate_names(&self.particle_emitters)?;
        validate_names(&self.particle_systems)?;
        validate_names(&self.paths)?;
        validate_names(&self.rooms)?;
        validate_names(&self.scripts)?;
        validate_names(&self.sequences)?;
        validate_names(&self.shaders)?;
        validate_names(&self.sounds)?;
        validate_names(&self.sprites)?;
        validate_names(&self.texture_group_infos)?;
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

        for texture_page in &mut self.texture_pages {
            let Some(image) = &mut texture_page.image else {
                continue;
            };
            let error: &str = "deserializing all embedded texture pages";
            image.deserialize().context(error)?;
        }

        log::trace!(
            "Deserializing all {} texture pages took {}",
            self.texture_pages.len(),
            stopwatch,
        );
        Ok(())
    }

    /// Runs some actions to fully verify integrity and to prepare the data file
    /// for editing.
    pub fn post_deserialize(&mut self) -> Result<()> {
        self.validate_names()?;
        self.deserialize_all_textures()?;
        self.optimize_memory();
        Ok(())
    }
}
