use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::animation_curves::GMAnimationCurves;
use crate::gamemaker::elements::audio_groups::GMAudioGroups;
use crate::gamemaker::elements::backgrounds::GMBackgrounds;
use crate::gamemaker::elements::code::{GMCodes, GMInstanceType};
use crate::gamemaker::elements::data_files::GMDataFiles;
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudios;
use crate::gamemaker::elements::embedded_images::GMEmbeddedImages;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTextures;
use crate::gamemaker::elements::extensions::GMExtensions;
use crate::gamemaker::elements::feature_flags::GMFeatureFlags;
use crate::gamemaker::elements::filter_effects::GMFilterEffects;
use crate::gamemaker::elements::fonts::GMFonts;
use crate::gamemaker::elements::functions::{GMFunction, GMFunctions};
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
use crate::gamemaker::elements::variables::{to_vari_instance_type, GMVariable, GMVariableB15Data, GMVariables};

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

    pub fn make_variable_b15(&mut self, name: &str, instance_type: GMInstanceType) -> Result<GMRef<GMVariable>, String> {
        if instance_type == GMInstanceType::Local {
            return Err("Local variables have to be unique; this function will not work".to_string())
        }
        let vari_instance_type: GMInstanceType = to_vari_instance_type(&instance_type);

        for (i, variable) in self.variables.variables.iter().enumerate() {
            let var_name: &String = variable.name.resolve(&self.strings.strings)?;
            if var_name != name {
                continue
            }

            let Some(b15) = &variable.b15_data else {
                return Err(format!("Variable {} does not have bytecode 15 data", variable.name.display(&self.strings)))
            };
            if b15.instance_type != vari_instance_type {
                continue
            }

            // found existing variable!
            return Ok(GMRef::new(i as u32))
        }

        // couldn't find a variable; make a new one
        let new_name_string: GMRef<String> = self.make_string(name);

        // first update these scuffed ass variable counts
        let Some(b15_header) = &mut self.variables.b15_header else {
            return Err("Variables element does not have bytecode 15 header".to_string())
        };
        let mut variable_id: i32 = b15_header.var_count1 as i32;

        if self.general_info.is_version_at_least((2, 3)) {
            if instance_type != GMInstanceType::Builtin {
                b15_header.var_count1 += 1;
                b15_header.var_count2 += 1;
                variable_id = new_name_string.index as i32;
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

        // now actually create the variable
        let variable_ref: GMRef<GMVariable> = GMRef::new(self.variables.variables.len() as u32);
        self.variables.variables.push(GMVariable {
            name: new_name_string,
            b15_data: Some(GMVariableB15Data { instance_type, variable_id }),
        });

        Ok(variable_ref)
    }

    fn find_function(&self, name: &str) -> Result<Option<GMRef<GMFunction>>, String> {
        for (i, function) in self.functions.functions.iter().enumerate() {
            let func_name: &String = function.name.resolve(&self.strings.strings)?;
            if name == func_name {
                return Ok(Some(GMRef::new(i as u32)))
            }
        }
        Ok(None)
    }

    pub fn function_by_name(&self, name: &str) -> Result<GMRef<GMFunction>, String> {
        self.find_function(name)?.ok_or_else(|| format!("Could not find function with name \"{name}\""))
    }

    /// Only intended for finding (or creating if it doesn't exist) **builtin** GameMaker functions.
    pub fn make_builtin_function(&mut self, name: &'static str) -> Result<GMRef<GMFunction>, String> {
        if let Some(func) = self.find_function(name)? {
            return Ok(func)
        }

        // create new function
        let func_ref = GMRef::new(self.functions.functions.len() as u32);
        let func = GMFunction {
            name: self.make_string(name),
        };
        self.functions.functions.push(func);    // separation needed to shut up the borrow checker
        Ok(func_ref)
    }
}

