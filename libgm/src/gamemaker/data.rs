use crate::{
    gamemaker::{
        elements::{
            animation_curves::GMAnimationCurves,
            audio_groups::GMAudioGroups,
            backgrounds::GMBackgrounds,
            code::GMCodes,
            embedded_audio::GMEmbeddedAudios,
            embedded_images::GMEmbeddedImages,
            embedded_textures::GMEmbeddedTextures,
            extensions::GMExtensions,
            feature_flags::GMFeatureFlags,
            filter_effects::GMFilterEffects,
            fonts::GMFonts,
            functions::{GMFunction, GMFunctions},
            game_end::GMGameEndScripts,
            game_objects::GMGameObjects,
            general_info::GMGeneralInfo,
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
            tags::GMTags,
            texture_group_info::GMTextureGroupInfos,
            texture_page_items::GMTexturePageItems,
            timelines::GMTimelines,
            ui_nodes::GMRootUINodes,
            variables::GMVariables,
        },
        reference::GMRef,
    },
    prelude::*,
};

/// Byte order (endianness) for integers and chunk names in data files.
///
/// Most modern platforms use little-endian, which is the default.
/// Big-endian support exists for legacy platforms and may be deprecated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Endianness {
    /// Little-endian byte order (reversed bytes).
    ///
    /// This is the standard for x86/x64 architectures and most modern platforms.
    /// All new projects should use this format.
    #[default]
    Little,

    /// Big-endian byte order (forward bytes).
    ///
    /// Supported for legacy compatibility with older platforms like PlayStation 3.
    /// This format is not thoroughly tested and may be removed in future versions.
    Big,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMData {
    pub animation_curves: GMAnimationCurves,      // ACRV
    pub audio_groups: GMAudioGroups,              // AGRP
    pub audios: GMEmbeddedAudios,                 // AUDO
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
    pub root_ui_nodes: GMRootUINodes,             // UILR
    pub scripts: GMScripts,                       // SCPT
    pub sequences: GMSequences,                   // SEQN
    pub shaders: GMShaders,                       // SHDR
    pub sounds: GMSounds,                         // SOND
    pub sprites: GMSprites,                       // SPRT
    pub tags: GMTags,                             // TAGS
    pub texture_group_infos: GMTextureGroupInfos, // TGIN
    pub texture_page_items: GMTexturePageItems,   // TPAG
    pub timelines: GMTimelines,                   // TMLN
    pub embedded_textures: GMEmbeddedTextures,    // TXTR
    pub variables: GMVariables,                   // VARI

    /// Indicates the number of padding bytes (null bytes) between chunks.
    /// Note that the last chunk does not get padding.
    /// This padding is influenced by the data file's `GameMaker` Version, as well as target platform/architecture.
    pub chunk_padding: u32,

    /// Indicates the data's byte endianness.
    /// This affects byte order of integers and chunk names.
    /// In most cases (and assumed by default), this is set to little-endian.
    /// Big-endian is an edge case for certain target platforms (e.g. PS3 or Xbox 360)
    /// and its support may be removed in the future.
    pub endianness: Endianness,

    /// The size of the original data file; useful for
    /// approximating the size of the modified data file.
    /// This is a micro optimisation. This field's value
    /// can be initialized to zero without any problems.
    pub original_data_size: u32,
}

impl Default for GMData {
    fn default() -> Self {
        Self {
            animation_curves: GMAnimationCurves::default(),
            audio_groups: GMAudioGroups::default(),
            audios: GMEmbeddedAudios::default(),
            backgrounds: GMBackgrounds::default(),
            codes: GMCodes::default(),
            embedded_images: GMEmbeddedImages::default(),
            extensions: GMExtensions::default(),
            feature_flags: GMFeatureFlags::default(),
            filter_effects: GMFilterEffects::default(),
            fonts: GMFonts::default(),
            functions: GMFunctions::default(),
            game_end_scripts: GMGameEndScripts::default(),
            game_objects: GMGameObjects::default(),
            general_info: GMGeneralInfo::default(),
            global_init_scripts: GMGlobalInitScripts::default(),
            language_info: GMLanguageInfo::default(),
            options: GMOptions::default(),
            particle_emitters: GMParticleEmitters::default(),
            particle_systems: GMParticleSystems::default(),
            paths: GMPaths::default(),
            rooms: GMRooms::default(),
            root_ui_nodes: GMRootUINodes::default(),
            scripts: GMScripts::default(),
            sequences: GMSequences::default(),
            shaders: GMShaders::default(),
            sounds: GMSounds::default(),
            sprites: GMSprites::default(),
            tags: GMTags::default(),
            texture_group_infos: GMTextureGroupInfos::default(),
            texture_page_items: GMTexturePageItems::default(),
            timelines: GMTimelines::default(),
            embedded_textures: GMEmbeddedTextures::default(),
            variables: GMVariables::default(),

            // Use 16 chunk padding by default for compatibility.
            chunk_padding: 16,
            endianness: Endianness::Little,
            original_data_size: 0,
        }
    }
}

impl GMData {
    fn find_function(&self, name: &str) -> Result<Option<GMRef<GMFunction>>> {
        for (i, function) in self.functions.iter().enumerate() {
            if name == function.name {
                return Ok(Some(GMRef::new(i as u32)));
            }
        }
        Ok(None)
    }

    pub fn function_by_name(&self, name: &str) -> Result<GMRef<GMFunction>> {
        let function = self
            .find_function(name)?
            .ok_or_else(|| format!("Could not find function with name {name:?}"))?;
        Ok(function)
    }
}
