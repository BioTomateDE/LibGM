use std::collections::HashMap;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::embedded_textures::{Image, GMEmbeddedTexture};
use image;
use crate::printing::format_type_of;

#[derive(Debug, Clone)]
pub struct GMTexture {
    pub img: image::DynamicImage,
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GMTextureRef {
    pub index: usize,
}
impl GMTextureRef {
    pub fn resolve<'a>(&self, textures: &'a GMTextures) -> Result<&'a GMTexture, String> {
        match textures.textures_by_index.get(self.index) {
            Some(texture) => Ok(texture),
            None => Err(format!(
                "Could not resolve Texture Page Item with index {} in list with length {}.",
                self.index, textures.textures_by_index.len()
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GMTextures {
    abs_pos_to_index: HashMap<usize, usize>,    // convert absolute position/pointer in data.win to index in Self.textures_by_index
    textures_by_index: Vec<GMTexture>,          // texture page items by absolute position/pointer in data.win
}
impl GMTextures {
    pub fn get_texture_by_pos(&self, abs_pos: usize) -> Option<GMTextureRef> {
        let index: usize = match self.abs_pos_to_index.get(&abs_pos) {
            Some(index) => *index,
            None => return None,
        };
        Some(GMTextureRef { index })
    }
    pub fn get_texture_by_index(&self, index: usize) -> Option<GMTextureRef> {
        if index >= self.textures_by_index.len() {
            return None;
        }
        Some(GMTextureRef {index})
    }
    pub fn len(&self) -> usize {
        self.textures_by_index.len()
    }
}


#[derive(Debug, Clone)]
pub struct GMTexturePageItem {
    pub source_x: u16,
    pub source_y: u16,
    pub source_width: u16,
    pub source_height: u16,
    pub texture_page_id: u16,
    pub texture: GMTextureRef,
}


#[allow(non_snake_case)]
pub fn parse_chunk_TPAG(chunk: &mut GMChunk, texture_pages: Vec<GMEmbeddedTexture>) -> Result<GMTextures, String> {
    chunk.file_index = 0;
    let items_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(items_count);
    for _ in 0..items_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut textures_by_index: Vec<GMTexture> = Vec::with_capacity(items_count);
    let mut abs_pos_to_index: HashMap<usize, usize> = HashMap::new();
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

        let texture_page: &GMEmbeddedTexture = match texture_pages.get(texture_page_id) {
            Some(page) => page,
            None => return Err(format!(
                "Texture Page ID out ouf bounds at position {} in chunk 'TPAG': {} >= {}.",
                chunk.file_index, texture_page_id, texture_pages.len(),
            )),
        };
        let spritesheet: &image::RgbaImage = match &texture_page.texture_data {
            Image::Img(image::DynamicImage::ImageRgba8(img)) => &img,
            _ => return Err(format!(
                "Unknown type of texture page image at position {} in chunk 'TPAG': {}.",
                chunk.file_index, format_type_of(texture_page),
            )),
        };

        // untested code
        let img = image::imageops::crop_imm(spritesheet, source_x as u32, source_y as u32, source_width as u32, source_height as u32).to_image();
        let texture_page_item: GMTexture = GMTexture {
            img: image::DynamicImage::ImageRgba8(img),
            target_x,
            target_y,
            target_width,
            target_height,
            bounding_width,
            bounding_height,
        };
        textures_by_index.push(texture_page_item);
        abs_pos_to_index.insert(start_position + chunk.abs_pos, i);
    }

    let textures: GMTextures = GMTextures { textures_by_index, abs_pos_to_index };
    Ok(textures)
}

