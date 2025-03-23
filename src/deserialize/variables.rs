use crate::chunk_reading::UTChunk;
use crate::deserialize::strings::UTStrings;

#[derive(Debug, Clone)]
pub struct UTVariable {
    pub name: String,
    pub instance_type: i32,
    pub variable_id: i32,
    pub occurrences_count: u32,
    pub first_occurrence_address: u32,
    pub abs_pos: usize,
}

pub fn parse_chunk_VARI(mut chunk: UTChunk, strings: &UTStrings) -> Result<Vec<UTVariable>, String> {
    let _unknown1: u32 = chunk.read_u32()?;
    let _unknown2: u32 = chunk.read_u32()?;
    let _unknown3: u32 = chunk.read_u32()?;
    let file_len: usize = chunk.data.len();
    let mut variables: Vec<UTVariable> = vec![];

    while chunk.file_index < file_len {
        let abs_pos: usize = chunk.file_index;
        variables.push(UTVariable {
            name: chunk.read_ut_string(strings)?,
            instance_type: chunk.read_i32()?,
            variable_id: chunk.read_i32()?,
            occurrences_count: chunk.read_u32()?,
            first_occurrence_address: chunk.read_u32()?,
            abs_pos,
        })
    }

    Ok(variables)
}
