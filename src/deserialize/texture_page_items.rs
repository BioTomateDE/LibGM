use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::embedded_textures::GMEmbeddedTexture;

#[derive(Debug, Clone, PartialEq)]
pub struct GMTexturePageItem {
    pub source_x: u16,
    pub source_y: u16,
    pub source_width: u16,
    pub source_height: u16,
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
    pub texture_page: GMRef<GMEmbeddedTexture>,
}

#[derive(Debug, Clone)]
pub struct GMTextures {
    pub abs_pos_to_ref: HashMap<usize, GMRef<GMTexturePageItem>>,     // convert absolute position/pointer in data.win to texture ref
    pub textures_by_index: Vec<GMTexturePageItem>,                    // texture page items by absolute position/pointer in data.win
}

pub fn parse_chunk_tpag(chunk: &mut GMChunk) -> Result<GMTextures, String> {
    chunk.cur_pos = 0;
    let items_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(items_count);
    for _ in 0..items_count {
        start_positions.push(chunk.read_relative_pointer()?);
    }

    let mut textures_by_index: Vec<GMTexturePageItem> = Vec::with_capacity(items_count);
    let mut abs_pos_to_ref: HashMap<usize, GMRef<GMTexturePageItem>> = HashMap::with_capacity(items_count);
    for (i, start_position) in start_positions.iter().enumerate() {
        chunk.cur_pos = *start_position;
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
        let texture_page_id: u16 = chunk.read_u16()?;
        let texture_page: GMRef<GMEmbeddedTexture> = GMRef::new(usize::from(texture_page_id));

        let texture_page_item: GMTexturePageItem = GMTexturePageItem {
            source_x,
            source_y,
            source_width,
            source_height,
            target_x,
            target_y,
            target_width,
            target_height,
            bounding_width,
            bounding_height,
            texture_page,
        };
        textures_by_index.push(texture_page_item);
        abs_pos_to_ref.insert(start_position + chunk.abs_pos, GMRef::new(i));
    }

    let textures: GMTextures = GMTextures { textures_by_index, abs_pos_to_ref };
    Ok(textures)
}

