use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::GMCode;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMLanguageRoot {
    pub unknown1: u32,
    pub languages: Vec<GMLanguageData>,
    pub entry_ids: Vec<GMRef<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMLanguageData {
    pub name: GMRef<String>,
    pub region: GMRef<String>,
    pub entries: Vec<GMRef<String>>,
}

pub fn parse_chunk_lang(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMLanguageRoot, String> {
    chunk.cur_pos = 0;
    let unknown1: u32 = chunk.read_u32()?;
    let language_count: usize = chunk.read_usize_count()?;
    let entry_count: usize = chunk.read_usize_count()?;

    let mut languages: Vec<GMLanguageData> = Vec::with_capacity(language_count);
    let mut entry_ids: Vec<GMRef<String>> = Vec::with_capacity(language_count);

    for _ in 0..entry_count {
        entry_ids.push(chunk.read_gm_string(strings)?);
    }

    for _ in 0..language_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let region: GMRef<String> = chunk.read_gm_string(strings)?;
        let mut entries: Vec<GMRef<String>> = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(chunk.read_gm_string(strings)?);
        }
        languages.push(GMLanguageData {
            name,
            region,
            entries,
        })
    }

    Ok(GMLanguageRoot {
        unknown1,
        languages,
        entry_ids,
    })
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtension {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
    pub kind: GMExtensionOptionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMExtensionOptionKind {
    Boolean = 0,
    Number = 1,
    String = 2,
}

#[derive(Debug, Clone)]
pub struct GMExtensions {
    pub extensions: Vec<GMExtension>,
    pub serialize: bool,
}
impl GMExtensions {
    pub fn empty() -> Self {
        Self { extensions: vec![], serialize: false }
    }
}

pub fn parse_chunk_extn(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMExtensions, String> {
    chunk.cur_pos = 0;
    let extension_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(extension_count);
    for _ in 0..extension_count {
        start_positions.push(chunk.read_relative_pointer()?);
    }
    
    let mut extensions: Vec<GMExtension> = Vec::with_capacity(extension_count);
    for start_pos in start_positions {
        chunk.cur_pos = start_pos;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let value: GMRef<String> = chunk.read_gm_string(strings)?;
        let kind: u32 = chunk.read_u32()?;
        let kind: GMExtensionOptionKind = kind.try_into().map_err(|_| format!("Invalid Extension Option Kind {kind} (0x{kind:08X})"))?;
        extensions.push(GMExtension {
            name,
            value,
            kind,
        });
    }

    Ok(GMExtensions { extensions, serialize: true })
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAudioGroup {
    pub name: GMRef<String>,
    pub path: Option<GMRef<String>>,
}

#[derive(Debug, Clone)]
pub struct GMAudioGroups {
    pub audio_groups: Vec<GMAudioGroup>,
    pub serialize: bool,
}
impl GMAudioGroups {
    pub fn empty() -> Self {
        Self { audio_groups: vec![], serialize: false }
    }
}

pub fn parse_chunk_agrp(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMAudioGroups, String> {
    chunk.cur_pos = 0;
    let audio_group_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(audio_group_count);
    for _ in 0..audio_group_count {
        start_positions.push(chunk.read_relative_pointer()?);
    }

    let mut audio_groups: Vec<GMAudioGroup> = Vec::with_capacity(audio_group_count);
    for start_pos in start_positions {
        chunk.cur_pos = start_pos;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let path: Option<GMRef<String>> = if general_info.is_version_at_least(2024, 14, 0, 0) {
            Some(chunk.read_gm_string(strings)?)
        } else { None };
        audio_groups.push(GMAudioGroup {
            name,
            path,
        });
    }

    Ok(GMAudioGroups { audio_groups, serialize: true })
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMGlobalInit {
    pub code: GMRef<GMCode>,
}

#[derive(Debug, Clone)]
pub struct GMGlobalInits {
    pub global_inits: Vec<GMGlobalInit>,
    pub serialize: bool,
}
impl GMGlobalInits {
    pub fn empty() -> Self {
        Self { global_inits: vec![], serialize: false }
    }
}

pub fn parse_chunk_glob(chunk: &mut GMChunk) -> Result<GMGlobalInits, String> {
    // TODO also chunk 'GMEN'???
    chunk.cur_pos = 0;
    let global_inits_count: usize = chunk.read_usize_count()?;
    let mut global_inits: Vec<GMGlobalInit> = Vec::with_capacity(global_inits_count);
    
    for _ in 0..global_inits_count {
        let code: GMRef<GMCode> = chunk.read_resource_by_id()?;
        global_inits.push(GMGlobalInit { code });
    }

    Ok(GMGlobalInits { global_inits, serialize: true })
}

