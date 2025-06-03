use std::io::Write;
use hardqoi::common::QOIHeader;
use image::DynamicImage;
use crate::debug_utils::DurationExt;
use crate::deserialize::all::GMData;
use crate::deserialize::embedded_textures::{GMEmbeddedTexture, MAGIC_BZ2_QOI_HEADER};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};


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

    for (i, texture_page) in gm_data.texture_pages.iter().enumerate() {
        if let Some(ref image) = texture_page.image {
            build_texture_page_image(builder, &gm_data.general_info, i, image)?;
        }
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

fn build_texture_page_image(builder: &mut DataBuilder, general_info: &GMGeneralInfo, index: usize, image: &DynamicImage) -> Result<(), String> {
    // log::warn!("");
    // let t_start1 = cpu_time::ProcessTime::now();
    // padding
    while builder.len() % 0x80 != 0 {
        builder.write_u8(0);
    }
    
    builder.resolve_pointer(GMPointer::TexturePageData(index))?;

    let width: u32 = image.width();
    let height: u32 = image.height();
    let pixels: Vec<u8> = if let Some(rgba) = image.as_rgba8() {
        rgba.clone().into_raw()         // Zero-copy if already RGBA    (.clone does not copy entire image buffer; only takes ownership)
    } else {
        log::warn!("Image is not Rgba8! This should not happen if the image loaded was PNG or \
        QOI (always the case in GameMaker). Cloning entire image buffer as fallback. Image: {image:?}");
        image.to_rgba8().into_raw()     // Fallback to conversion
    };

    // let t_start2 = cpu_time::ProcessTime::now();
    let qoi_header = QOIHeader {
        width,
        height,
        has_alpha: image.color().has_alpha(),   // will basically always be true
        linear_rgb: false,
    };
    let pixels: Vec<u32> = rgba8_to_rgba32(pixels);
    let mut uncompressed_data: Vec<u8> = vec![0; (width * height * 4) as usize];
    hardqoi::encode(&pixels, &mut uncompressed_data, qoi_header).map_err(|(last_pos, pixel_count)| format!(
        "Could not build QOI image for texture page #{index}; last pos is {last_pos} and pixel count is {pixel_count}",
    ))?;
    let uncompressed_size: usize = uncompressed_data.len();
    // log::debug!("Encoding image into QOI took {}", t_start2.elapsed().ms());
    
    // // let t_start2 = cpu_time::ProcessTime::now();
    // let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::best());
    // encoder.write_all(&uncompressed_data)
    //     .map_err(|e| format!("Could not write QOI image data to BZip2 archive: {e}"))?;
    // drop(uncompressed_data);
    // let data: Vec<u8> = encoder.finish()
    //     .map_err(|e| format!("Could not finish compressing Bzip2 QOI image: {e}"))?;
    let data = uncompressed_data;   // comment out lines above to use bzip compression (slower)
    let data_size: usize = data.len();
    // log::debug!("Compressing QOI image data using Bzip2 took {}", t_start2.elapsed().ms());

    builder.write_bytes(MAGIC_BZ2_QOI_HEADER);
    builder.write_u16(width as u16);
    builder.write_u16(height as u16);
    if general_info.is_version_at_least(2022, 5, 0, 0) {
        builder.write_usize(uncompressed_size);
    }
    builder.write_bytes(&data);
    
    if general_info.is_version_at_least(2022, 3, 0, 0) {
        builder.resolve_placeholder(GMPointer::TexturePageDataSize(index), data_size as i32)?;
    }
    // log::debug!("Writing image with dimensions {width}x{height} took {}", t_start1.elapsed().ms());
    Ok(())
}





fn rgba8_to_rgba32(bytes: Vec<u8>) -> Vec<u32> {
    assert_eq!(bytes.len() % 4, 0);
    assert_eq!(bytes.as_ptr() as usize % align_of::<u32>(), 0);

    let mut bytes = bytes;
    let (ptr, len, cap) = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());

    // SAFETY:
    // - Original Vec<u8> is properly aligned (asserted above)
    // - Length is divisible by 4 (asserted above)
    // - u32 and [u8; 4] have the same size/alignment
    unsafe {
        std::mem::forget(bytes); // Prevent double-free
        Vec::from_raw_parts(ptr as *mut u32, len / 4, cap / 4)
    }
}

