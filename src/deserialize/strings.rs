use std::collections::HashMap;
use crate::deserialize::chunk_reading::GMChunk;


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GMStringRef {
    pub index: usize,
}

impl GMStringRef {
    pub fn resolve<'a>(&self, strings: &'a GMStrings) -> Result<&'a str, String> {
        match strings.strings_by_index.get(self.index) {
            Some(string) => Ok(string),
            None => Err(format!(
                "Could not resolve string with index {} in list with length {}.",
                self.index, strings.strings_by_index.len()
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GMStrings {
    abs_pos_to_index: HashMap<usize, usize>,    // convert absolute position/pointer in data.win to index in Self.strings_by_index
    strings_by_index: Vec<String>,              // strings by index/order in chunk STRG
}

impl GMStrings {
    pub fn get_string_by_pos(&self, position: usize) -> Option<GMStringRef> {
        let index: usize = match self.abs_pos_to_index.get(&position) {
            Some(index) => *index,
            None => return None,
        };
        Some(GMStringRef { index })
    }

    pub fn get_string_by_index(&self, index: usize) -> Option<GMStringRef> {
        if index >= self.strings_by_index.len() {
            return None;
        }
        Some(GMStringRef { index })
    }

    pub fn len(&self) -> usize { self.strings_by_index.len() }
}


pub fn parse_chunk_strg(chunk: &mut GMChunk) -> Result<GMStrings, String> {
    chunk.file_index = 0;
    let string_count: usize = chunk.read_usize()?;
    // skip redundant list of absolute positions of upcoming strings
    chunk.file_index += string_count * 4;
    let mut strings_by_index: Vec<String> = Vec::with_capacity(string_count);
    let mut abs_pos_to_index: HashMap<usize, usize> = HashMap::new();

    for i in 0..string_count {
        let string_length: usize = chunk.read_usize()?;
        let absolute_position: usize = chunk.abs_pos + chunk.file_index;
        let string: String = chunk.read_literal_string(string_length)?;
        chunk.file_index += 1;  // skip one byte for the null byte after the string
        strings_by_index.push(string.clone());
        abs_pos_to_index.insert(absolute_position, i);
    }

    let strings: GMStrings = GMStrings {
        abs_pos_to_index,
        strings_by_index,
    };
    Ok(strings)
}


