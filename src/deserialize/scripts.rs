use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone)]
pub struct GMScript {
    pub name: GMRef<String>,
    pub id: Option<u32>,
}


#[derive(Debug, Clone)]
pub struct GMScripts {
    pub scripts_by_index: Vec<GMScript>,
    pub abs_pos_to_index: HashMap<usize, usize>,
}

pub fn parse_chunk_scpt(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMScripts, String> {
    chunk.file_index = 0;
    let script_count: usize = chunk.read_usize()?;

    let mut absolute_start_positions: Vec<usize> = Vec::with_capacity(script_count);
    for _ in 0..script_count {
        absolute_start_positions.push(chunk.read_usize()?);
    }

    let mut abs_pos_to_index: HashMap<usize, usize> = HashMap::new();
    let mut scripts_by_index: Vec<GMScript> = Vec::with_capacity(script_count);
    for (i, abs_start_position) in absolute_start_positions.iter().enumerate() {
        chunk.file_index = abs_start_position - chunk.abs_pos;
        let name: GMRef<String> = chunk.read_gm_string(&strings)?;
        let id: i32 = chunk.read_i32()?;
        if id < -1 {
            return Err(format!("Script with name {} has ID less than -1: {}", name.resolve(&strings.strings_by_index)?, id))
        }
        let id: Option<u32> = if id == -1 { None } else { Some(id as u32) };

        // println!("Script  {:<10?} {}", id, name.resolve(strings)?);
        scripts_by_index.push(GMScript { name, id });
        abs_pos_to_index.insert(*abs_start_position, i);
    }


    Ok(GMScripts { scripts_by_index, abs_pos_to_index })
}

