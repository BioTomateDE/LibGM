use std::collections::HashMap;
use std::fs;
use image::DynamicImage;
use crate::deserialize::all::GMData;
use crate::deserialize::backgrounds::GMBackgroundRef;
use crate::deserialize::embedded_audio::GMEmbeddedAudioRef;
use crate::deserialize::fonts::GMFontRef;
use crate::deserialize::functions::GMFunctionRef;
use crate::deserialize::game_objects::GMGameObjectRef;
use crate::deserialize::scripts::GMScriptRef;
use crate::deserialize::sounds::GMSoundRef;
use crate::deserialize::sprites::GMSpriteRef;
use crate::deserialize::strings::GMStringRef;
use crate::deserialize::texture_page_items::{GMTexturePageItem, GMTextureRef};
use crate::serialize::chunk_writing::ChunkBuilder;
use crate::serialize::embedded_textures::build_chunk_TXTR;
use crate::serialize::strings::build_chunk_STRG;
use crate::serialize::general_info::{build_chunk_OPTN, build_chunk_GEN8};
use crate::serialize::scripts::build_chunk_SCPT;
use crate::serialize::sounds::build_chunk_SOND;
use crate::serialize::texture_page_items::{build_chunk_TPAG, generate_texture_pages};

#[derive(Debug, Clone)]
pub struct DataBuilder {
    raw_data: Vec<u8>,
    pointer_pool: HashMap<GMRef, GMPointer>,
}
impl DataBuilder {
    pub fn push_pointer_position(&mut self, chunk_builder: &mut ChunkBuilder, reference: GMRef) -> Result<(), String> {
        let position: usize = self.len() + chunk_builder.len();
        chunk_builder.write_usize(0)?;      // placeholder
        let pointer: GMPointer = GMPointer {
            position,
            pointing_to: None,
        };
        self.pointer_pool.insert(reference, pointer);
        Ok(())
    }
    pub fn push_pointer_position_maybe(&mut self, chunk_builder: &mut ChunkBuilder, reference: Option<GMRef>) -> Result<(), String> {
        match reference {
            Some(reference) => self.push_pointer_position(chunk_builder, reference),
            None => chunk_builder.write_i32(-1),
        }
    }
    pub fn push_pointing_to(&mut self, chunk_builder: &mut ChunkBuilder, reference: GMRef) -> Result<(), String> {
        let pointer = match self.pointer_pool.get_mut(&reference) {
            Some(ptr) => ptr,
            None => return Err(format!(
                "Pointer with absolute position {} doesn't exist in pool (len: {}) with reference {:?}.",
                chunk_builder.abs_pos + chunk_builder.len(), self.pointer_pool.len(), reference,
            )),
        };

        pointer.pointing_to = Some(chunk_builder.abs_pos + chunk_builder.len());
        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GMRef {
    String(GMStringRef),
    Texture(GMTextureRef),
    Audio(GMEmbeddedAudioRef),
    Sound(GMSoundRef),
    Sprite(GMSpriteRef),
    Function(GMFunctionRef),
    Background(GMBackgroundRef),
    GameObject(GMGameObjectRef),
    Font(GMFontRef),
    Script(GMScriptRef),
    TexturePage(usize),
    TexturePageData(usize),
}
#[derive(Debug, Clone)]
pub struct GMPointer {
    position: usize,
    pointing_to: Option<usize>,
}

impl DataBuilder {
    pub fn write_usize(&mut self, number: usize) -> Result<(), String> {
        for byte in (number as u32).to_le_bytes() {
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn write_chunk_name(&mut self, string: &str) -> Result<(), String> {
        // write a 4 character ascii string to the data
        for (i, char) in string.chars().enumerate() {
            let byte: u8 = match char.try_into() {
                Ok(byte) => byte,
                Err(_) => return Err(format!("Char Typecasting error while writing chunk name \"{string}\" (i: {i}) to data (len: {})", self.len())),
            };
            self.raw_data.push(byte);
        }
        Ok(())
    }
    pub fn overwrite_data(&mut self, data: &[u8], position: usize) -> Result<(), String> {
        if position + data.len() >= self.len() {
            return Err(format!(
                "Could not overwrite {} bytes at position {} in data with length {} while building data.",
                data.len(),
                position,
                self.len()
            ))
        };
        for (i, byte) in data.iter().enumerate() {
            self.raw_data[position + i] = *byte;
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.raw_data.len()
    }
}


pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>, String> {
    let mut builder: DataBuilder = DataBuilder { raw_data: Vec::new(), pointer_pool: HashMap::new() };

    // write placeholder u32 for total length
    builder.write_chunk_name("FORM")?;
    builder.write_usize(0)?;

    build_chunk_GEN8(&mut builder, &gm_data)?;
    build_chunk_OPTN(&mut builder, &gm_data)?;
    build_chunk_STRG(&mut builder, &gm_data)?;
    build_chunk_SOND(&mut builder, &gm_data)?;
    build_chunk_SCPT(&mut builder, &gm_data)?;
    let (texture_page_items, texture_pages): (Vec<GMTexturePageItem>, Vec<DynamicImage>) = generate_texture_pages(&gm_data.textures)?;
    build_chunk_TPAG(&mut builder, &gm_data, texture_page_items)?;
    build_chunk_TXTR(&mut builder, &gm_data, texture_pages)?;

    // {~~} IMPORTANT TODO: resolve pointers

    let bytes: [u8; 4] = (builder.len() as u32).to_le_bytes();
    builder.overwrite_data(&bytes, 4)?;     // overwrite placeholder total length

    Ok(builder.raw_data)
}


pub fn write_data_file(data_file_path: &str, raw_data: &[u8]) -> Result<(), String> {
    match fs::write(data_file_path, raw_data) {
        Ok(_) => Ok(()),
        Err(error) => {
            Err(format!("Could not write to data file: {error}"))
        }
    }
}


pub fn build_chunk(data_builder: &mut DataBuilder, chunk_builder: ChunkBuilder) -> Result<(), String> {
    data_builder.write_chunk_name(chunk_builder.chunk_name)?;
    data_builder.write_usize(chunk_builder.len())?;
    data_builder.raw_data.extend(chunk_builder.raw_data);
    Ok(())
}

