use std::collections::HashMap;
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::embedded_textures::{Image, UTEmbeddedTexture};
use image;
use crate::printing::format_type_of;

#[derive(Debug, Clone)]
pub struct UTTexture {
    pub img: image::DynamicImage,
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
    pub index: usize,
}


pub struct UTTextures {
    textures_by_absolute_position: HashMap<usize, UTTexture>,       // texture page items by absolute position/pointer in data.win
}
impl UTTextures {
    pub fn get_texture_by_pos(&self, abs_pos: usize) -> Option<&UTTexture> {
        self.textures_by_absolute_position.get(&abs_pos)
    }
}


pub fn parse_chunk_TPAG(chunk: &mut UTChunk, texture_pages: Vec<UTEmbeddedTexture>) -> Result<UTTextures, String> {
    chunk.file_index = 0;
    let items_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(items_count);
    for _ in 0..items_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut texture_page_items: HashMap<usize, UTTexture> = HashMap::new();
    for (i, start_position) in start_positions.iter().enumerate() {
        chunk.file_index = *start_position;
        let source_x: u16 = chunk.read_u16()?;
        let source_y: u16 = chunk.read_u16()?;
        let source_width: u16 = chunk.read_u16()?;
        let source_height: u16 = chunk.read_u16()?;
        let target_x: u16 = chunk.read_u16()?;
        let target_y: u16 = chunk.read_u16()?;
        let target_width: u16 = chunk.read_u16()?;
        let target_height: u16 = chunk.read_u16()?;
        let bounding_width: u16 = chunk.read_u16()?;
        let bounding_height: u16 = chunk.read_u16()?;
        let texture_page_id: usize = chunk.read_u16()? as usize;

        let texture_page: &UTEmbeddedTexture = match texture_pages.get(texture_page_id) {
            Some(page) => page,
            None => return Err(format!(
                "Texture Page ID out ouf bounds at position {} in chunk 'TPAG': {} >= {}.",
                chunk.file_index, texture_page_id, texture_pages.len(),
            )),
        };
        let spritesheet: &image::RgbaImage = match &texture_page.texture_data {
            Image::Img(image::DynamicImage::ImageRgba8(img)) => img,
            _ => return Err(format!(
                "Unknown type of texture page image at position {} in chunk 'TPAG': {}.",
                chunk.file_index, format_type_of(texture_page),
            )),
        };

        let img = image::imageops::crop_imm(spritesheet, source_x as u32, source_y as u32, source_width as u32, source_height as u32).to_image();
        let texture_page_item: UTTexture = UTTexture {
            img: image::DynamicImage::ImageRgba8(img),
            target_x,
            target_y,
            target_width,
            target_height,
            bounding_width,
            bounding_height,
            index: i,
        };
        texture_page_items.insert(start_position + chunk.abs_pos, texture_page_item);
    }

    let textures: UTTextures = UTTextures {textures_by_absolute_position: texture_page_items};
    Ok(textures)
}

