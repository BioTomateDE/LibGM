use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};


#[derive(Debug, Clone)]
pub struct GMStrings {
    pub abs_pos_to_reference: HashMap<usize, GMRef<String>>,  // convert absolute position/pointer in data.win to string ref
    pub strings_by_index: Vec<String>,                        // strings by index/order in chunk STRG
}


pub fn parse_chunk_strg(chunk: &mut GMChunk) -> Result<GMStrings, String> {
    chunk.cur_pos = 0;
    let string_count: usize = chunk.read_usize_count()?;
    let mut abs_start_positions: Vec<usize> = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        abs_start_positions.push(chunk.read_usize_pos()?);
    }

    let mut strings_by_index: Vec<String> = Vec::with_capacity(string_count);
    let mut abs_pos_to_reference: HashMap<usize, GMRef<String>> = HashMap::with_capacity(string_count);

    for (i, abs_start_position) in abs_start_positions.iter().enumerate() {
        chunk.cur_pos = *abs_start_position - chunk.abs_pos;
        let string_length: usize = chunk.read_usize_pos()?;
        let string: String = chunk.read_literal_string(string_length)?;
        strings_by_index.push(string.clone());
        // start_position + 4 because gamemaker moment
        // gamemaker does this because it's faster to access strings if you don't need to add or subtract 4 every time
        abs_pos_to_reference.insert(*abs_start_position + 4, GMRef::new(i));
    }

    Ok(GMStrings {
        abs_pos_to_reference,
        strings_by_index,
    })
}

