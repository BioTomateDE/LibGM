use crate::deserialize::all::GMData;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};


pub fn build_chunk_txtr(data_builder: &mut DataBuilder, gm_data: &GMData, texture_pages: Vec<image::DynamicImage>) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "TXTR");
    let len: usize = texture_pages.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.write_pointer_placeholder(&mut builder, GMPointer::TexturePage(i))?;
    }

    for (i, texture_page) in texture_pages.iter().enumerate() {
        data_builder.resolve_pointer(&mut builder, GMPointer::TexturePage(i))?;
        builder.write_u32(0);          // scaled
        if gm_data.general_info.is_version_at_least(2, 0, 6, 0) {
            builder.write_u32(0);      // number of generated mipmap levels
        }
        if gm_data.general_info.is_version_at_least(2022, 3, 0, 0) {
            builder.write_u32(0);      // texture block size  (this placeholder will definitely break)
        }
        if gm_data.general_info.is_version_at_least(2022, 9, 0, 0) {
            builder.write_u32(texture_page.width());
            builder.write_u32(texture_page.height());
            builder.write_usize(i);
        }
        data_builder.write_pointer_placeholder(&mut builder, GMPointer::TexturePageData(i))?;
    }

    for (i, texture_page) in texture_pages.iter().enumerate() {
        data_builder.resolve_pointer(&mut builder, GMPointer::TexturePageData(i))?;
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        if let Err(error) = texture_page.write_to(&mut buf, image::ImageFormat::Png) {
            return Err(format!("Could not build PNG image for texture page with index {}: {}", i, error))
        }
        builder.write_bytes(&buf.into_inner());
    }

    builder.finish(data_builder)?;
    Ok(())
}

