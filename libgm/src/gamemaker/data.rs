use crate::gamemaker::elements::animation_curves::GMAnimationCurves;
use crate::gamemaker::elements::audio_groups::GMAudioGroups;
use crate::gamemaker::elements::backgrounds::GMBackgrounds;
use crate::gamemaker::elements::code::{GMCodes, GMInstanceType};
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudios;
use crate::gamemaker::elements::embedded_images::GMEmbeddedImages;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTextures;
use crate::gamemaker::elements::extensions::GMExtensions;
use crate::gamemaker::elements::feature_flags::GMFeatureFlags;
use crate::gamemaker::elements::filter_effects::GMFilterEffects;
use crate::gamemaker::elements::fonts::GMFonts;
use crate::gamemaker::elements::functions::{GMFunction, GMFunctions};
use crate::gamemaker::elements::game_end::GMGameEndScripts;
use crate::gamemaker::elements::game_objects::GMGameObjects;
use crate::gamemaker::elements::general_info::GMGeneralInfo;
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
use crate::gamemaker::elements::variables::{GMVariable, GMVariableB15Data, GMVariables, to_vari_instance_type};
use crate::gamemaker::reference::GMRef;
use crate::prelude::*;

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

#[derive(Debug, Clone)]
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
    /// This padding is influenced by the data file's GameMaker Version, as well as target platform/architecture.
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
    // TODO: make this work for bytecode 14. also docs. also vari_instance_type is wrong/buggy?
    pub fn make_variable_b15(&mut self, name: String, instance_type: GMInstanceType) -> Result<GMRef<GMVariable>> {
        if instance_type == GMInstanceType::Local {
            bail!("Local variables have to be unique; this function will not work");
        }
        let vari_instance_type: GMInstanceType = to_vari_instance_type(&instance_type);

        for (i, variable) in self.variables.iter().enumerate() {
            if variable.name != name {
                continue;
            }

            let Some(b15) = &variable.b15_data else {
                bail!("Variable {} does not have bytecode 15 data", variable.name);
            };
            if b15.instance_type != vari_instance_type {
                continue;
            }

            // Found existing variable!
            return Ok(GMRef::new(i as u32));
        }

        // Couldn't find a variable; make a new one

        // First update these scuffed ass variable counts
        let Some(b15_header) = &mut self.variables.b15_header else {
            bail!("Variables element does not have bytecode 15 header");
        };
        let mut variable_id: i32 = b15_header.var_count1 as i32;

        if self.general_info.is_version_at_least((2, 3)) {
            if instance_type != GMInstanceType::Builtin {
                b15_header.var_count1 += 1;
                b15_header.var_count2 += 1;
                //variable_id = new_name_string.index as i32;
                variable_id = 67;
            }
        } else if self.general_info.bytecode_version >= 16 {
            // this condition is only suggested by utmt; not confirmed (original: `!DifferentVarCounts`)
            b15_header.var_count1 += 1;
            b15_header.var_count2 += 1;
        } else if matches!(vari_instance_type, GMInstanceType::Self_(_)) {
            variable_id = b15_header.var_count2 as i32;
            b15_header.var_count2 += 1;
        } else if vari_instance_type == GMInstanceType::Global {
            b15_header.var_count1 += 1;
        }

        if instance_type == GMInstanceType::Builtin {
            variable_id = -6;
        }

        // Now actually create the variable
        let variable_ref: GMRef<GMVariable> = GMRef::new(self.variables.len() as u32);
        self.variables.push(GMVariable {
            name,
            b15_data: Some(GMVariableB15Data { instance_type, variable_id }),
        });

        Ok(variable_ref)
    }

    fn find_function(&self, name: &str) -> Result<Option<GMRef<GMFunction>>> {
        for (i, function) in self.functions.iter().enumerate() {
            if name == function.name {
                return Ok(Some(GMRef::new(i as u32)));
            }
        }
        Ok(None)
    }

    pub fn function_by_name(&self, name: &str) -> Result<GMRef<GMFunction>> {
        self.find_function(name)?
            .with_context(|| format!("Could not find function with name {name:?}"))
    }
}
