use crate::deserialize::all::GMData;
use crate::serialize::chunk_writing::{DataBuilder, DataPlaceholder};

pub fn build_chunk_tpag(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("TPAG")?;
    let len: usize = gm_data.texture_page_items.texture_page_items.len();
    builder.write_usize(len);

    for i in 0..len{
        builder.write_placeholder(DataPlaceholder::TexturePageItem(i))?;
    }

    for (i, texture_page_item) in gm_data.texture_page_items.texture_page_items.iter().enumerate() {
        builder.resolve_pointer(DataPlaceholder::TexturePageItem(i))?;
        builder.write_u16(texture_page_item.source_x);
        builder.write_u16(texture_page_item.source_y);
        builder.write_u16(texture_page_item.source_width);
        builder.write_u16(texture_page_item.source_height);
        builder.write_u16(texture_page_item.target_x);
        builder.write_u16(texture_page_item.target_y);
        builder.write_u16(texture_page_item.target_width);
        builder.write_u16(texture_page_item.target_height);
        builder.write_u16(texture_page_item.bounding_width);
        builder.write_u16(texture_page_item.bounding_height);
        builder.write_u16(texture_page_item.texture_page.index as u16);
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

