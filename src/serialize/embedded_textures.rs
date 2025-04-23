use crate::deserialize::all::GMData;
use crate::serialize::all::{DataBuilder, GMRef};
use crate::serialize::chunk_writing::ChunkBuilder;


pub fn build_chunk_TXTR(data_builder: &mut DataBuilder, gm_data: &GMData, texture_pages: Vec<image::DynamicImage>) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "TXTR", abs_pos: data_builder.len() };
    let len: usize = texture_pages.len();
    builder.write_usize(len)?;

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, GMRef::TexturePage(i))?;
    }

    for (i, texture_page) in texture_pages.iter().enumerate() {
        data_builder.push_pointing_to(&mut builder, GMRef::TexturePage(i))?;
        builder.write_u32(0)?;          // scaled
        if gm_data.general_info.is_version_at_least(2, 0, 6, 0) {
            builder.write_u32(0)?;      // number of generated mipmap levels
        }
        if gm_data.general_info.is_version_at_least(2022, 3, 0, 0) {
            builder.write_u32(0)?;      // texture block size  (this placeholder will definitely break)
        }
        if gm_data.general_info.is_version_at_least(2022, 9, 0, 0) {
            builder.write_u32(texture_page.width())?;
            builder.write_u32(texture_page.height())?;
            builder.write_usize(i)?;
        }
        data_builder.push_pointer_position(&mut builder, GMRef::TexturePageData(i))?;
    }

    for (i, texture_page) in texture_pages.iter().enumerate() {
        data_builder.push_pointing_to(&mut builder, GMRef::TexturePageData(i))?;
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        if let Err(error) = texture_page.write_to(&mut buf, image::ImageFormat::Png) {
            return Err(format!("Could not build PNG image for texture page with index {}: {}", i, error))
        }
        builder.write_bytes(&buf.into_inner())?;
    }

    Ok(())
}