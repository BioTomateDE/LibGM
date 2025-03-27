use crate::deserialize::all::UTData;
use crate::deserialize::chunk_reading::UTChunk;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_chunk_STRG(data_builder: &mut DataBuilder, ut_data: &UTData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "STRG" };
    let len: usize = ut_data.strings.len();
    builder.write_usize(len)?;

    // write placeholder bytes for string absolute positions
    for _ in 0..len {
        builder.write_usize(0)?;
    }

    for i in 0..len {
        let string: String = match ut_data.strings.get_string_by_index(i) {
            Some(string) => string,
            None => return Err(format!(
                "[Internal Error] String index out of bounds ({} >= {}) while building chunk 'STRG'.\
                 This should never happen as `UTData.strings.len()` should return the same length \
                 as the private `UTData.string.strings_by_index` list.",
                i,
                len,
            )),
        };

        builder.write_usize(string.len())?;
        let absolute_position: usize = data_builder.len() + builder.len();
        // write string absolute position to placeholder bytes
        let bytes: [u8; 4] = (absolute_position as u32).to_le_bytes();
        builder.overwrite_data(&bytes, i * 4 + 4)?;

        builder.write_string(&string)?;
        builder.write_u8(0)?        // write trailing null byte
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}

