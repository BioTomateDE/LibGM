use crate::deserialize::all::UTData;
use crate::deserialize::chunk_reading::UTChunk;
use crate::serialize::data_writing::DataBuilder;

pub fn build_chunk_STRG(ut_data: &UTData) -> Result<UTChunk, String> {
    let mut builder: DataBuilder = DataBuilder { raw_data: Vec::new() };

    let len: usize = ut_data.strings.len();
    builder.write_usize(len)?;

    for i in 0..len {
        let ut_string = &ut_data.strings.strings_by_index[i];
        builder.write_u32(ut_string.id)?;
    }

    for i in 0..len {
        let ut_string = &ut_data.strings.strings_by_index[i];
        builder.write_u32(ut_string.id)?;
        builder.write_string(&ut_string.value)?;
        builder.write_u8(0)?;   // trailing null byte after every string
    }

    let chunk: UTChunk = UTChunk {
        name: "STRG".to_string(),
        abs_pos: 0,     // stub
        data_len: builder.raw_data.len(),
        data: builder.raw_data,
        file_index: 0
    };
    Ok(chunk)
}