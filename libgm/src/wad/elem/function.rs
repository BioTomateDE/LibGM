// SPDX-License-Identifier: GPL-3.0-only
pub mod code_local;

pub use self::code_local::CodeLocal;
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::GMVersion;
use crate::wad::chunk::ChunkName;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMComplexElement;
use crate::wad::elem::GMElement;
use crate::wad::elem::string::Strings;
use crate::wad::load::chunk::ChunkBounds;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Functions {
    pub elems: Vec<Function>,
    pub code_locals: Vec<CodeLocal>,
}

gm_list_chunk!(FUNC, Functions, Function);

impl Functions {
    #[must_use]
    pub fn find(&self, name: &str, strings: &Strings) -> GMRef<Function> {
        for (gm_ref, func) in self.element_refs() {
            let Ok(func_name) = strings.by_ref(func.name) else {
                continue;
            };
            if name == func_name {
                return gm_ref;
            }
        }
        GMRef::none()
    }

    /// Returns an existing function with the given name if it exists,
    /// otherwise creates a new one.
    pub fn make(&mut self, name: &str, strings: &mut Strings) -> GMRef<Function> {
        let existing = self.find(name, strings);
        if existing.is_some() {
            return existing;
        }
        let name = strings.make(name);
        let idx = self.elems.len();
        let func = Function { name };
        self.elems.push(func);
        GMRef::from(idx)
    }
}

impl GMElement for Functions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let functions_count = if reader.version >= GMVersion::Wad15 {
            reader.read_u32()?
        } else {
            reader.chunk.length() / 12
        };

        let mut elems: Vec<Function> = vec_with_capacity(functions_count)?;

        for idx in 0..functions_count {
            let function: Function = reader.read_elem_ctx(GMRef::from(idx))?;
            elems.push(function);
        }

        let code_locals: Vec<CodeLocal> =
            if (GMVersion::Wad15..GMVersion::GM2024_8).contains(&reader.version) {
                reader.read_simple_list()?
            } else {
                Vec::new()
            };

        Ok(Self { elems, code_locals })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        if writer.version() >= GMVersion::Wad15 {
            writer.write_usize(self.elems.len())?;
        }

        for (idx, function) in self.elems.iter().enumerate() {
            writer.write_elem_ctx(function, idx)?;
        }

        if (GMVersion::Wad15..GMVersion::GM2024_8).contains(&writer.version()) {
            writer.write_simple_list(&self.code_locals)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: GMRef<String>,
}

impl GMComplexElement for Function {
    type ReadContext = GMRef<Self>;
    type WriteContext = usize;

    fn deserialize_complex(reader: &mut DataReader, self_ref: GMRef<Self>) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let occurrence_count = reader.read_u32()?;
        let first_occurrence_pos = reader.read_u32()?;
        read_occurrence_chain(reader, first_occurrence_pos, occurrence_count, self_ref)?;
        Ok(Self { name })
    }

    fn serialize_complex(&self, writer: &mut DataWriter, self_idx: usize) -> Result<()> {
        let occurrences: &Vec<u32> = &writer.function_occurrences[self_idx];
        let occurrence_count: usize = occurrences.len();

        // Before GMS 2.3, the first occurrence points to the instruction rather than the occurrence chain value.
        let gms2_3: bool = writer.version() >= GMVersion::GMS2_3;
        let first_occurrence: i32 = match occurrences.first() {
            Some(&occurrence) if gms2_3 => occurrence as i32 + 4,
            Some(&occurrence) => occurrence as i32,
            None => -1,
        };

        writer.write_gm_string(self.name)?;
        writer.write_usize(occurrence_count)?;
        writer.write_i32(first_occurrence);
        Ok(())
    }
}

fn read_occurrence_chain(
    reader: &mut DataReader,
    first_occurrence_pos: u32,
    occurrence_count: u32,
    func_ref: GMRef<Function>,
) -> Result<()> {
    if occurrence_count < 1 {
        return Ok(());
    }

    let saved_chunk: ChunkBounds = reader.chunk;
    let saved_position = reader.cur_pos;
    reader.chunk = reader
        .chunks
        .get(ChunkName::CODE)
        .ok_or("Chunk CODE not set while parsing function occurrences")?;

    let first_extra_offset: u32 = if reader.version >= GMVersion::GMS2_3 { 0 } else { 4 };
    reader.cur_pos = first_occurrence_pos + first_extra_offset;

    for _ in 0..occurrence_count {
        if let Some(old_func) = reader.function_occurrences.insert(reader.cur_pos, func_ref) {
            bail!(
                "Conflicting function occurrence position {} belongs to both {:?} and {:?}",
                reader.cur_pos,
                old_func,
                func_ref,
            )
        }

        let offset = reader.read_u32()? & 0x07FF_FFFF;
        reader.cur_pos -= 4;
        reader.cur_pos += offset;
    }

    reader.chunk = saved_chunk;
    reader.cur_pos = saved_position;
    Ok(())
}
