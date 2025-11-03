use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMLanguageInfo {
    pub unknown1: u32,
    pub languages: Vec<GMLanguageData>,
    pub entry_ids: Vec<GMRef<String>>,
    pub exists: bool,
}

impl Deref for GMLanguageInfo {
    type Target = Vec<GMLanguageData>;
    fn deref(&self) -> &Self::Target {
        &self.languages
    }
}

impl DerefMut for GMLanguageInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.languages
    }
}

impl GMChunkElement for GMLanguageInfo {
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMLanguageInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let unknown1 = reader.read_u32()?;
        let language_count = reader.read_u32()?;
        let entry_count = reader.read_u32()?;

        let mut entry_ids: Vec<GMRef<String>> = vec_with_capacity(entry_count)?;
        for _ in 0..entry_count {
            entry_ids.push(reader.read_gm_string()?);
        }

        let mut languages: Vec<GMLanguageData> = vec_with_capacity(language_count)?;
        for _ in 0..language_count {
            let name: GMRef<String> = reader.read_gm_string()?;
            let region: GMRef<String> = reader.read_gm_string()?;
            let mut entries: Vec<GMRef<String>> = Vec::with_capacity(entry_count as usize);
            for _ in 0..entry_count {
                entries.push(reader.read_gm_string()?);
            }
            languages.push(GMLanguageData { name, region, entries });
        }

        Ok(GMLanguageInfo { unknown1, languages, entry_ids, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMLanguageData {
    pub name: GMRef<String>,
    pub region: GMRef<String>,
    pub entries: Vec<GMRef<String>>,
}
