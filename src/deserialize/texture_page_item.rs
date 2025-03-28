use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::embedded_textures::{Image, UTEmbeddedTexture};
use image;

pub struct UTTexture {
    pub img: image::DynamicImage,
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
}


pub fn parse_chunk_TPAG(mut chunk: UTChunk, texture_pages: Vec<UTEmbeddedTexture>) -> Result<Vec<UTTexture>, String> {
    let items_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(items_count);
    for _ in 0..items_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut texture_page_items: Vec<UTTexture> = Vec::with_capacity(items_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
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
        };
        texture_page_items.push(texture_page_item);
    }

    Ok(texture_page_items)
}


fn format_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

