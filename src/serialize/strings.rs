use crate::deserialize::all::GMData;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_strg(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "STRG");
    let len: usize = gm_data.strings.strings_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::string_pointerlist(i))?;
    }

    for (i, string) in gm_data.strings.strings_by_index.iter().enumerate() {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::string_pointerlist(i))?;
        builder.write_usize(string.len());
        data_builder.push_pointer_resolve(&mut builder, GMPointer::string(i))?; // actual string reference need to get resolved here bc of gamemaker moment

        builder.write_literal_string(string)?;
        builder.write_u8(0)        // write trailing null byte
    }

    builder.finish(data_builder)?;
    Ok(())
}

