// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::element_stub;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LanguageInfo {
    pub unknown1: u32,
    pub elems: Vec<LanguageData>,
    pub entry_ids: Vec<GMRef<String>>,
}

gm_list_chunk!(LANG, LanguageInfo, LanguageData, direct);

impl GMElement for LanguageInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let unknown1 = reader.read_u32()?;
        let language_count = reader.read_u32()?;
        let entry_count = reader.read_u32()?;

        let mut entry_ids: Vec<GMRef<String>> = vec_with_capacity(entry_count)?;
        for _ in 0..entry_count {
            entry_ids.push(reader.read_gm_string()?);
        }

        let mut elems: Vec<LanguageData> = vec_with_capacity(language_count)?;
        for _ in 0..language_count {
            let name: GMRef<String> = reader.read_gm_string()?;
            let region: GMRef<String> = reader.read_gm_string()?;
            let mut entries: Vec<GMRef<String>> = Vec::with_capacity(entry_count as usize);
            for _ in 0..entry_count {
                entries.push(reader.read_gm_string()?);
            }
            elems.push(LanguageData { name, region, entries });
        }

        Ok(Self { unknown1, elems, entry_ids })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.unknown1);
        builder.write_usize(self.elems.len())?;
        builder.write_usize(self.entry_ids.len())?;
        for &entry in &self.entry_ids {
            builder.write_gm_string(entry)?;
        }
        for language in &self.elems {
            builder.write_gm_string(language.name)?;
            builder.write_gm_string(language.region)?;
            for &entry in &language.entries {
                builder.write_gm_string(entry)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageData {
    pub name: GMRef<String>,
    pub region: GMRef<String>,
    pub entries: Vec<GMRef<String>>,
}
element_stub!(LanguageData);
