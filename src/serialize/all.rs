use std::collections::HashMap;
use std::fs;
use std::path::Path;
use image::DynamicImage;
use crate::deserialize::all::GMData;
use crate::deserialize::texture_page_items::{GMTexturePageItem};
use crate::serialize::backgrounds::build_chunk_bgnd;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};
use crate::serialize::embedded_audio::build_chunk_audo;
use crate::serialize::embedded_textures::build_chunk_txtr;
use crate::serialize::fonts::build_chunk_font;
use crate::serialize::functions::build_chunk_func;
use crate::serialize::game_objects::build_chunk_objt;
use crate::serialize::strings::build_chunk_strg;
use crate::serialize::general_info::{build_chunk_optn, build_chunk_gen8};
use crate::serialize::paths::build_chunk_path;
use crate::serialize::rooms::build_chunk_room;
use crate::serialize::scripts::build_chunk_scpt;
use crate::serialize::sounds::build_chunk_sond;
use crate::serialize::sprites::build_chunk_sprt;
use crate::serialize::stubs::{build_chunk_agrp, build_chunk_code, build_chunk_dafl, build_chunk_extn, build_chunk_shdr, build_chunk_tmln};
use crate::serialize::texture_page_items::{build_chunk_tpag, generate_texture_pages};
use crate::serialize::variables::build_chunk_vari;

#[derive(Debug, Clone)]
pub struct DataBuilder {
    raw_data: Vec<u8>,
    pointer_pool_placeholders: HashMap<GMPointer, usize>,  // maps gamemaker element references to absolute positions of where they're referenced
    pointer_pool_resources: HashMap<GMPointer, usize>,     // maps gamemaker element references to absolute positions of where their data is
}
impl DataBuilder {
    /// Create a placeholder pointer at the current position in the chunk
    /// and store the target gamemaker element (reference) in the pool.
    /// This will later be resolved; replacing the placeholder pointer with
    /// the absolute position of the target data in the data file
    /// (assuming the pointer origin position was added to the pool).
    /// This method should be called, when the data file format expects
    /// a pointer to some element, but you don't yet (necessarily) know where
    /// that element will be located in the data file.
    pub fn push_pointer_placeholder(&mut self, chunk_builder: &mut ChunkBuilder, pointer: GMPointer) -> Result<(), String> {
        let position: usize = self.len() + chunk_builder.len();
        chunk_builder.write_usize(0);      // write placeholder
        self.pointer_pool_placeholders.insert(pointer, position);
        Ok(())
    }

    /// Store the gamemaker element's absolute position in the pool.
    /// The element's absolute position is the chunk builder's current position,
    /// since this method should get called when the element is built to the data file.
    pub fn push_pointer_resolve(&mut self, chunk_builder: &mut ChunkBuilder, pointer: GMPointer) -> Result<(), String> {
        let position: usize = chunk_builder.abs_pos + chunk_builder.len();
        if let Some(old_value) = self.pointer_pool_resources.insert(pointer.clone(), position) {
            return Err(format!("Pointer to {:?} already resolved to absolute position {}; \
            tried to resolve again to position {}.", pointer, old_value, position))
        }
        Ok(())
    }
}

impl DataBuilder {
    pub fn write_usize(&mut self, number: usize) {
        let bytes = (number as u32).to_le_bytes();
        self.raw_data.extend_from_slice(&bytes);
    }
    pub fn write_chunk_name(&mut self, string: &str) -> Result<(), String> {
        // write a 4 character ascii string to the data
        for (i, char) in string.chars().enumerate() {
            let byte: u8 = char.try_into().map_err(|e| format!(
                "Char Typecasting error while writing chunk name \"{string}\" \
                (i: {i}) to data (len: {}): {e}", self.len()))?;
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
    let mut builder: DataBuilder = DataBuilder { raw_data: Vec::new(), pointer_pool_placeholders: HashMap::new(), pointer_pool_resources: HashMap::new() };

    let (texture_page_items, texture_pages): (Vec<GMTexturePageItem>, Vec<DynamicImage>) = generate_texture_pages(&gm_data.textures)?;

    // write placeholder u32 for total length
    builder.write_chunk_name("FORM")?;
    builder.write_usize(0);

    build_chunk_gen8(&mut builder, &gm_data)?;
    build_chunk_optn(&mut builder, &gm_data)?;
    build_chunk_extn(&mut builder, &gm_data)?;      // stub
    build_chunk_sond(&mut builder, &gm_data)?;
    build_chunk_agrp(&mut builder, &gm_data)?;      // stub
    build_chunk_sprt(&mut builder, &gm_data)?;
    build_chunk_bgnd(&mut builder, &gm_data)?;
    build_chunk_path(&mut builder, &gm_data)?;
    build_chunk_scpt(&mut builder, &gm_data)?;
    build_chunk_shdr(&mut builder, &gm_data)?;      // stub
    build_chunk_font(&mut builder, &gm_data)?;
    build_chunk_tmln(&mut builder, &gm_data)?;      // stub
    build_chunk_objt(&mut builder, &gm_data)?;
    build_chunk_room(&mut builder, &gm_data)?;
    build_chunk_dafl(&mut builder, &gm_data)?;      // stub
    build_chunk_tpag(&mut builder, &gm_data, texture_page_items)?;
    build_chunk_code(&mut builder, &gm_data)?;      // stub
    build_chunk_vari(&mut builder, &gm_data)?;
    build_chunk_func(&mut builder, &gm_data)?;
    build_chunk_strg(&mut builder, &gm_data)?;
    build_chunk_txtr(&mut builder, &gm_data, texture_pages)?;
    build_chunk_audo(&mut builder, &gm_data)?;

    // {~~} IMPORTANT TODO: resolve pointers

    let bytes: [u8; 4] = (builder.len() as u32).to_le_bytes();
    builder.overwrite_data(&bytes, 4)?;     // overwrite placeholder total length

    Ok(builder.raw_data)
}


pub fn write_data_file(data_file_path: &Path, raw_data: &[u8]) -> Result<(), String> {
    fs::write(data_file_path, raw_data)
        .map_err(|e| format!("Could not write data file to location \"{}\": {e}", data_file_path.display()))
}


pub fn build_chunk(data_builder: &mut DataBuilder, chunk_builder: ChunkBuilder) -> Result<(), String> {
    data_builder.write_chunk_name(chunk_builder.chunk_name)?;
    data_builder.write_usize(chunk_builder.len());
    data_builder.raw_data.extend(chunk_builder.raw_data);
    Ok(())
}

