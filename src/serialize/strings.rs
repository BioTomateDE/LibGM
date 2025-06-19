use crate::deserialize::all::GMData;
use crate::serialize::chunk_writing::{DataBuilder, DataPlaceholder};

pub fn build_chunk_strg(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("STRG")?;
    let len: usize = gm_data.strings.strings.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(DataPlaceholder::StringPointerList(i))?;
    }

    for (i, string) in gm_data.strings.strings.iter().enumerate() {
        builder.align(4);
        builder.resolve_pointer(DataPlaceholder::StringPointerList(i))?;
        builder.write_usize(string.len());
        builder.resolve_pointer(DataPlaceholder::String(i))?; // actual string reference need to get resolved here bc of gamemaker moment

        builder.write_literal_string(string);
        builder.write_u8(0)        // write trailing null byte
    }
    
    while builder.len() % 0x80 != 0 {
        builder.write_u8(0);
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

