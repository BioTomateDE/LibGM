use std::collections::HashMap;
use crate::deserialize::chunk_reading::UTChunk;


#[derive(Debug, Clone)]
pub struct UTString {
    pub index: usize,
    pub id: u32,
    pub value: String,
}

pub struct UTStrings {
    pub strings_by_id: HashMap<u32, UTString>,
    pub strings_by_index: Vec<UTString>,
}

impl UTStrings {
    pub fn get_string_by_id(&self, string_id: u32) -> Option<String> {
        match self.strings_by_id.get(&string_id) {
            Some(ut_string) => Some(ut_string.value.clone()),
            None => None
        }
    }

    pub fn get_string_by_index(&self, index: usize) -> Option<String> {
        match self.strings_by_index.get(index) {
            Some(ut_string) => Some(ut_string.value.clone()),
            None => None
        }
    }

    pub fn len(&self) -> usize {
        self.strings_by_index.len()
    }

}


pub fn parse_chunk_STRG(mut chunk: UTChunk) -> Result<UTStrings, String> {
    let string_count: usize = chunk.read_usize()?;
    let mut string_ids: Vec<u32> = Vec::with_capacity(string_count);
    let mut strings_by_index: Vec<UTString> = Vec::with_capacity(string_count);
    let mut strings_by_id: HashMap<u32, UTString> = HashMap::new();

    for _ in 0..string_count {
        // you have to add 4 to the string id for some unknown reason
        let string_id = 4 + chunk.read_u32()?;
        string_ids.push(string_id);
    }

    for (i, string_id) in string_ids.iter().enumerate() {
        let string_length: usize = chunk.read_usize()?;
        let string: String = chunk.read_literal_string(string_length)?;
        chunk.file_index += 1;  // skip one byte for the null byte after the string
        let ut_string: UTString = UTString { index: i, id: *string_id, value: string };
        strings_by_index.push(ut_string.clone());
        strings_by_id.insert(*string_id, ut_string);
    }

    let strings: UTStrings = UTStrings { strings_by_id, strings_by_index };
    Ok(strings)
}


