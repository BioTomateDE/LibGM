use image::{ImageBuffer, Rgba};
use crate::debug_utils::DurationExt;
use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::texture_page_items::{GMTexture, GMTexturePageItem, GMTextures};
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_tpag(builder: &mut DataBuilder, gm_data: &GMData, texture_page_items: Vec<GMTexturePageItem>) -> Result<(), String> {
    builder.start_chunk("TPAG")?;
    let len: usize = texture_page_items.len();
    builder.write_usize(len);

    for texture_page_item in &texture_page_items {
        builder.write_placeholder(GMPointer::Texture(texture_page_item.texture.index))?;
    }

    for texture_page_item in &texture_page_items {
        builder.resolve_pointer(GMPointer::Texture(texture_page_item.texture.index))?;
        let texture: &GMTexture = texture_page_item.texture.resolve(&gm_data.textures.textures_by_index)?;

        builder.write_u16(texture_page_item.source_x);
        builder.write_u16(texture_page_item.source_y);
        builder.write_u16(texture_page_item.source_width);
        builder.write_u16(texture_page_item.source_height);
        builder.write_u16(texture.target_x);
        builder.write_u16(texture.target_y);
        builder.write_u16(texture.target_width);
        builder.write_u16(texture.target_height);
        builder.write_u16(texture.bounding_width);
        builder.write_u16(texture.bounding_height);
        builder.write_u16(texture_page_item.texture_page_id);
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


// note: in undertale, the dimensions of texture pages are all powers of 2 (512, 1024, 2048)
//       i don't really know if this is important (i'm ignoring it for now)
pub fn generate_texture_pages(gm_textures: &GMTextures) -> Result<(Vec<GMTexturePageItem>, Vec<image::DynamicImage>), String> {
    static PAGE_MAX_WIDTH: usize = 2048;        // MAX: u16 limit (65535)
    static PAGE_MAX_HEIGHT: usize = 2048;       // MAX: u16 limit (65535)

    let texture_count: usize = gm_textures.textures_by_index.len();
    let mut textures: Vec<(usize, &GMTexture)> = Vec::with_capacity(texture_count);
    for i in 0..texture_count {
        let texture: &GMTexture = &gm_textures.textures_by_index[i];
        textures.push((i, texture));
    }
    // sort textures by height; ascending order
    let tstart = ::cpu_time::ProcessTime::now();
    textures.sort_by(|(_, a), (_, b)| a.img.height().cmp(&b.img.height()));
    log::trace!("Sorting textures by height took {}", tstart.elapsed().ms());

    let mut texture_pages: Vec<image::DynamicImage> = Vec::new();
    let mut texture_page_items: Vec<GMTexturePageItem> = Vec::with_capacity(texture_count);

    // place rows; left to right
    let mut cur_texture_page: ImageBuffer<Rgba<u8>, Vec<u8>> = image::RgbaImage::new(2028, 2048);
    let mut x: usize = 0;
    let mut y: usize = 0;

    for (index, texture) in textures {
        if x >= PAGE_MAX_WIDTH {
            x = 0;
            // since they're in ascending order, this image's height will be (at least) as much as the 'tallest' image from the last row
            y += texture.img.height() as usize;
        }
        if y >= PAGE_MAX_HEIGHT {
            texture_pages.push(image::DynamicImage::ImageRgba8(cur_texture_page));
            cur_texture_page = image::RgbaImage::new(2028, 2048);
            x = 0;
            y = 0;
        }

        texture_page_items.push(GMTexturePageItem {
            source_x: x as u16,
            source_y: y as u16,
            source_width: texture.img.width() as u16,
            source_height: texture.img.height() as u16,
            texture_page_id: texture_pages.len() as u16,
            texture: GMRef::new(index),
        });
        image::imageops::overlay(&mut cur_texture_page, &texture.img, x as i64, y as i64);
        x += texture.img.width() as usize;
    }

    // push last texture page
    texture_pages.push(image::DynamicImage::ImageRgba8(cur_texture_page));

    Ok((texture_page_items, texture_pages))
}

