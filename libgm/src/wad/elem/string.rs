// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::util::assert;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

const ALIGNMENT: u32 = 4;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMStrings {
    pub strings: Vec<String>,
    pub align: bool,
    pub exists: bool,
}

gm_chunk!(STRG, GMStrings);
// gm_list_chunk!(STRG, GMStrings, String, strings, direct);

impl GMElement for GMStrings {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let align: bool = pointers.iter().all(|&p| p.is_multiple_of(ALIGNMENT));

        let mut strings: Vec<String> = Vec::with_capacity(pointers.len());

        for (i, pointer) in pointers.into_iter().enumerate() {
            if align {
                reader.align(ALIGNMENT)?;
            }
            reader.assert_pos(pointer, "String")?;
            reader
                .string_occurrences
                .insert(pointer + 4, GMRef::from(i));

            let string_length = reader.read_u32()?;
            let string: String = reader.read_literal_string(string_length)?;
            let byte = reader.read_u8()?;
            assert::int(byte, 0, "Null terminator byte after string")?;
            strings.push(string);
        }

        reader.align(0x80)?;
        Ok(Self { strings, align, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let count = self.strings.len();

        builder.write_usize(count)?;
        let pointer_list_pos = builder.pos();
        for _ in 0..count {
            builder.write_u32(0xDEAD_C0DE);
        }

        for (idx, string) in self.strings.iter().enumerate() {
            if self.align {
                builder.align(ALIGNMENT);
            }
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, idx)?;
            builder.write_u32(string.len() as u32);
            builder.resolve_pointer(string)?;
            builder.write_bytes(string.as_bytes());
            builder.write_u8(0);
        }

        builder.align(0x80);
        Ok(())
    }
}

impl GMStrings {
    pub fn find(&self, string: &str) -> Result<GMRef<String>> {
        for (gm_ref, str) in self.element_refs() {
            if str == string {
                return Ok(gm_ref);
            }
        }
        Err(err!("Could not find existing string {string:?}"))
    }

    #[inline]
    pub fn make(&mut self, string: &str) -> GMRef<String> {
        for (gm_ref, str) in self.element_refs() {
            if str == string {
                return gm_ref;
            }
        }
        self.make_new(string.to_owned())
    }

    #[inline]
    pub fn make_new(&mut self, string: String) -> GMRef<String> {
        self.strings.push(string);
        GMRef::from(self.len() - 1)
    }

    pub fn element_refs(&self) -> impl Iterator<Item = (GMRef<String>, &String)> {
        self.strings
            .iter()
            .enumerate()
            .map(|(idx, string)| (GMRef::from(idx), string))
    }

    pub fn element_refs_mut(&mut self) -> impl Iterator<Item = (GMRef<String>, &mut String)> {
        self.strings
            .iter_mut()
            .enumerate()
            .map(|(idx, string)| (GMRef::from(idx), string))
    }

    pub fn by_ref(&self, gm_ref: GMRef<String>) -> Result<&String> {
        gm_ref.resolve(&self.strings)
    }

    pub fn by_ref_mut(&mut self, gm_ref: GMRef<String>) -> Result<&mut String> {
        gm_ref.resolve_mut(&mut self.strings)
    }

    pub fn push(&mut self, string: String) {
        self.strings.push(string);
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.strings.len()
    }
}
