use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use crate::util::assert::assert_int;
use std::collections::HashMap;

const ALIGNMENT: u32 = 4;

#[derive(Debug, Clone)]
pub struct GMStrings {
    pub strings: Vec<String>,
    pub is_aligned: bool,
    pub exists: bool,
}

impl Default for GMStrings {
    fn default() -> Self {
        Self {
            strings: vec![],
            // Align by default for compatibility
            is_aligned: true,
            exists: false,
        }
    }
}

impl GMChunkElement for GMStrings {
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMStrings {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let is_aligned: bool = pointers.iter().all(|&p| p % ALIGNMENT == 0);

        let mut strings_by_index: Vec<String> = Vec::with_capacity(pointers.len());
        let mut abs_pos_to_reference: HashMap<u32, GMRef<String>> = HashMap::with_capacity(pointers.len());

        for (i, pointer) in pointers.into_iter().enumerate() {
            reader.cur_pos = pointer;
            if is_aligned {
                reader.align(ALIGNMENT)?;
            }
            let string_length = reader.read_u32()?;
            let string: String = reader.read_literal_string(string_length)?;
            let byte = reader.read_u8()?;
            assert_int("Null terminator byte after string", 0, byte)?;
            strings_by_index.push(string.clone());
            // Occurrence is `start_position + 4` because string refs point to the actual
            // String data instead of the gamemaker element for faster access.
            abs_pos_to_reference.insert(pointer + 4, GMRef::new(i as u32));
        }

        reader.align(0x80)?;
        reader.string_occurrences = abs_pos_to_reference;
        Ok(GMStrings { strings: strings_by_index, is_aligned, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if !self.exists {
            bail!("Required chunk STRG does not exist");
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
            builder.overwrite_usize(builder.len(), pointer_list_start + 4 * i)?;
            builder.write_usize(string.len())?;
            builder.resolve_pointer(string)?; // Gamemaker string references point to the actual string data
            builder.write_literal_string(string);
            builder.write_u8(0); // Trailing null terminator byte
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
