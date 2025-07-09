use std::collections::HashMap;
use crate::utility::{format_bytes, typename, Stopwatch};
use crate::gm_serialize::DataBuilder;
use crate::detect_version::detect_gamemaker_version;
use crate::gamemaker::animation_curves::GMAnimationCurves;
use crate::gamemaker::audio_groups::GMAudioGroups;
use crate::gamemaker::backgrounds::GMBackgrounds;
use crate::gamemaker::code::GMCodes;
use crate::gamemaker::data_files::GMDataFiles;
use crate::gamemaker::embedded_audio::GMEmbeddedAudios;
use crate::gamemaker::embedded_images::GMEmbeddedImages;
use crate::gamemaker::embedded_textures::GMEmbeddedTextures;
use crate::gamemaker::extensions::GMExtensions;
use crate::gamemaker::feature_flags::GMFeatureFlags;
use crate::gamemaker::filter_effects::GMFilterEffects;
use crate::gamemaker::fonts::GMFonts;
use crate::gamemaker::functions::{GMFunction, GMFunctions};
use crate::gamemaker::game_objects::GMGameObjects;
use crate::gamemaker::scripts::GMScripts;
use crate::gamemaker::strings::GMStrings;
use crate::gamemaker::variables::{GMVariable, GMVariables};
use crate::gamemaker::general_info::{GMGeneralInfo, GMVersion, GMVersionReq};
use crate::gamemaker::global_init::{GMGameEndScripts, GMGlobalInitScripts};
use crate::gamemaker::languages::GMLanguageInfo;
use crate::gamemaker::paths::GMPaths;
use crate::gamemaker::rooms::GMRooms;
use crate::gamemaker::sounds::GMSounds;
use crate::gamemaker::sprites::GMSprites;
use crate::gamemaker::texture_page_items::{GMTexturePageItem, GMTexturePageItems};
use crate::gamemaker::options::GMOptions;
use crate::gamemaker::particles::{GMParticleEmitters, GMParticleSystems};
use crate::gamemaker::sequence::GMSequences;
use crate::gamemaker::shaders::GMShaders;
use crate::gamemaker::tags::GMTags;
use crate::gamemaker::texture_group_info::GMTextureGroupInfos;
use crate::gamemaker::ui_nodes::GMRootUINodes;
use crate::gamemaker::timelines::GMTimelines;


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
    pub padding: usize,
}


pub fn parse_data_file(raw_data: &Vec<u8>, allow_unread_chunks: bool) -> Result<GMData, String> {
    let stopwatch = Stopwatch::start();
    let mut reader = DataReader::new(&raw_data);

    if reader.read_chunk_name()? != "FORM" {
        return Err("Invalid or corrupted data.win file: 'FORM' chunk missing".to_string());
    }
    let total_data_len: usize = reader.read_usize()? + reader.cur_pos;
    
    while reader.cur_pos + 8 < total_data_len { 
        let name: String = reader.read_chunk_name()?;
        let chunk_length: usize = reader.read_usize()?;
        let start_pos: usize = reader.cur_pos;

        reader.cur_pos += chunk_length;
        if reader.cur_pos > raw_data.len() {
            return Err(format!(
                "Trying to read chunk '{}' out of data bounds: specified length {} implies chunk \
                end position {}; which is greater than the total data length {}",
                name, chunk_length, reader.cur_pos, raw_data.len(),
            ))
        }

        let is_last_chunk: bool = reader.cur_pos == raw_data.len();
        let chunk = GMChunk {
            name: name.clone(),
            start_pos,
            end_pos: reader.cur_pos,
            is_last_chunk,
        };

        if let Some(old_chunk) = reader.chunks.insert(name.clone(), chunk.clone()) {
            return Err(format!(
                "Chunk '{}' is defined multiple times: old data range {}..{}; new data range {}..{}",
                name, old_chunk.start_pos, old_chunk.end_pos, chunk.start_pos, chunk.end_pos,
            ))
        }
    }
    log::trace!("Parsing FORM took {stopwatch}");
    
    reader.strings = reader.read_chunk_required("STRG")?;
    reader.general_info = reader.read_chunk_required("GEN8")?;
    
    let stopwatch2 = Stopwatch::start();
    let old_version: GMVersion = reader.general_info.version.clone();
    let detected_version_opt = detect_gamemaker_version(&mut reader)
        .map_err(|e| format!("{e}\n↳ while detecting gamemaker version"))?;
    log::trace!("Detecting GameMaker Version took {stopwatch2}");

    if let Some(detected_version) = detected_version_opt {
        log::info!("Data file specified GameMaker version {}; detected real version {}", old_version, detected_version);
    } else {
        log::info!("Data file specified GameMaker version {}", old_version);
    }

    let stopwatch2 = Stopwatch::start();
    let embedded_textures: GMEmbeddedTextures = reader.read_chunk_required("TXTR")?;
    let texture_page_items: GMTexturePageItems = reader.read_chunk_required("TPAG")?;
    let variables: GMVariables = reader.read_chunk_required("VARI")?;   // variables and functions have to be parsed before code
    let functions: GMFunctions = reader.read_chunk_required("FUNC")?;
    let scripts: GMScripts = reader.read_chunk_required("SCPT")?;
    let codes: GMCodes = reader.read_chunk_required("CODE")?;
    let fonts: GMFonts = reader.read_chunk_required("FONT")?;
    let sprites: GMSprites = reader.read_chunk_required("SPRT")?;
    let game_objects: GMGameObjects = reader.read_chunk_required("OBJT")?;
    let rooms: GMRooms = reader.read_chunk_required("ROOM")?;
    let backgrounds: GMBackgrounds = reader.read_chunk_required("BGND")?;
    let paths: GMPaths = reader.read_chunk_required("PATH")?;
    let audios: GMEmbeddedAudios = reader.read_chunk_required("AUDO")?;
    let sounds: GMSounds = reader.read_chunk_required("SOND")?;
    let options: GMOptions = reader.read_chunk_required("OPTN")?;
    // some of these chunks probably aren't actually required; make them optional when issues occur

    let sequences: GMSequences = reader.read_chunk_optional("SEQN")?;
    let particle_systems: GMParticleSystems = reader.read_chunk_optional("PSYS")?;
    let particle_emitters: GMParticleEmitters = reader.read_chunk_optional("PSEM")?;
    let language_info: GMLanguageInfo = reader.read_chunk_optional("LANG")?;
    let extensions: GMExtensions = reader.read_chunk_optional("EXTN")?;
    let audio_groups: GMAudioGroups = reader.read_chunk_optional("AGRP")?;
    let global_init_scripts: GMGlobalInitScripts = reader.read_chunk_optional("GLOB")?;
    let game_end_scripts: GMGameEndScripts = reader.read_chunk_optional("GMEN")?;
    let shaders: GMShaders = reader.read_chunk_optional("SHDR")?;
    let root_ui_nodes: GMRootUINodes = reader.read_chunk_optional("UILR")?;
    let data_files: GMDataFiles = reader.read_chunk_optional("DAFL")?;
    let timelines: GMTimelines = reader.read_chunk_optional("TMLN")?;
    let embedded_images: GMEmbeddedImages = reader.read_chunk_optional("EMBI")?;
    let texture_group_infos: GMTextureGroupInfos = reader.read_chunk_optional("TGIN")?;
    let tags: GMTags = reader.read_chunk_optional("TAGS")?;
    let feature_flags: GMFeatureFlags = reader.read_chunk_optional("FEAT")?;
    let filter_effects: GMFilterEffects = reader.read_chunk_optional("FEDS")?;
    let animation_curves: GMAnimationCurves = reader.read_chunk_optional("ACRV")?;
    log::trace!("Parsing chunks took {stopwatch2}");
    
    // Throw error if not all chunks read to prevent silent data loss
    if !allow_unread_chunks && !reader.chunks.is_empty() {
        let chunks_str: String = reader.chunks.keys().cloned().collect::<Vec<_>>().join(", ");
        return Err(format!(
            "Not all chunks in the data file were read, which would lead to data loss when writing.\n\
            The following chunks are unknown or not supported: [{chunks_str}]"
        ))
    }

    let data = GMData {
        general_info: reader.general_info,
        strings: reader.strings,
        embedded_textures,
        texture_page_items,
        variables,
        functions,
        scripts,
        codes,
        fonts,
        sprites,
        game_objects,
        rooms,
        backgrounds,
        paths,
        audios,
        sounds,
        options,
        sequences,
        particle_systems,
        particle_emitters,
        language_info,
        extensions,
        audio_groups,
        global_init_scripts,
        game_end_scripts,
        shaders,
        root_ui_nodes,
        data_files,
        timelines,
        embedded_images,
        texture_group_infos,
        tags,
        feature_flags,
        filter_effects,
        animation_curves,
        padding: reader.padding,
    };

    log::trace!("Parsing data took {stopwatch}");
    Ok(data)
}


/// GMRef has (fake) generic types to make it clearer which type it belongs to (`name: GMRef` vs `name: GMRef<String>`).
/// It can be resolved to the data it references using the `.resolve()` method, which needs the list the elements are stored in.
/// This means that removing or inserting elements in the middle of the list will shift all their `GMRef`s; breaking them.
#[derive(Hash, PartialEq, Eq)]
pub struct GMRef<T> {
    pub index: u32,
    // marker needs to be here to ignore "unused generic T" error; doesn't store any data
    _marker: std::marker::PhantomData<T>,
}

impl<T> GMRef<T> {
    pub fn new(index: u32) -> GMRef<T> {
        Self {
            index,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Clone for GMRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for GMRef<T> {}
impl<T> std::fmt::Debug for GMRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMRef<{}#{}>", typename::<T>(), self.index)
    }
}


impl<'a, T> GMRef<T> {
    pub fn resolve(&self, elements_by_index: &'a Vec<T>) -> Result<&'a T, String> {
        elements_by_index.get(self.index as usize)
            .ok_or_else(|| format!(
                "Could not resolve {} reference with index {} in list with length {}",
                std::any::type_name::<T>(),
                self.index,
                elements_by_index.len(),
            ))
    }
}


#[derive(Debug, Clone)]
pub struct GMChunk {
    pub name: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub is_last_chunk: bool,
}

pub struct DataReader<'a> {
    /// Should not be read until GEN8 chunk is parsed
    pub general_info: GMGeneralInfo,
    /// Should only be set by `gamemaker::string`
    pub strings: GMStrings,

    pub chunks: HashMap<String, GMChunk>,
    pub chunk: GMChunk,

    data: &'a [u8],
    pub cur_pos: usize,
    pub padding: usize,

    /// Should only be set by `gamemaker::strings::GMStrings`
    pub string_occurrence_map: HashMap<usize, GMRef<String>>,
    /// Should only be set by `gamemaker::texture_page_items::GMTexturePageItems`
    pub texture_page_item_occurrence_map: HashMap<usize, GMRef<GMTexturePageItem>>,
    /// Should only be set by `gamemaker::variables::GMVariables`
    pub variable_occurrence_map: HashMap<usize, GMRef<GMVariable>>,
    /// Should only be set by `gamemaker::functions::GMFunctions`
    pub function_occurrence_map: HashMap<usize, GMRef<GMFunction>>,
}
impl<'a> DataReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            general_info: GMGeneralInfo::empty(),
            strings: GMStrings::empty(),
            chunks: HashMap::with_capacity(24),
            chunk: GMChunk {
                name: "FORM".to_string(),
                start_pos: 0,
                end_pos: data.len(),
                is_last_chunk: true,
            },
            data,
            cur_pos: 0,
            padding: 16,    // default padding value (if used) is 16
            string_occurrence_map: HashMap::new(),
            texture_page_item_occurrence_map: HashMap::new(),
            variable_occurrence_map: HashMap::new(),
            function_occurrence_map: HashMap::new(),
        }
    }

    pub fn read_bytes_dyn(&mut self, count: usize) -> Result<&'a [u8], String> {
        // combined check to hopefully increase performance
        if !(self.chunk.start_pos <= self.cur_pos && self.cur_pos+count <= self.chunk.end_pos) {
            return if self.cur_pos < self.chunk.start_pos {
                Err(format!(
                    "out of lower bounds at position {} in chunk '{}' with start position {}",
                    self.cur_pos, self.chunk.name, self.chunk.start_pos,
                ))
            } else {
                Err(format!(
                    "out of upper bounds at position {} in chunk '{}': {} > {}",
                    self.cur_pos, self.chunk.name, self.cur_pos+count, self.chunk.end_pos,
                ))
            }
        }
        // if chunk.start_pos and chunk.end_pos are set correctly; this should never read memory out of bounds.
        let slice: &[u8] = unsafe { self.data.get_unchecked(self.cur_pos..self.cur_pos + count) };
        self.cur_pos += count;
        Ok(slice)
    }
    pub fn read_bytes_const<const N: usize>(&mut self) -> Result<&[u8; N], String> {
        let slice: &[u8] = self.read_bytes_dyn(N)?;
        // read_bytes_dyn is guaranteed to read N bytes so the unwrap never fails.
        Ok(unsafe { &*(slice.as_ptr() as *const [u8; N]) })
    }

    pub fn read_u64(&mut self) -> Result<u64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read u64 {e}"))?;
        Ok(u64::from_le_bytes(*bytes))
    }
    pub fn read_i64(&mut self) -> Result<i64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read i64 {e}"))?;
        Ok(i64::from_le_bytes(*bytes))
    }
    pub fn read_u32(&mut self) -> Result<u32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read u32 {e}"))?;
        Ok(u32::from_le_bytes(*bytes))
    }
    pub fn read_i32(&mut self) -> Result<i32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read i32 {e}"))?;
        Ok(i32::from_le_bytes(*bytes))
    }
    pub fn read_u16(&mut self) -> Result<u16, String> {
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read u16 {e}"))?;
        Ok(u16::from_le_bytes(*bytes))
    }
    pub fn read_i16(&mut self) -> Result<i16, String> {
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read i16 {e}"))?;
        Ok(i16::from_le_bytes(*bytes))
    }
    pub fn read_u8(&mut self) -> Result<u8, String> {
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read u8 {e}"))?;
        Ok(u8::from_le_bytes(*bytes))
    }
    pub fn read_i8(&mut self) -> Result<i8, String> {
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read i8 {e}"))?;
        Ok(i8::from_le_bytes(*bytes))
    }

    pub fn read_f64(&mut self) -> Result<f64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read f64 {e}"))?;
        Ok(f64::from_le_bytes(*bytes))
    }
    pub fn read_f32(&mut self) -> Result<f32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read f32 {e}"))?;
        Ok(f32::from_le_bytes(*bytes))
    }

    pub fn read_usize(&mut self) -> Result<usize, String> {
        let number: u32 = self.read_u32()?;
        Ok(number as usize)
    }

    /// Read unsigned 32-bit integer and convert to usize (little endian).
    /// Meant for reading positions/pointers; uses total data length as failsafe.
    /// Automatically subtracts `chunks.abs_pos`; converting it to a relative chunk position.
    pub fn read_pointer(&mut self) -> Result<usize, String> {
        let failsafe_amount: usize = self.data.len();
        let number: usize = self.read_usize()?;
        if number >= failsafe_amount {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying to read usize \
                (pointer) integer: Number {} ({}) is larger than the total data length of {} ({})",
                self.chunk.name, self.cur_pos-4, number, format_bytes(number), failsafe_amount, format_bytes(failsafe_amount),
            ))
        }
        Ok(number)
    }

    pub fn read_resource_by_id<T>(&mut self) -> Result<GMRef<T>, String> {
        Ok(GMRef::new(self.read_u32()?))
    }

    pub fn read_resource_by_id_opt<T>(&mut self) -> Result<Option<GMRef<T>>, String> {
        const FAILSAFE_COUNT: u32 = 100_000;    // increase limit is not enough
        let number: i32 = self.read_i32()?;
        if number == -1 {
            return Ok(None)
        }
        let number: u32 = number.try_into().map_err(|_| format!(
            "Invalid negative number {number} (0x{number:08X}) while reading optional resource by ID",
        ))?;
        if number > FAILSAFE_COUNT {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} \
                while reading optional resource by ID: \
                Number {} is larger than the failsafe count of {}",
                self.chunk.name, self.cur_pos - 4, number, FAILSAFE_COUNT,
            ))
        }
        Ok(Some(GMRef::new(number)))
    }

    /// Read a 32-bit integer and convert it to a bool.
    /// ___
    /// Returns `Err<String>` when the read number is neither 0 nor 1.
    pub fn read_bool32(&mut self) -> Result<bool, String> {
        let number: u32 = self.read_u32()?;
        match number {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!(
                "Read invalid boolean value in chunk '{0}' at position {1}: {2} (0x{2:08X})",
                self.chunk.name, self.cur_pos, number,
            ))
        }
    }

    pub fn read_literal_string(&mut self, length: usize) -> Result<String, String> {
        let bytes: &[u8] = self.read_bytes_dyn(length)
            .map_err(|e| format!("Trying to read literal string with length {length} {e}"))?;
        let string: String = String::from_utf8(bytes.to_vec()).map_err(|e| format!(
            "Could not parse literal string with length {} in chunk '{}' at position {}: {e}",
            length, self.chunk.name, self.cur_pos,
        ))?;
        Ok(string)
    }

    /// Read chunk name (4 ascii characters)
    pub fn read_chunk_name(&mut self) -> Result<String, String> {
        if self.chunk.start_pos != 0 {
            return Err(format!(
                "Reading a chunk name is only allowed in root; not in a chunk!
                Chunk is called '{}' and has start position {} and end position {}",
                self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
            ))
        }
        let string: String = self.read_literal_string(4)
            .map_err(|e| if self.cur_pos == 4 {
                "Invalid data.win file; data doesn't start with 'FORM' string".to_string()
            } else {
                format!("Could not parse chunk name at position {}: {e}", self.cur_pos)
            })?;
        if string.len() != 4 {
            return Err(format!("Chunk name string \"{string}\" has length {} (chunk names need to be 4 chars long)", string.len()))
        }
        if !string.is_ascii() {
            return Err(format!("Chunk name string \"{string}\" is not ascii"))
        }
        Ok(string)
    }

    fn read_chunk_internal<T: GMChunkElement + GMElement>(&mut self, chunk: GMChunk) -> Result<T, String> {
        let stopwatch = Stopwatch::start();
        self.cur_pos = chunk.start_pos;
        self.chunk = chunk;

        let element = T::deserialize(self)?;
        self.read_chunk_padding()?;

        if self.cur_pos != self.chunk.end_pos {
            return Err(format!(
                "Misaligned chunk '{}': expected chunk end position {} but reader is actually at position {} (diff: {})",
                self.chunk.name, self.chunk.end_pos, self.cur_pos, self.chunk.end_pos as i64 - self.cur_pos as i64,
            ))
        }

        log::trace!("Parsing chunk '{}' took {stopwatch}", self.chunk.name);
        Ok(element)
    }

    fn read_chunk_padding(&mut self) -> Result<(), String> {
        if self.chunk.is_last_chunk {
            return Ok(())   // last chunk does not get padding
        }
        let ver: GMVersion = if self.general_info.exists {
            self.general_info.version.clone()
        } else {
            self.unstable_get_gm_version()?     // only happens before chunk GEN8 is read (STRG)
        };
        if !(ver.major >= 2 || (ver.major == 1 && ver.minor >= 9999)) {
            return Ok(())     // no padding before these versions
        }

        while self.cur_pos % self.padding != 0 {
            let byte: u8 = self.read_u8().map_err(|e| format!("{e}\n↳ while reading chunk padding"))?;
            if byte == 0 { continue }
            // byte is not zero => padding is incorrect
            self.cur_pos -= 1;  // undo reading incorrect padding byte
            self.padding = if self.cur_pos % 4 == 0 { 4 } else { 1 };
            log::debug!("Set padding to {}", self.padding);
            return Ok(())
        }
        Ok(())    // padding was already set correctly
    }

    pub fn read_chunk_required<T: GMChunkElement + GMElement>(&mut self, chunk_name: &str) -> Result<T, String> {
        let chunk: GMChunk = self.chunks.get(chunk_name).ok_or_else(|| format!(
            "Required chunk '{}' not found in chunk hashmap with length {}",
            chunk_name, self.chunks.len(),
        ))?.clone();

        let element: T = self.read_chunk_internal(chunk)
            .map_err(|e| format!("{e}\n↳ while deserializing required chunk '{chunk_name}'"))?;

        // Remove the chunk only after chunk parsing completes.
        // Removing it earlier (e.g. when reading GEN8) would prevent
        // the padding handling from finding the GEN8 chunk in the map,
        // since the real GEN8 info is only set after this function returns.
        self.chunks.remove(chunk_name);
        Ok(element)
    }

    pub fn read_chunk_optional<T: GMChunkElement + GMElement>(&mut self, chunk_name: &str) -> Result<T, String> {
        let Some(chunk) = self.chunks.remove(chunk_name) else {
            log::trace!("Skipped parsing optional chunk '{chunk_name}'");
            return Ok(T::empty())
        };
        let element: T = self.read_chunk_internal(chunk)
            .map_err(|e| format!("{e}\n↳ while deserializing optional chunk '{chunk_name}'"))?;
        Ok(element)
    }

    fn unstable_get_gm_version(&mut self) -> Result<GMVersion, String> {
        let saved_pos: usize = self.cur_pos;
        let saved_chunk: GMChunk = self.chunk.clone();
        self.chunk = self.chunks.get("GEN8").cloned().ok_or("Chunk GEN8 does not exist while trying to (unstable) read gm version")?;
        self.cur_pos = self.chunk.start_pos + 44;   // skip to GEN8 GameMaker version
        let gm_version = GMVersion::deserialize(self)?;
        self.cur_pos = saved_pos;
        self.chunk = saved_chunk;
        Ok(gm_version)
    }

    pub fn read_gm_string(&mut self) -> Result<GMRef<String>, String> {
        let occurrence_position: usize = self.read_usize()?;
        resolve_occurrence(occurrence_position, &self.string_occurrence_map, &self.chunk.name, self.cur_pos)
    }

    pub fn read_gm_texture(&mut self) -> Result<GMRef<GMTexturePageItem>, String> {
        let occurrence_position: usize = self.read_usize()?;
        resolve_occurrence(occurrence_position, &self.texture_page_item_occurrence_map, &self.chunk.name, self.cur_pos)
    }

    pub fn read_gm_string_opt(&mut self) -> Result<Option<GMRef<String>>, String> {
        let occurrence_position: usize = self.read_usize()?;
        if occurrence_position == 0 {
            return Ok(None)
        }
        Ok(Some(resolve_occurrence(occurrence_position, &self.string_occurrence_map, &self.chunk.name, self.cur_pos)?))
    }

    pub fn read_gm_texture_opt(&mut self) -> Result<Option<GMRef<GMTexturePageItem>>, String> {
        let occurrence_position: usize = self.read_usize()?;
        if occurrence_position == 0 {
            return Ok(None)
        }
        Ok(Some(resolve_occurrence(occurrence_position, &self.texture_page_item_occurrence_map, &self.chunk.name, self.cur_pos)?))
    }

    fn read_simple_list_internal<T>(&mut self, deserializer_fn: impl Fn(&mut Self) -> Result<T, String>) -> Result<Vec<T>, String> {
        const FAILSAFE_SIZE: usize = 1_000_000;   // 1 Megabyte
        let count: usize = self.read_usize()?;
        let implied_data_size: usize = count * size_of::<T>();
        if implied_data_size > FAILSAFE_SIZE {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying \
                to read simple list of {}: Element count {} implies a total data \
                size of {} which is larger than the failsafe size of {}",
                self.chunk.name, self.cur_pos-4, typename::<T>(),
                count, format_bytes(implied_data_size), format_bytes(FAILSAFE_SIZE),
            ))
        }
        let mut elements: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            let element: T = deserializer_fn(self)?;
            elements.push(element);
        }
        Ok(elements)
    }

    pub fn read_simple_list<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        self.read_simple_list_internal(T::deserialize)
    }

    pub fn read_simple_list_of_resource_ids<T/*: GMElement*/>(&mut self) -> Result<Vec<GMRef<T>>, String> {
        self.read_simple_list_internal(|reader| reader.read_resource_by_id())
    }

    pub fn read_simple_list_of_strings(&mut self) -> Result<Vec<GMRef<String>>, String> {
        self.read_simple_list_internal(|reader| reader.read_gm_string())
    }

    /// this could probably be moved to gmkerning; it doesn't seem to be used anywhere else
    pub fn read_simple_list_short<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        const FAILSAFE_SIZE: usize = 10_000;   // 10 Kilobytes
        let count: usize = self.read_u16()? as usize;
        let implied_data_size: usize = count * size_of::<T>();
        if implied_data_size > FAILSAFE_SIZE {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying \
                to read short simple list of {}: Element count {} implies a total data \
                size of {} which is larger than the failsafe size of {}",
                self.chunk.name, self.cur_pos-4, typename::<T>(),
                count, format_bytes(implied_data_size), format_bytes(FAILSAFE_SIZE),
            ))
        }
        let mut elements: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            elements.push(T::deserialize(self)?);
        }
        Ok(elements)
    }

    pub fn read_pointer_list<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        // TODO implement 2024.11+ null pointers (unused asset removal)
        let pointers: Vec<usize> = self.read_simple_list()?;
        let count: usize = pointers.len();

        let mut elements: Vec<T> = Vec::with_capacity(count);
        for (i, pointer) in pointers.into_iter().enumerate() {
            // note: this scuffed closure is only used to prevent repetition in map_err.
            //       it will be replaced when try blocks are added to stable.
            let element: T = (|| {
                T::deserialize_pre_padding(self)?;
                self.assert_pos(pointer, &format!("(Pointer list) {}", typename::<T>()))?;
                let element = T::deserialize(self)?;
                T::deserialize_post_padding(self, i == count-1)?;
                Ok(element)
            })().map_err(|e: String| format!(
                "{e}\n↳ while reading pointer list of {} with {} elements",
                typename::<T>(), count,
            ))?;
            elements.push(element);
        }
        Ok(elements)
    }

    /// UndertaleAlignUpdatedListChunk; used for BGND and STRG
    pub fn read_aligned_list_chunk<T: GMElement>(&mut self, alignment: usize, is_aligned: &mut bool) -> Result<Vec<T>, String> {
        let pointers: Vec<usize> = self.read_simple_list()?;
        let mut elements: Vec<T> = Vec::with_capacity(pointers.len());
        
        for pointer in &pointers {
            if pointer % alignment != 0 {
                *is_aligned = false;
            }
            if *pointer == 0 {
                // can happen in 2024.11+ (unused assets removal)
                return Err("Null pointers are not yet supported while parsing aligned list chunk".to_string())
            }
        }
        
        for pointer in pointers {
            if *is_aligned {
                self.align(alignment)?;
            }
            self.assert_pos(pointer, "Aligned list chunk")?;    // UTMT doesn't do this afaik
            let element = T::deserialize(self)?;
            elements.push(element);
        }
        Ok(elements)
    }

    pub fn align(&mut self, alignment: usize) -> Result<(), String> {
        while self.cur_pos & (alignment - 1) != 0 {
            if self.cur_pos > self.chunk.end_pos {
                return Err(format!("Trying to align reader out of chunk bounds at position {}", self.cur_pos))
            }
            self.read_u8()?;
        }
        Ok(())
    }

    pub fn assert_pos(&self, position: usize, pointer_name: &str) -> Result<(), String> {
        if self.cur_pos != position {
            return Err(format!(
                "{} pointer misaligned: expected position {} but reader is actually at {} (diff: {})",
                pointer_name, position, self.cur_pos, position as i64 - self.cur_pos as i64,
            ))
        }
        Ok(())
    }

    pub fn set_rel_cur_pos(&mut self, relative_position: usize) -> Result<(), String> {
        if self.chunk.start_pos + relative_position > self.chunk.end_pos {
            return Err(format!(
                "Tried to set relative reader position to {} in chunk '{}' with start position {} and end position {}; out of bounds",
                relative_position, self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
            ))
        }
        self.cur_pos = self.chunk.start_pos + relative_position;
        Ok(())
    }
    pub fn get_rel_cur_pos(&self) -> usize {
        self.cur_pos - self.chunk.start_pos
    }
    pub fn get_chunk_length(&self) -> usize {
        self.chunk.end_pos - self.chunk.start_pos
    }
    pub fn skip_bytes(&mut self, bytes_count: usize) {
        self.cur_pos += bytes_count;
    }

    pub fn assert_chunk_name(&self, chunk_name: &str) -> Result<(), String> {
        if self.chunk.name == chunk_name {
            Ok(())
        } else {
            Err(format!(
                "Expected chunk with name '{}'; got chunk with name '{}' (length: {})",
                self.chunk.name, chunk_name, chunk_name.len(),
            ))
        }
    }

    pub fn resolve_gm_str(&self, string_ref: GMRef<String>) -> Result<&String, String> {
        string_ref.resolve(&self.strings.strings)
    }
    pub fn display_gm_str(&self, string_ref: GMRef<String>) -> &str {
        string_ref.display(&self.strings)
    }

    pub fn deserialize_if_gm_version<T: GMElement, V: Into<GMVersionReq>>(&mut self, ver_req: V) -> Result<Option<T>, String> {
        if self.general_info.is_version_at_least(ver_req) {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }
    
    pub fn deserialize_if_bytecode_version<T: GMElement>(&mut self, ver_req: u8) -> Result<Option<T>, String> {
        if self.general_info.bytecode_version >= ver_req {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }
}


fn resolve_occurrence<T>(occurrence_position: usize, occurrence_map: &HashMap<usize, GMRef<T>>, chunk_name: &str, position: usize) -> Result<GMRef<T>, String> {
    match occurrence_map.get(&occurrence_position) {
        Some(gm_ref) => Ok(gm_ref.clone()),
        None => Err(format!(
            "Could not read {} with absolute position {} in chunk '{}' at position {} \
            because it doesn't exist in the occurrence map (length: {})",
            typename::<T>(), occurrence_position, chunk_name, position, occurrence_map.len(),
        ))
    }
}


#[allow(unused_variables)]
pub trait GMElement {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> where Self: Sized;
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String>;

    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<(), String> {
        Ok(())
    }
    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<(), String> {
        Ok(())
    }
    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<(), String> {
        Ok(())
    }
    fn serialize_post_padding(&self, builder: &mut DataBuilder, is_last: bool) -> Result<(), String> {
        Ok(())
    }
}

impl GMElement for u8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u8(*self);
        Ok(())
    }
}
impl GMElement for i8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i8(*self);
        Ok(())
    }
}
impl GMElement for u16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u16(*self);
        Ok(())
    }
}
impl GMElement for i16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i16(*self);
        Ok(())
    }
}
impl GMElement for u32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(*self);
        Ok(())
    }
}
impl GMElement for i32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(*self);
        Ok(())
    }
}
impl GMElement for u64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u64(*self);
        Ok(())
    }
}
impl GMElement for i64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i64(*self);
        Ok(())
    }
}
impl GMElement for f32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_f32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(*self);
        Ok(())
    }
}
impl GMElement for f64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_f64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f64(*self);
        Ok(())
    }
}
impl GMElement for usize {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_usize()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_usize(*self)?;
        Ok(())
    }
}
impl GMElement for bool {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_bool32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_bool32(*self);
        Ok(())
    }
}

pub trait GMChunkElement {
    fn empty() -> Self;
    fn exists(&self) -> bool;
}


