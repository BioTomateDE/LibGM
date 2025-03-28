use std::collections::HashMap;
use crate::deserialize::chunk_reading::UTChunk;

pub struct UTStrings {
    strings_by_absolute_position: HashMap<usize, String>,       // strings by absolute position/pointer in data.win
    strings_by_index: Vec<String>,                              // strings by index/order in chunk STRG
}

impl UTStrings {
    pub fn get_string_by_pos(&self, position: usize) -> Option<String> {
        match self.strings_by_absolute_position.get(&position) {
            Some(string) => Some(string.clone()),
            None => None
        }
    }

    pub fn get_string_by_index(&self, index: usize) -> Option<String> {
        match self.strings_by_index.get(index) {
            Some(ut_string) => Some(ut_string.clone()),
            None => None
        }
    }

    pub fn len(&self) -> usize {
        self.strings_by_index.len()
    }

}


pub fn parse_chunk_STRG(chunk: &mut UTChunk) -> Result<UTStrings, String> {
    chunk.file_index = 0;
    let string_count: usize = chunk.read_usize()?;
    // skip redundant list of absolute positions of upcoming strings
    chunk.file_index += string_count * 4;
    let mut strings_by_index: Vec<String> = Vec::with_capacity(string_count);
    let mut strings_by_absolute_position: HashMap<usize, String> = HashMap::new();

    for _ in 0..string_count {
        let string_length: usize = chunk.read_usize()?;
        let absolute_position: usize = chunk.abs_pos + chunk.file_index;
        let string: String = chunk.read_literal_string(string_length)?;
        chunk.file_index += 1;  // skip one byte for the null byte after the string
        strings_by_index.push(string.clone());
        strings_by_absolute_position.insert(absolute_position, string);
    }

    let strings: UTStrings = UTStrings { strings_by_absolute_position, strings_by_index };
    Ok(strings)
}


