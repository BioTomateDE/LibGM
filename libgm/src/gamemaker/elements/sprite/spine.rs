pub mod texture_entry;

pub use texture_entry::TextureEntry;

use crate::{gamemaker::deserialize::reader::DataReader, prelude::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
    pub version: i32,
    pub textures: Vec<TextureEntry>,
    pub json: String,
    pub atlas: String,
}

impl Data {
    fn decode_spine_blob(blob: &mut Vec<u8>) {
        // don't ask me, ask Nikita Krapivin (or don't)
        let mut k: u32 = 42;
        for byte in blob {
            // If this panics in debug profile, replace with wrapping operations
            *byte -= k as u8;
            k *= k + 1;
        }
    }

    fn encode_spine_blob(blob: &mut Vec<u8>) {
        // don't ask me, ask Nikita Krapivin (or don't)
        let mut k: u32 = 42;
        for byte in blob {
            // If this panics in debug profile, replace with wrapping operations
            *byte += k as u8;
            k *= k + 1;
        }
    }

    pub(super) fn read_weird_string(reader: &mut DataReader, size: u32) -> Result<String> {
        let mut blob: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
        Self::decode_spine_blob(&mut blob);
        let string: String = String::from_utf8(blob)
            .map_err(|e| e.to_string())
            .context("reading weird UTF-8 String for Spine data")?;
        Ok(string)
    }

    pub(super) fn build_weird_string(string: &String) -> Vec<u8> {
        let mut blob: Vec<u8> = string.as_bytes().to_vec();
        Self::encode_spine_blob(&mut blob);
        blob
    }
}
