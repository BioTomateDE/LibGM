use crate::{
    gamemaker::{deserialize::reader::DataReader, gm_version::GMVersionReq},
    prelude::*,
};

pub fn check_2024_8(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2024, 8).into()));
    if reader.get_chunk_length() == 0 {
        return Ok(None);
    }

    // The CodeLocals list was removed in 2024.8, so we check if Functions is the only thing in here.
    let function_count = reader.read_u32()?;
    // Skip over the (Simple)List
    // (3*4 is the size of a GMFunction object)
    reader.cur_pos += function_count * 3 * 4;

    if reader.cur_pos == reader.chunk.end_pos {
        // Directly reached the end of the chunk after the function list, so code locals are definitely missing
        return target_ver;
    }

    // Align position
    let mut padding_bytes_read: u32 = 0;

    while reader.cur_pos & (reader.chunk_padding - 1) != 0 {
        if reader.cur_pos >= reader.chunk.end_pos || reader.read_u8()? != 0 {
            return Ok(None); // If we hit a non-zero byte (or exceed chunk boundaries), it can't be padding
        }
        padding_bytes_read += 1;
    }

    // If we're at the end of the chunk after aligning padding, code locals are either empty or do not exist altogether.
    if reader.cur_pos != reader.chunk.end_pos {
        return Ok(None);
    }

    if padding_bytes_read < 4 {
        return target_ver;
    }

    // If we read at least 4 padding bytes, we don't know for sure unless we have at least one code entry.
    let Some(chunk_code) = reader.chunks.get("CODE") else {
        return Ok(None);
    };
    reader.chunk = chunk_code.clone();
    reader.cur_pos = chunk_code.start_pos;
    let code_count = reader.read_u32()?;
    if code_count < 1 {
        return Ok(None);
    }

    target_ver
}
