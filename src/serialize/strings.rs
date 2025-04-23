use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_chunk_strg(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "STRG", abs_pos: data_builder.len() };
    let len: usize = gm_data.strings.strings_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMRef::string(i))?;
    }

    for i in 0..len {
        let string: &String = &gm_data.strings.strings_by_index[i];

        builder.write_usize(string.len());
        data_builder.push_pointer_resolve(&mut builder, GMRef::string(i))?;

        builder.write_literal_string(string)?;
        builder.write_u8(0)        // write trailing null byte
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}

