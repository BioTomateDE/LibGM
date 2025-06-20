use std::collections::HashMap;
use crate::deserialize::chunk_reading::{vec_with_capacity, DataReader, GMChunk, GMChunkElement, GMElement, GMRef};


#[derive(Debug, Clone)]
pub struct GMFunctions {
    pub functions: Vec<GMFunction>,
    pub code_locals: GMCodeLocals,
    pub exists: bool,
}
impl GMChunkElement for GMFunctions {
    fn empty() -> Self {
        Self {
            functions: vec![],
            code_locals: GMCodeLocals::empty(),
            exists: false,
        }
    }
}
impl GMElement for GMFunctions {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let functions_count: usize = if reader.general_info.bytecode_version <= 14 {
            reader.get_chunk_length() / 12
        } else {
            reader.read_usize()?
        };

        let mut functions: Vec<GMFunction> = vec_with_capacity(functions_count)?;
        let mut occurrences_to_refs: HashMap<usize, GMRef<GMFunction>> = HashMap::with_capacity(functions_count);

        for i in 0..functions_count {
            let name: GMRef<String> = reader.read_gm_string()?;
            let occurrence_count: usize = reader.read_usize()?;
            let first_occurrence_abs_pos: i32 = reader.read_i32()?;
            let (occurrences, name_string_id): (Vec<usize>, i32) = parse_occurrence_chain(reader, first_occurrence_abs_pos, occurrence_count)?;
            
            for occurrence in &occurrences {
                if let Some(old_value) = occurrences_to_refs.insert(*occurrence, GMRef::new(i as u32)) {
                    return Err(format!(
                        "Conflicting occurrence positions while parsing functions: absolute position {} \
                        was already set for function #{} with name \"{}\"; trying to set to function #{} with name \"{}\"",
                        occurrence, old_value.index, reader.display_gm_str(old_value.resolve(&functions)?.name), i, reader.display_gm_str(name),
                    ))
                }
            }

            functions.push(GMFunction { name, name_string_id });
        }

        let code_locals: GMCodeLocals = GMCodeLocals::deserialize(reader)?;
        Ok(GMFunctions { functions, code_locals, exists: true })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMFunction {
    pub name: GMRef<String>,
    pub name_string_id: i32,
}


#[derive(Debug, Clone)]
pub struct GMCodeLocals {
    pub code_locals: Vec<GMCodeLocal>,
    pub exists: bool,
}
impl GMChunkElement for GMCodeLocals {
    fn empty() -> Self {
        Self { code_locals: vec![], exists: false }
    }
}
impl GMElement for GMCodeLocals {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        if reader.general_info.bytecode_version <= 14 || reader.general_info.is_version_at_least((2024, 8, 0, 0)) {
            return Ok(Self::empty())
        }
        let code_locals: Vec<GMCodeLocal> = reader.read_simple_list()?;
        Ok(Self { code_locals, exists: true })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeLocal {
    pub name: GMRef<String>,
    pub variables: Vec<GMCodeLocalVariable>,
}
impl GMElement for GMCodeLocal {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let local_variables_count: usize = reader.read_usize()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let mut variables: Vec<GMCodeLocalVariable> = vec_with_capacity(local_variables_count)?;
        for _ in 0..local_variables_count {
            variables.push(GMCodeLocalVariable::deserialize(reader)?);
        }
        Ok(GMCodeLocal { name, variables })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeLocalVariable {
    /// unknown what this does
    pub index: u32,
    pub name: GMRef<String>,
}
impl GMElement for GMCodeLocalVariable {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let index: u32 = reader.read_u32()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        Ok(GMCodeLocalVariable { index, name })
    }
}


pub fn parse_occurrence_chain(reader: &mut DataReader, first_occurrence_pos: i32, occurrence_count: usize) -> Result<(Vec<usize>, i32), String> {
    if occurrence_count < 1 {
        return Ok((vec![], first_occurrence_pos));
    }

    let saved_chunk: GMChunk = reader.chunk.clone();
    let saved_position: usize = reader.cur_pos;
    reader.chunk = reader.chunks.get("CODE").cloned().ok_or("Chunk CODE not set while parsing function occurrences")?;

    let first_extra_offset: usize;
    if reader.general_info.is_version_at_least((2, 3, 0, 0)) {
        first_extra_offset = 0;
    } else {
        first_extra_offset = 4;
    };
    let mut occurrence_pos: usize = first_occurrence_pos as usize + first_extra_offset;
    let mut occurrences: Vec<usize> = vec_with_capacity(occurrence_count)?;
    let mut offset: i32 = first_occurrence_pos;

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        reader.cur_pos = occurrence_pos;
        let raw_value: i32 = reader.read_i32()?;
        offset = raw_value & 0x07FFFFFF;
        if offset < 1 {
            return Err(format!(
                "Next occurrence offset is {0} (0x{0:08X}) which is negative while parsing \
                function occurrences at position {1} (raw value is 0x{2:08X})",
                offset, reader.cur_pos-4, raw_value,
            ))
        }
        occurrence_pos = offset as usize;   // might overflow on last occurrence (name string id) but doesn't matter
    }

    let name_string_id: i32 = offset & 0xFFFFFF;
    reader.chunk = saved_chunk;
    reader.cur_pos = saved_position;
    Ok((occurrences, name_string_id))
}


