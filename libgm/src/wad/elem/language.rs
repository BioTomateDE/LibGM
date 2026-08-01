// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMComplexElement;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LanguageInfo {
    pub unknown1: u32,
    pub elems: Vec<LanguageData>,
    pub entry_ids: Vec<GMRef<String>>,
}

gm_list_chunk!(LANG, LanguageInfo, LanguageData);

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
            let language_data: LanguageData = reader.read_elem_ctx(entry_count)?;
            elems.push(language_data);
        }

        Ok(Self { unknown1, elems, entry_ids })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_u32(self.unknown1);
        writer.write_usize(self.elems.len())?;
        writer.write_usize(self.entry_ids.len())?;
        for &entry in &self.entry_ids {
            writer.write_gm_string(entry)?;
        }

        let expected_count = self.elems.first().map_or(0, |lang| lang.entries.len());

        for language in &self.elems {
            let actual_count = language.entries.len();
            if actual_count != expected_count {
                bail!("Expected {expected_count} entries, got {actual_count} entries");
            }
            writer.write_elem_ctx(language, ())?;
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

impl GMComplexElement for LanguageData {
    type ReadContext = u32;
    type WriteContext = ();

    fn deserialize_complex(
        reader: &mut DataReader,
        entry_count: Self::ReadContext,
    ) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let region: GMRef<String> = reader.read_gm_string()?;
        let mut entries: Vec<GMRef<String>> = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(reader.read_gm_string()?);
        }
        Ok(Self { name, region, entries })
    }

    fn serialize_complex(&self, writer: &mut DataWriter, _ctx: ()) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_gm_string(self.region)?;
        for &entry in &self.entries {
            writer.write_gm_string(entry)?;
        }
        Ok(())
    }
}
