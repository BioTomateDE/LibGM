use crate::deserialize::all::GMData;
use crate::deserialize::embedded_textures::GMEmbeddedTexture;
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
        builder.resolve_pointer(GMPointer::TexturePageData(i))?;
        
        // TODO formats other than PNG?
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        texture_page.image.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| format!("Could not build PNG image for texture page #{i}: {e}"))?;
        
        if gm_data.general_info.is_version_at_least(2022, 3, 0, 0) {
            builder.resolve_placeholder(GMPointer::TexturePageDataSize(i), buf.position() as i32)?;
        }
        builder.write_bytes(&buf.into_inner());
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
    if general_info.is_version_at_least(2022, 3, 0, 0) {
        builder.write_placeholder(GMPointer::TexturePageDataSize(index))?;
    }
    if general_info.is_version_at_least(2022, 9, 0, 0) {
        builder.write_u32(texture_page.image.width());
        builder.write_u32(texture_page.image.height());
        builder.write_usize(index);     // TODO not sure what "index in group" means. maybe this is not just the index?
    }
    builder.write_placeholder(GMPointer::TexturePageData(index))?;
    Ok(())
}

