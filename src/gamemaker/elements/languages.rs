use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::utility::vec_with_capacity;

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
    fn exists(&self) -> bool {
        self.exists
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


#[derive(Debug, Clone, PartialEq)]
pub struct GMLanguageData {
    pub name: GMRef<String>,
    pub region: GMRef<String>,
    pub entries: Vec<GMRef<String>>,
}

