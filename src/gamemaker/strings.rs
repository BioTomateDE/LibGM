use std::collections::HashMap;
use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMPointer, GMRef};
use crate::gm_serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMStrings {
    pub strings: Vec<String>,
    pub exists: bool,
}
impl GMChunkElement for GMStrings {
    fn empty() -> Self {
        Self { strings: vec![], exists: false }
    }
}
impl GMElement for GMStrings {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let start_positions: Vec<GMPointer> = reader.read_simple_list()?;

        let mut strings_by_index: Vec<String> = Vec::with_capacity(start_positions.len());
        let mut abs_pos_to_reference: HashMap<usize, GMRef<String>> = HashMap::with_capacity(start_positions.len());

        for (i, pointer) in start_positions.into_iter().enumerate() {
            reader.cur_pos = pointer.pointing_to_position;
            let string_length: usize = reader.read_usize()?;
            let string: String = reader.read_literal_string(string_length)?;
            strings_by_index.push(string.clone());
            // occurrence is start_position + 4 because yoyogames moment
            // gamemaker does this because it's faster to access strings if you don't need to add or subtract 4 every time
            abs_pos_to_reference.insert(pointer.pointing_to_position + 4, GMRef::new(i as u32));
        }

        reader.string_occurrence_map = abs_pos_to_reference;
        
        Ok(GMStrings { strings: strings_by_index, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists {
            return Err("Required chunk STRG does not exist".to_string())
        }
        builder.write_usize(self.strings.len())?;
        let pointer_list_start: usize = builder.len();
        for _ in 0..self.strings.len() {
            builder.write_u32(0xDEADC0DE);
        }
        for (i, string) in self.strings.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_start + 4*i)?;
            builder.write_usize(string.len())?;
            builder.resolve_pointer(string)?;   // gamemaker string references point to the actual string data
            builder.write_literal_string(string);
            builder.write_u8(0);    // trailing null byte
        }
        Ok(())
    }
}


impl GMRef<String> {
    pub fn display<'a>(&self, gm_strings: &'a GMStrings) -> &'a str {
        self.resolve(&gm_strings.strings)
            .map(|i| i.as_str())
            .unwrap_or("<invalid string reference>")
    }
}

