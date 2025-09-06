use crate::gamemaker::deserialize::GMRef;
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
use crate::gamemaker::elements::functions::GMFunctions;
use crate::gamemaker::elements::game_objects::GMGameObjects;
use crate::gamemaker::elements::general_info::GMGeneralInfo;
use crate::gamemaker::elements::global_init::{GMGameEndScripts, GMGlobalInitScripts};
use crate::gamemaker::elements::languages::GMLanguageInfo;
use crate::gamemaker::elements::options::GMOptions;
use crate::gamemaker::elements::particles::{GMParticleEmitters, GMParticleSystems};
use crate::gamemaker::elements::paths::GMPaths;
use crate::gamemaker::elements::rooms::GMRooms;
use crate::gamemaker::elements::scripts::GMScripts;
use crate::gamemaker::elements::sequence::GMSequences;
use crate::gamemaker::elements::shaders::GMShaders;
use crate::gamemaker::elements::sounds::GMSounds;
use crate::gamemaker::elements::sprites::GMSprites;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::tags::GMTags;
use crate::gamemaker::elements::texture_group_info::GMTextureGroupInfos;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItems;
use crate::gamemaker::elements::timelines::GMTimelines;
use crate::gamemaker::elements::ui_nodes::GMRootUINodes;
use crate::gamemaker::elements::variables::GMVariables;

#[derive(Debug, Clone)]
pub struct GMData {
    pub general_info: GMGeneralInfo,                    // GEN8
    pub strings: GMStrings,                             // STRG
    pub embedded_textures: GMEmbeddedTextures,          // TXTR
    pub texture_page_items: GMTexturePageItems,         // TPAG
    pub variables: GMVariables,                         // VARI
    pub functions: GMFunctions,                         // FUNC
    pub scripts: GMScripts,                             // SCPT
    pub codes: GMCodes,                                 // CODE
    pub fonts: GMFonts,                                 // FONT
    pub sprites: GMSprites,                             // SPRT
    pub game_objects: GMGameObjects,                    // OBJT
    pub rooms: GMRooms,                                 // ROOM
    pub backgrounds: GMBackgrounds,                     // BGND
    pub paths: GMPaths,                                 // PATH
    pub audios: GMEmbeddedAudios,                       // AUDO
    pub sounds: GMSounds,                               // SOND
    pub options: GMOptions,                             // OPTN
    pub sequences: GMSequences,                         // SEQN
    pub particle_systems: GMParticleSystems,            // PSYS
    pub particle_emitters: GMParticleEmitters,          // PSEM
    pub language_info: GMLanguageInfo,                  // LANG
    pub extensions: GMExtensions,                       // EXTN
    pub audio_groups: GMAudioGroups,                    // AGRP
    pub global_init_scripts: GMGlobalInitScripts,       // GLOB
    pub game_end_scripts: GMGameEndScripts,             // GMEN
    pub shaders: GMShaders,                             // SHDR
    pub root_ui_nodes: GMRootUINodes,                   // UILR
    pub data_files: GMDataFiles,                        // DAFL
    pub timelines: GMTimelines,							// TMLN
    pub embedded_images: GMEmbeddedImages,              // EMBI
    pub texture_group_infos: GMTextureGroupInfos,       // TGIN
    pub tags: GMTags,                                   // TAGS
    pub feature_flags: GMFeatureFlags,                  // FEAT
    pub filter_effects: GMFilterEffects,                // FEDS
    pub animation_curves: GMAnimationCurves,            // ACRV

    /// Should not be edited; only set by `GMData::read_chunk_padding`.
    pub chunk_padding: usize,

    /// Whether the data file is formatted as big endian.
    /// This is only the case for certain target architectures.
    pub is_big_endian: bool,

    /// Size of the original data file; useful for approximating.
    pub original_data_size: usize,
}

impl GMData {
    pub fn make_string(&mut self, string: &str) -> GMRef<String> {
        // Try to find existing string
        for (i, str) in self.strings.strings.iter().enumerate() {
            if str == string {
                return GMRef::new(i as u32)
            }
        }

        // Make new string
        self.make_unique_string(string.to_string())
    }

    pub fn make_unique_string(&mut self, string: String) -> GMRef<String> {
        let index: usize = self.strings.strings.len();
        self.strings.strings.push(string);
        GMRef::new(index as u32)
    }
}

