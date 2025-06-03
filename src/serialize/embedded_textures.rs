use std::io::Write;
use image::DynamicImage;
use crate::debug_utils::DurationExt;
use crate::deserialize::all::GMData;
use crate::deserialize::embedded_textures::{GMEmbeddedTexture, MAGIC_BZ2_QOI_HEADER, MAGIC_QOI_HEADER};
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
    log::warn!("");
    let t_start1 = cpu_time::ProcessTime::now();
    // padding
    while builder.len() % 0x80 != 0 {
        builder.write_u8(0);
    }
    
    builder.resolve_pointer(GMPointer::TexturePageData(index))?;

    let width: u32 = image.width();
    let height: u32 = image.height();
    let pixels = if let Some(rgba) = image.as_rgba8() {
        rgba.clone().into_raw()         // Zero-copy if already RGBA    (.clone does not copy entire image buffer; only takes ownership)
    } else {
        log::warn!("Image is not Rgba8! This should not happen if the image loaded was PNG or \
        QOI (always the case in GameMaker). Cloning entire image buffer as fallback. Image: {image:?}");
        image.to_rgba8().into_raw()     // Fallback to conversion
    };

    let t_start2 = cpu_time::ProcessTime::now();
    let qoi_header = rapid_qoi::Qoi {
        width,
        height,
        colors: rapid_qoi::Colors::Rgba,
    };
    let mut uncompressed_data = Vec::with_capacity((width * height * 4) as usize + MAGIC_QOI_HEADER.len());
    qoi_header.encode(&pixels, &mut uncompressed_data)
        .map_err(|e| format!("Could not build QOI image for texture page #{index}: {e}"))?;
    let uncompressed_size: usize = uncompressed_data.len();
    log::debug!("Encoding image into QOI took {}", t_start2.elapsed().ms());

    let t_start2 = cpu_time::ProcessTime::now();
    let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::best());
    encoder.write_all(&uncompressed_data)
        .map_err(|e| format!("Could not write QOI image data to BZip2 archive: {e}"))?;
    drop(uncompressed_data);
    let compressed_data: Vec<u8> = encoder.finish()
        .map_err(|e| format!("Could not finish compressing Bzip2 QOI image: {e}"))?;
    // let compressed_data: Vec<u8> = compress_bzip2_ffi(&uncompressed_data)?;
    let compressed_size: usize = compressed_data.len();
    log::debug!("Compressing QOI image data using Bzip2 took {}", t_start2.elapsed().ms());

    builder.write_bytes(MAGIC_BZ2_QOI_HEADER);
    builder.write_u16(width as u16);
    builder.write_u16(height as u16);
    if general_info.is_version_at_least(2022, 5, 0, 0) {
        builder.write_usize(uncompressed_size);
    }
    builder.write_bytes(&compressed_data);
    
    if general_info.is_version_at_least(2022, 3, 0, 0) {
        builder.resolve_placeholder(GMPointer::TexturePageDataSize(index), compressed_size as i32)?;
    }
    log::debug!("Writing image with dimensions {width}x{height} took {}", t_start1.elapsed().ms());
    Ok(())
}




// // Unsafe but FAST - compresses in a single call
// fn compress_bzip2_ffi(data: &[u8]) -> Result<Vec<u8>, String> {
//     use libbz2_rs_sys::*;
//     
//     let mut dest_len: u32 = (data.len() * 2) as u32;   // Conservative estimate
//     let mut dest: Vec<u8> = Vec::with_capacity(dest_len as usize);
// 
//     let ret = unsafe { BZ2_bzBuffToBuffCompress(
//         dest.as_mut_ptr() as *mut std::ffi::c_char,   // *mut c_char
//         &mut dest_len as *mut u32,                    // *mut u32
//         data.as_ptr() as *mut std::ffi::c_char,       // *mut c_char
//         data.len() as u32,                            // u32
//         9,      // Best compression
//         0,         // Verbosity (0 = silent)
//         30,      // Work factor (default=30)
//     ) };
// 
//     if ret != BZ_OK {
//         return Err(format!("Bzip compression not successful: error code {ret}"))
//     }
//     unsafe { dest.set_len(dest_len as usize); }
//     Ok(dest)
// }
// 


