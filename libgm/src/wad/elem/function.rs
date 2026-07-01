// SPDX-License-Identifier: GPL-3.0-only
pub mod code_local;

pub use self::code_local::CodeLocal;
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::ChunkName;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::element_stub;
use crate::wad::elem::string::Strings;
use crate::wad::parse::chunk::ChunkBounds;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Functions {
    pub elems: Vec<Function>,
    pub code_locals: Vec<CodeLocal>,
    pub exists: bool,
}

gm_named_list_chunk!(FUNC, Functions, Function, direct);

impl Functions {
    /// Returns an existing function with the given name if it exists,
    /// otherwise creates a new one.
    pub fn make(&mut self, name: &str, strings: &mut Strings) -> GMRef<Function> {
        if let Ok(func_ref) = self.ref_by_name(name, strings) {
            return func_ref;
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
        let functions_count = if reader.general_info.wad_version >= 15 {
            reader.read_u32()?
        } else {
            reader.chunk.length() / 12
        };

        let mut elems: Vec<Function> = vec_with_capacity(functions_count)?;

        for i in 0..functions_count {
            let name: GMRef<String> = reader.read_gm_string()?;
            let occurrence_count = reader.read_u32()?;
            let first_occurrence_pos = reader.read_u32()?;
            let occurrences: Vec<u32> =
                parse_occurrence_chain(reader, first_occurrence_pos, occurrence_count)?;

            for occurrence in occurrences {
                if let Some(old_func) = reader.function_occurrences.insert(occurrence, i.into()) {
                    bail!(
                        "Conflicting occurrence positions while parsing functions: Position {} \
                         was already set for function #{} with name {:?}; trying to set to \
                         function #{} with name {:?}",
                        occurrence,
                        old_func.index,
                        elems[old_func.index as usize].name,
                        i,
                        name,
                    )
                }
            }

            elems.push(Function { name });
        }

        let code_locals: Vec<CodeLocal> =
            if reader.general_info.wad_version >= 15 && reader.general_info.version < (2024, 8) {
                reader.read_simple_list()?
            } else {
                Vec::new()
            };

        Ok(Self { elems, code_locals, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if builder.wad_version() >= 15 {
            builder.write_usize(self.elems.len())?;
        }

        for (i, function) in self.elems.iter().enumerate() {
            let occurrences: &Vec<u32> = builder.function_occurrences.get(i).ok_or_else(|| {
                format!(
                    "Could not resolve function occurrence with index {i} in list with length {}",
                    builder.function_occurrences.len(),
                )
            })?;
            let occurrence_count: usize = occurrences.len();

            // Before GM 2.3, the first occurrence points to the instruction rather than the
            // next offset
            let gm2_3: bool = builder.version() >= (2, 3);
            let first_occurrence: i32 = match occurrences.first() {
                Some(&occurrence) if gm2_3 => occurrence as i32 + 4,
                Some(&occurrence) => occurrence as i32,
                None => -1,
            };

            builder.write_gm_string(function.name)?;
            builder.write_usize(occurrence_count)?;
            builder.write_i32(first_occurrence);
        }

        if builder.wad_version() >= 15 && builder.version() < (2024, 8) {
            builder.write_simple_list(&self.code_locals)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: GMRef<String>,
}
element_stub!(Function);

fn parse_occurrence_chain(
    reader: &mut DataReader,
    first_occurrence_pos: u32,
    occurrence_count: u32,
) -> Result<Vec<u32>> {
    if occurrence_count < 1 {
        return Ok(vec![]);
    }

    let saved_chunk: ChunkBounds = reader.chunk;
    let saved_position = reader.cur_pos;
    reader.chunk = reader
        .chunks
        .get(ChunkName::CODE)
        .ok_or("Chunk CODE not set while parsing function occurrences")?;

    let first_extra_offset: u32 = if reader.general_info.version >= (2, 3) {
        0
    } else {
        4
    };
    let mut occurrence_pos = first_occurrence_pos + first_extra_offset;
    let mut occurrences: Vec<u32> = vec_with_capacity(occurrence_count)?;
    let mut offset: i32;

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        reader.cur_pos = occurrence_pos;
        let raw_value = reader.read_i32()?;
        offset = raw_value & 0x07FF_FFFF;
        if offset < 0 {
            bail!(
                "Next occurrence offset is {0} (0x{0:08X}) which is negative while parsing \
                 function occurrences at position {1} (raw value is 0x{2:08X})",
                offset,
                reader.cur_pos - 4,
                raw_value,
            )
        }
        occurrence_pos += offset as u32;
    }

    reader.chunk = saved_chunk;
    reader.cur_pos = saved_position;
    Ok(occurrences)
}
