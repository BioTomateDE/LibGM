use rayon::iter::ParallelIterator;
use std::io::Write;
use std::sync::Mutex;
use image::DynamicImage;
use rayon::prelude::IntoParallelIterator;
use crate::deserialize::all::GMData;
use crate::deserialize::embedded_textures::{GMEmbeddedTexture, MAGIC_BZ2_QOI_HEADER};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};


struct ImageData {
    compressed_data: Vec<u8>,
    uncompressed_size: usize,
    width: u32,
    height: u32,
}


pub fn build_chunk_txtr(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("TXTR")?;
    let len: usize = gm_data.texture_pages.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(GMPointer::TexturePage(i))?;
    }

    for (i, texture_page) in gm_data.texture_pages.iter().enumerate() {
        builder.resolve_pointer(GMPointer::TexturePage(i))?;
        build_texture_page(builder, &gm_data.general_info, i, texture_page)
            .map_err(|e| format!("{e} for texture page #{i} and \"index in group\" #{:?}", texture_page.index_in_group))?;
    }
    
    let texture_page_images_compressed: Vec<ImageData> = render_image_bz2_qoi(gm_data.texture_pages.iter().filter_map(|i| i.image.as_ref()).collect())?;
    for (i, image_data) in texture_page_images_compressed.iter().enumerate() { 
        build_texture_page_image(builder, &gm_data.general_info, i, image_data)?;
    }

    // nobody knows if this actually correct
    while builder.len() % 4 != 0 {
        builder.write_u8(0);
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

fn build_texture_page(builder: &mut DataBuilder, general_info: &GMGeneralInfo, index: usize, texture_page: &GMEmbeddedTexture) -> Result<(), String> {
    builder.write_u32(texture_page.scaled);
    if general_info.is_version_at_least(2, 0, 6, 0) {
        builder.write_u32(texture_page.generated_mips.ok_or("Generated mipmap levels not set")?);
    }
    if general_info.is_version_at_least(2022, 3, 0, 0) && texture_page.image.is_some() {
        builder.write_placeholder(GMPointer::TexturePageDataSize(index))?;
    }
    if general_info.is_version_at_least(2022, 9, 0, 0) {
        builder.write_i32(texture_page.texture_width.ok_or("Texture width not set")?);
        builder.write_i32(texture_page.texture_height.ok_or("Texture height not set")?);
        builder.write_usize(index);     // TODO not sure what "index in group" means. maybe this is not just the index?
    }
    if texture_page.image.is_some() {
        builder.write_placeholder(GMPointer::TexturePageData(index))?;
    } else {
        builder.write_usize(0);
    }
    Ok(())
}

fn build_texture_page_image(builder: &mut DataBuilder, general_info: &GMGeneralInfo, index: usize, image_data: &ImageData) -> Result<(), String> {
    // padding
    while builder.len() % 0x80 != 0 {
        builder.write_u8(0);
    }
    
    builder.resolve_pointer(GMPointer::TexturePageData(index))?;

    builder.write_bytes(MAGIC_BZ2_QOI_HEADER);
    builder.write_u16(image_data.width as u16);
    builder.write_u16(image_data.height as u16);
    
    if general_info.is_version_at_least(2022, 5, 0, 0) {
        builder.write_usize(image_data.uncompressed_size);
    }
    builder.write_bytes(&image_data.compressed_data);
    
    if general_info.is_version_at_least(2022, 3, 0, 0) {
        builder.resolve_placeholder(GMPointer::TexturePageDataSize(index), image_data.compressed_data.len() as i32)?;
    }
    
    Ok(())
}


fn render_image_bz2_qoi(images: Vec<&DynamicImage>) -> Result<Vec<ImageData>, String> {
    let compressed_images: Mutex<Vec<ImageData>> = Mutex::new(Vec::with_capacity(images.len()));
    
    images.into_par_iter().try_for_each(|image| { 
        let width: u32 = image.width();
        let height: u32 = image.height();
        let bytes: Vec<u8> = image.to_rgba8().into_raw();

        let data: Vec<u8> = qoi::encode_to_vec(bytes, width, height)
            .map_err(|e| format!("Could not build QOI image: {e}"))?;
        let uncompressed_size: usize = data.len();

        let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::best());
        encoder.write_all(&data)
            .map_err(|e| format!("Could not write QOI image data to BZip2 archive: {e}"))?;
        drop(data);
        let compressed_data: Vec<u8> = encoder.finish()
            .map_err(|e| format!("Could not finish compressing Bzip2 QOI image: {e}"))?;
        
        compressed_images.lock().unwrap().push(ImageData { 
            compressed_data,
            uncompressed_size,
            width, 
            height, 
        });
        Ok(())
    }).map_err(|e: String| format!("Error while rendering bzip2 qoi images: {e}"))?;
    
    Ok(compressed_images.into_inner().map_err(|e| format!("Could not acquire compressed images mutex: {e}"))?)
}
