use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::code::GMCode;
use crate::gm_serialize::DataBuilder;
use crate::utility::vec_with_capacity;


#[derive(Debug, Clone, PartialEq)]
pub struct GMLanguageData {
    pub name: GMRef<String>,
    pub region: GMRef<String>,
    pub entries: Vec<GMRef<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMLanguageInfo {
    pub unknown1: u32,
    pub languages: Vec<GMLanguageData>,
    pub entry_ids: Vec<GMRef<String>>,
    pub exists: bool,
}
impl GMChunkElement for GMLanguageInfo {
    fn empty() -> Self {
        Self {
            unknown1: 0,
            languages: vec![],
            entry_ids: vec![],
            exists: false,
        }
    }
}
impl GMElement for GMLanguageInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let unknown1: u32 = reader.read_u32()?;
        let language_count: usize = reader.read_usize()?;
        let entry_count: usize = reader.read_usize()?;

        let mut entry_ids: Vec<GMRef<String>> = vec_with_capacity(entry_count)?;
        for _ in 0..entry_count {
            entry_ids.push(reader.read_gm_string()?);
        }

        let mut languages: Vec<GMLanguageData> = vec_with_capacity(language_count)?;
        for _ in 0..language_count {
            let name: GMRef<String> = reader.read_gm_string()?;
            let region: GMRef<String> = reader.read_gm_string()?;
            let mut entries: Vec<GMRef<String>> = Vec::with_capacity(entry_count);
            for _ in 0..entry_count {
                entries.push(reader.read_gm_string()?);
            }
            languages.push(GMLanguageData { name, region, entries });
        }

        Ok(GMLanguageInfo { unknown1, languages, entry_ids, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_u32(self.unknown1);
        builder.write_usize(self.languages.len())?;
        builder.write_usize(self.entry_ids.len())?;
        for entry in &self.entry_ids {
            builder.write_gm_string(entry)?;
        }
        for language in &self.languages {
            builder.write_gm_string(&language.name)?;
            builder.write_gm_string(&language.region)?;
            for entry in &language.entries {
                builder.write_gm_string(entry)?;
            }
        }
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMExtensions {
    pub extensions: Vec<GMExtension>,
    pub exists: bool,
}
impl GMChunkElement for GMExtensions {
    fn empty() -> Self {
        Self { extensions: vec![], exists: false }
    }
}
impl GMElement for GMExtensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let extensions: Vec<GMExtension> = reader.read_pointer_list()?;
        Ok(GMExtensions { extensions, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.extensions)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtension {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
    pub kind: GMExtensionOptionKind,
}
impl GMElement for GMExtension {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        let kind: u32 = reader.read_u32()?;
        let kind: GMExtensionOptionKind = kind.try_into().map_err(|_| format!("Invalid Extension Option Kind {kind} (0x{kind:08X})"))?;
        Ok(GMExtension { name, value, kind })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string(&self.value)?;
        builder.write_u32(self.kind.into());
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMExtensionOptionKind {
    Boolean = 0,
    Number = 1,
    String = 2,
}



#[derive(Debug, Clone)]
pub struct GMAudioGroups {
    pub audio_groups: Vec<GMAudioGroup>,
    pub exists: bool,
}
impl GMChunkElement for GMAudioGroups {
    fn empty() -> Self {
        Self { audio_groups: vec![], exists: false }
    }
}
impl GMElement for GMAudioGroups {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let audio_groups: Vec<GMAudioGroup> = reader.read_pointer_list()?;
        Ok(Self { audio_groups, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.audio_groups)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAudioGroup {
    pub name: GMRef<String>,
    pub path: Option<GMRef<String>>,
}
impl GMElement for GMAudioGroup {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let path: Option<GMRef<String>> = if reader.general_info.is_version_at_least((2024, 14)) {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        Ok(Self { name, path })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.resolve_pointer_elem(self)?;
        builder.write_gm_string(&self.name)?;
        if builder.is_gm_version_at_least((2024, 14)) {
            builder.write_gm_string(&self.path.ok_or("Audio Group Path not set for 2024.14+")?)?;
        }
        Ok(())
    }
}



#[derive(Debug, Clone)]
pub struct GMGlobalInitScripts {
    pub global_inits: Vec<GMGlobalInit>,
    pub exists: bool,
}
impl GMChunkElement for GMGlobalInitScripts {
    fn empty() -> Self {
        Self { global_inits: vec![], exists: false }
    }
}
impl GMElement for GMGlobalInitScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let global_inits: Vec<GMGlobalInit> = reader.read_simple_list()?;
        Ok(Self { global_inits, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_simple_list(&self.global_inits)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMGameEndScripts {
    pub global_inits: Vec<GMGlobalInit>,
    pub exists: bool,
}
impl GMChunkElement for GMGameEndScripts {
    fn empty() -> Self {
        Self { global_inits: vec![], exists: false }
    }
}
impl GMElement for GMGameEndScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let global_inits: Vec<GMGlobalInit> = reader.read_simple_list()?;
        Ok(Self { global_inits, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_simple_list(&self.global_inits)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMGlobalInit {
    pub code: GMRef<GMCode>,
}
impl GMElement for GMGlobalInit {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let code: GMRef<GMCode> = reader.read_resource_by_id()?;
        Ok(Self { code })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_resource_id(&self.code);
        Ok(())
    }
}

