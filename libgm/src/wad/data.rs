//! The full GameMaker data struct, containing all information from a data file.

use std::{collections::HashMap, hash::Hash, path::PathBuf};

use crate::{
    prelude::*,
    util::fmt::format_bytes,
    wad::elements::{
        animation_curve::GMAnimationCurves, audio_group::GMAudioGroups, background::GMBackgrounds,
        code::GMCodes, embedded_audio::GMEmbeddedAudios, embedded_image::GMEmbeddedImages,
        embedded_texture::GMEmbeddedTextures, extension::GMExtensions,
        feature_flag::GMFeatureFlags, filter_effect::GMFilterEffects, font::GMFonts,
        function::GMFunctions, game_end::GMGameEndScripts, game_object::GMGameObjects,
        general_info::GMGeneralInfo, global_init::GMGlobalInitScripts, language::GMLanguageInfo,
        options::GMOptions, particle_emitter::GMParticleEmitters,
        particle_system::GMParticleSystems, path::GMPaths, room::GMRooms, script::GMScripts,
        sequence::GMSequences, shader::GMShaders, sound::GMSounds, sprite::GMSprites, tag::GMTags,
        texture_group_info::GMTextureGroupInfos, texture_page_item::GMTexturePageItems,
        timeline::GMTimelines, ui_node::GMRootUINodes, validate_names, variable::GMVariables,
    },
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Byte order (endianness) for integers and chunk names in data files.
///
/// Most modern platforms use little-endian, which is the default.
/// Big-endian support exists for legacy platforms and **may be deprecated**.
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
/// The full GameMaker data struct, containing all information from a data file.
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

    // TODO: should these extra be moved to a different substruct?
    // otherwise they could be confused with chunks (and clutter namespace)
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
    /// This padding is influenced by the data file's GameMaker Version, as well as target platform/architecture.
    pub chunk_padding: u32,

    /// Indicates the data's byte endianness.
    ///
    /// This affects byte order of integers and chunk names.
    /// In most cases (and assumed by default), this is set to little-endian.
    /// Big-endian is an edge case for certain target platforms (e.g. PS3 or Xbox 360)
    /// and its support may be removed in the future.
    pub endianness: Endianness,

    /// The size of the original data file; useful for
    /// approximating the size of the modified data file.
    ///
    /// This is a micro optimization. This field's value
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

            location: None,
            // Use 16 chunk padding by default for compatibility.
            chunk_padding: 16,
            endianness: Endianness::Little,
            original_data_size: 0,
        }
    }
}

impl GMData {
    /// Validates all names of all named root elements.
    /// This checks for duplicates as well as name charset.
    pub fn validate_names(&self) -> Result<()> {
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
        Ok(())
    }

    /// Deserializes all embedded texture pages, turning their underlying image data into [`DynamicImage`].
    ///
    /// This single-threaded implementation may take quite a while.
    /// If you care about performance, I would recommend making a custom
    /// multithreaded implementation (perhaps using the `rayon` crate).
    ///
    /// [`DynamicImage`]: image::DynamicImage
    pub fn deserialize_textures(&mut self) -> Result<()> {
        for texture_page in &mut self.embedded_textures {
            let Some(image) = &mut texture_page.image else {
                continue;
            };
            image
                .deserialize()
                .context("deserializing all embedded texture pages")?;
        }
        Ok(())
    }

    /// Tries to reduce memory footprint by shrinking `Vec`s and `HashMap`s so
    /// they don't take up unneeded space.
    ///
    /// This function may be useful to call once after data deserialization in a long-lived application
    /// (such as a GUI/TUI where the [`GMData`] is stored indefinitely).
    /// It is also useful to call this function after changing formats of lots of texture pages.
    ///
    /// You should not be calling this function frequently, as it consumes CPU power
    /// and will not meaningfully shrink your memory footprint by much.
    ///
    /// Currently, only vectors are shrunk which could've actually been
    /// overallocated by data deserialization (if the size wasn't known).
    /// Most vectors are produced by pointer lists or simple lists which state their exact element count.
    pub fn optimize_memory(&mut self) {
        let mut freed_bytes: usize = 0;

        for code in &mut self.codes {
            freed_bytes += shrink_vec(&mut code.instructions);
            // dbg!(freed_bytes);
        }

        for sequence in &mut self.sequences {
            freed_bytes += shrink_hashmap(&mut sequence.function_ids);
            dbg!(freed_bytes);
        }

        for texture_page in &mut self.embedded_textures {
            let Some(image) = &mut texture_page.image else {
                continue;
            };
            freed_bytes += image.optimize_memory();
            // dbg!(freed_bytes);
        }

        let human_size: String = format_bytes(freed_bytes);
        log::info!("Freed {human_size} ({freed_bytes} bytes)");
    }
}

fn shrink_vec<T>(vector: &mut Vec<T>) -> usize {
    let before = vector.capacity();
    vector.shrink_to_fit();
    let after = vector.capacity();
    before - after
}

fn shrink_hashmap<K: Hash + Eq, V>(hashmap: &mut HashMap<K, V>) -> usize {
    let before = hashmap.capacity();
    hashmap.shrink_to_fit();
    let after = hashmap.capacity();
    before - after
}
