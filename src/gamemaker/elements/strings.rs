use std::collections::HashMap;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;


const ALIGNMENT: usize = 4;

#[derive(Debug, Clone)]
pub struct GMStrings {
    pub strings: Vec<String>,
    pub is_aligned: bool,
    pub exists: bool,
}

impl GMChunkElement for GMStrings {
    fn stub() -> Self {
        Self { strings: vec![], is_aligned: true, exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMStrings {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let mut is_aligned: bool = true;
        let pointers: Vec<usize> = reader.read_simple_list()?;

        let mut strings_by_index: Vec<String> = Vec::with_capacity(pointers.len());
        let mut abs_pos_to_reference: HashMap<usize, GMRef<String>> = HashMap::with_capacity(pointers.len());

        for (i, pointer) in pointers.into_iter().enumerate() {
            if pointer % ALIGNMENT != 0 {
                is_aligned = false;
            }
            reader.cur_pos = pointer;
            if is_aligned {
                reader.align(ALIGNMENT)?;
            }
            let string_length: usize = reader.read_usize()?;
            let string: String = reader.read_literal_string(string_length)?;
            let byte: u8 = reader.read_u8()?;
            if byte != 0 {
                return Err(format!("Expected null terminator byte after string, found {byte} (0x{byte:02X})"))
            }
            strings_by_index.push(string.clone());
            // occurrence is start_position + 4 because yoyogames moment
            // gamemaker does this because it's faster to access strings if you don't need to add or subtract 4 every time
            abs_pos_to_reference.insert(pointer + 4, GMRef::new(i as u32));
        }

        reader.align(0x80)?;
        reader.string_occurrence_map = abs_pos_to_reference;
        Ok(GMStrings { strings: strings_by_index, is_aligned, exists: true })
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
            if self.is_aligned {
                builder.align(ALIGNMENT);
            }
            builder.overwrite_usize(builder.len(), pointer_list_start + 4*i)?;
            builder.write_usize(string.len())?;
            builder.resolve_pointer(string)?;   // gamemaker string references point to the actual string data
            builder.write_literal_string(string);
            builder.write_u8(0);    // trailing null terminator byte
        }

        builder.align(0x80);
        Ok(())
    }
}


impl GMRef<String> {
    /// Tries to resolve a GameMaker string reference to the actual character string.
    /// Returns a placeholder string if resolving failed.
    ///
    /// This function is meant to be used in closures where propagating errors is awkward.
    /// Otherwise, using [`GMRef::resolve`] is preferred.
    pub fn display<'a>(&self, gm_strings: &'a GMStrings) -> &'a str {
        self.resolve(&gm_strings.strings)
            .map(|i| i.as_str())
            .unwrap_or("<invalid string reference>")
    }
}

