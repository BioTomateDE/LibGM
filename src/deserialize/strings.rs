use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};


#[derive(Debug, Clone)]
pub struct GMStrings {
    pub abs_pos_to_reference: HashMap<usize, GMRef<String>>,  // convert absolute position/pointer in data.win to string ref
    pub strings_by_index: Vec<String>,                        // strings by index/order in chunk STRG
}


pub fn parse_chunk_strg(chunk: &mut GMChunk) -> Result<GMStrings, String> {
    chunk.cur_pos = 0;
    let string_count: usize = chunk.read_usize()?;
    // skip redundant list of absolute positions of upcoming strings
    chunk.cur_pos += string_count * 4;
    let mut strings_by_index: Vec<String> = Vec::with_capacity(string_count);
    let mut abs_pos_to_reference: HashMap<usize, GMRef<String>> = HashMap::new();

    for i in 0..string_count {
        let string_length: usize = chunk.read_usize()?;
        let absolute_position: usize = chunk.abs_pos + chunk.cur_pos;
        let string: String = chunk.read_literal_string(string_length)?;
        chunk.cur_pos += 1;  // skip one byte for the null byte after the string
        strings_by_index.push(string.clone());
        abs_pos_to_reference.insert(absolute_position, GMRef::string(i));
    }

    let strings: GMStrings = GMStrings {
        abs_pos_to_reference,
        strings_by_index,
    };
    Ok(strings)
}

