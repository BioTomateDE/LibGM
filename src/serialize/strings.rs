use crate::deserialize::all::GMData;
use crate::deserialize::strings::{GMStringRef};
use crate::serialize::all::{build_chunk, DataBuilder, GMRef};
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_chunk_STRG(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "STRG", abs_pos: data_builder.len() };
    let len: usize = gm_data.strings.len();
    builder.write_usize(len)?;

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, GMRef::String(GMStringRef { index: i }))?;
    }

    for i in 0..len {
        let string_ref: GMStringRef = match gm_data.strings.get_string_by_index(i) {
            Some(string) => string,
            None => return Err(format!(
                "[Internal Error] String index out of bounds ({} >= {}) while building chunk 'STRG'.\
                 This should never happen as `GMData.strings.len()` should return the same length \
                 as the private `GMData.strings.strings_by_index` list.",
                i, len,
            )),
        };
        let string: &str = string_ref.resolve(&gm_data.strings)?;

        builder.write_usize(string.len())?;
        data_builder.push_pointing_to(&mut builder, GMRef::String(GMStringRef {index: i}))?;

        builder.write_literal_string(&string)?;
        builder.write_u8(0)?        // write trailing null byte
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}

