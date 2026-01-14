pub mod code_local;

pub use code_local::{GMCodeLocal, GMCodeLocals};
use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::{chunk::ChunkBounds, reader::DataReader},
        elements::{GMElement, element_stub},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[named_list_chunk("FUNC")]
pub struct GMFunctions {
    pub functions: Vec<GMFunction>,
    pub code_locals: GMCodeLocals,
    pub exists: bool,
}

impl GMElement for GMFunctions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let functions_count = if reader.general_info.wad_version <= 14 {
            reader.chunk.length() / 12
        } else {
            reader.read_u32()?
        };

        let mut functions: Vec<GMFunction> = vec_with_capacity(functions_count)?;

        for i in 0..functions_count {
            let name: String = reader.read_gm_string()?;
            let occurrence_count = reader.read_u32()?;
            let first_occurrence_pos = reader.read_u32()?;
            let (occurrences, _name_string_id): (Vec<u32>, u32) =
                parse_occurrence_chain(reader, first_occurrence_pos, occurrence_count)?;

            // TODO: deal with the name string id somehow (also in VARI)
            // are name string ids relevant in any gamemaker version?
            // otherwise just write whatever lol

            //// verify name string id. allow -1 for unused function
            //if name_string_id as i32 != -1 && name.index != name_string_id {
            //    bail!(
            //        "Function #{i} with name {:?} specifies name string id {}; but the id of name string is actually {}",
            //        name,
            //        name_string_id,
            //        name.index,
            //    )
            //}

            for occurrence in occurrences {
                if let Some(old_func) = reader.function_occurrences.insert(occurrence, i.into()) {
                    bail!(
                        "Conflicting occurrence positions while parsing functions: Position {} \
                        was already set for function #{} with name {:?}; trying to set to function #{} with name {:?}",
                        occurrence,
                        old_func.index,
                        functions[old_func.index as usize].name,
                        i,
                        name,
                    )
                }
            }

            functions.push(GMFunction { name });
        }

        let code_locals: GMCodeLocals = if reader.general_info.wad_version >= 15
            && !reader.general_info.is_version_at_least((2024, 8))
        {
            GMCodeLocals::deserialize(reader)?
        } else {
            GMCodeLocals::default()
        };

        Ok(Self { functions, code_locals, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if builder.wad_version() >= 15 {
            builder.write_usize(self.functions.len())?;
        }

        for (i, function) in self.functions.iter().enumerate() {
            let occurrences: &Vec<usize> = builder.function_occurrences.get(i).ok_or_else(|| {
                format!(
                    "Could not resolve function occurrence with index {i} in list with length {}",
                    builder.function_occurrences.len(),
                )
            })?;
            let occurrence_count: usize = occurrences.len();

            // Before GM 2.3, the first occurrence points to the instruction rather than the next offset
            let gm2_3: bool = builder.is_version_at_least((2, 3));
            let first_occurrence: i32 = match occurrences.first() {
                Some(&occurrence) if gm2_3 => occurrence as i32 + 4,
                Some(&occurrence) => occurrence as i32,
                None => -1,
            };

            builder.write_gm_string(&function.name);
            builder.write_usize(occurrence_count)?;
            builder.write_i32(first_occurrence);
        }

        if builder.wad_version() >= 15 && !builder.is_version_at_least((2024, 8)) {
            if !self.code_locals.exists {
                bail!("Code Locals don't exist in WAD version 15+");
            }
            self.code_locals.serialize(builder)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMFunction {
    pub name: String,
}
element_stub!(GMFunction);

fn parse_occurrence_chain(
    reader: &mut DataReader,
    first_occurrence_pos: u32,
    occurrence_count: u32,
) -> Result<(Vec<u32>, u32)> {
    if occurrence_count < 1 {
        return Ok((vec![], first_occurrence_pos));
    }

    let saved_chunk: ChunkBounds = reader.chunk.clone();
    let saved_position = reader.cur_pos;
    reader.chunk = reader
        .chunks
        .get("CODE")
        .ok_or("Chunk CODE not set while parsing function occurrences")?;

    let first_extra_offset: u32;
    if reader.general_info.is_version_at_least((2, 3)) {
        first_extra_offset = 0;
    } else {
        first_extra_offset = 4;
    }
    let mut occurrence_pos = first_occurrence_pos + first_extra_offset;
    let mut occurrences: Vec<u32> = vec_with_capacity(occurrence_count)?;
    let mut offset: i32 = 6969; // Default value will never be relevant since it returns if no occurrences

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        reader.cur_pos = occurrence_pos;
        let raw_value = reader.read_i32()?;
        offset = raw_value & 0x07FF_FFFF;
        if offset < 1 {
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

    let name_string_id: u32 = (offset & 0xFF_FFFF) as u32;
    reader.chunk = saved_chunk;
    reader.cur_pos = saved_position;
    Ok((occurrences, name_string_id))
}
