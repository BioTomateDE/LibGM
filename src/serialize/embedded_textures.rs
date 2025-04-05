use crate::deserialize::all::UTData;
use crate::serialize::all::{DataBuilder, UTRef};
use crate::serialize::chunk_writing::ChunkBuilder;


#[allow(non_snake_case)]
pub fn build_chunk_TXTR(data_builder: &mut DataBuilder, ut_data: &UTData, texture_pages: Vec<image::DynamicImage>) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "TXTR", abs_pos: data_builder.len() };
    let len: usize = texture_pages.len();
    builder.write_usize(len)?;

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, UTRef::TexturePage(i))?;
    }

    for (i, texture_page) in texture_pages.iter().enumerate() {
        data_builder.push_pointing_to(&mut builder, UTRef::TexturePage(i))?;
        builder.write_u32(0)?;          // scaled
        if ut_data.general_info.is_version_at_least(2, 0, 6, 0) {
            builder.write_u32(0)?;      // number of generated mipmap levels
        }
        if ut_data.general_info.is_version_at_least(2022, 3, 0, 0) {
            builder.write_u32(0)?;      // texture block size  (this placeholder will definitely break)
        }
        if ut_data.general_info.is_version_at_least(2022, 9, 0, 0) {
            builder.write_u32(texture_page.width())?;
            builder.write_u32(texture_page.height())?;
            builder.write_usize(i)?;
        }
        data_builder.push_pointer_position(&mut builder, UTRef::TexturePageData(i))?;
    }

    for (i, texture_page) in texture_pages.iter().enumerate() {
        data_builder.push_pointing_to(&mut builder, UTRef::TexturePageData(i))?;
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        if let Err(error) = texture_page.write_to(&mut buf, image::ImageFormat::Png) {
            return Err(format!("Could not build PNG image for texture page with index {}: {}", i, error))
        }
        builder.write_bytes(&buf.into_inner())?;
    }

    Ok(())
}