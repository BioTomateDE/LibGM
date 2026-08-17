// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::util::assert;
use crate::wad::GMVersion;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Strings {
    pub elems: Vec<String>,
}

gm_list_chunk!(STRG, Strings, String);

impl GMElement for Strings {
    // be sure not to use `reader.general_info` here, as it is not initialized yet!
    // this includes methods like `deserialize_if_gm_version`.
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;

        let mut elems: Vec<String> = Vec::with_capacity(pointers.len());

        for (i, pointer) in pointers.into_iter().enumerate() {
            if reader.version >= GMVersion::GMS2_3 {
                reader.align(4)?;
            }
            reader.assert_pos(pointer, "String")?;
            let gmref = GMRef::from(i);
            reader.string_occurrences.insert(pointer + 4, gmref);

            let string_length = reader.read_u32()?;
            let string: String = reader.read_literal_string(string_length)?;
            let byte = reader.read_u8().ctx("reading null-terminator byte")?;
            assert::int(byte, 0, "Null terminator byte after string")?;
            elems.push(string);
        }

        reader.align(0x80)?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        let count = self.elems.len();

        writer.write_usize(count)?;
        let pointer_list_pos = writer.pos();
        for _ in 0..count {
            writer.write_u32(0xDEAD_C0DE);
        }

        for (idx, string) in self.elems.iter().enumerate() {
            if writer.version() >= GMVersion::GMS2_3 {
                writer.align(4);
            }
            writer.patch_pointer_list(pointer_list_pos, idx)?;
            writer.write_u32(string.len() as u32);
            writer.resolve_pointer(string)?;
            writer.write_bytes(string.as_bytes());
            writer.write_u8(0);
        }

        writer.align(0x80);
        Ok(())
    }
}

impl Strings {
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
        self.elems.push(string);
        GMRef::from(self.len() - 1)
    }
}
