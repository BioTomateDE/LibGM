use std::collections::HashMap;
use crate::gm_deserialize::{DataReader, GMChunk, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::embedded_textures::GMEmbeddedTexture;


#[derive(Debug, Clone)]
pub struct GMTexturePageItems {
    pub texture_page_items: Vec<GMTexturePageItem>,
    pub exists: bool,
}
impl GMChunkElement for GMTexturePageItems {
    fn empty() -> Self {
        Self { texture_page_items: vec![], exists: false }
    }
}
impl GMElement for GMTexturePageItems {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let texture_page_items: Vec<GMTexturePageItem> = reader.read_texture_page_items_with_occurrences()?;
        Ok(Self { texture_page_items, exists: true })
    }
}


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
impl GMElement for GMTexturePageItem {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let source_x: u16 = reader.read_u16()?;
        let source_y: u16 = reader.read_u16()?;
        let source_width: u16 = reader.read_u16()?;
        let source_height: u16 = reader.read_u16()?;
        let target_x: u16 = reader.read_u16()?;
        let target_y: u16 = reader.read_u16()?;
        let target_width: u16 = reader.read_u16()?;
        let target_height: u16 = reader.read_u16()?;
        let bounding_width: u16 = reader.read_u16()?;
        let bounding_height: u16 = reader.read_u16()?;
        let texture_page_id: u16 = reader.read_u16()?;
        let texture_page: GMRef<GMEmbeddedTexture> = GMRef::new(texture_page_id.into());

        Ok(GMTexturePageItem {
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
        })
    }
}

