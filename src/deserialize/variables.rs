use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub instance_type: i32,
    pub variable_id: i32,
    pub occurrences_count: u32,
    pub first_occurrence_address: u32,
    pub abs_pos: usize,
}

pub fn parse_chunk_vari(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMVariable>, String> {
    chunk.cur_pos = 0;
    let _unknown1: u32 = chunk.read_u32()?;
    let _unknown2: u32 = chunk.read_u32()?;
    let _unknown3: u32 = chunk.read_u32()?;
    let file_len: usize = chunk.data.len();
    let mut variables: Vec<GMVariable> = vec![];

    // println!("{} {} {}", _unknown1, _unknown2, _unknown3);

    while chunk.cur_pos < file_len {
        let abs_pos: usize = chunk.cur_pos;
        let variable: GMVariable = GMVariable {
            name: chunk.read_gm_string(strings)?,
            instance_type: chunk.read_i32()?,
            variable_id: chunk.read_i32()?,
            occurrences_count: chunk.read_u32()?,
            first_occurrence_address: chunk.read_u32()?,
            abs_pos,
        };
        // println!(
        //     "[Variable]   {:<22} {:<10} {:<4} {:<6} {:<12}",
        //      variable.name,
        //      variable.variable_id,
        //      variable.instance_type,
        //      variable.occurrences_count,
        //      variable.first_occurrence_address as i32
        // );
        variables.push(variable);
    }

    Ok(variables)
}
