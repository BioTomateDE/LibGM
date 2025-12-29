use crate::{
    gamemaker::{deserialize::reader::DataReader, version::GMVersionReq},
    prelude::*,
};

pub fn check_2022_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let ext_count = reader.read_u32()?;
    if ext_count < 1 {
        return Ok(None);
    }
    let first_ext_ptr = reader.read_u32()?;
    let first_ext_end_ptr = if ext_count >= 2 {
        reader.read_u32()?
    } else {
        reader.chunk.end_pos
    };
    reader.cur_pos = first_ext_ptr + 12;
    let new_pointer1 = reader.read_u32()?;
    let new_pointer2 = reader.read_u32()?;
    if new_pointer1 != reader.cur_pos {
        return Ok(None); // First pointer mismatch
    }
    if new_pointer2 <= reader.cur_pos || new_pointer2 >= reader.chunk.end_pos {
        return Ok(None); // Second pointer out of bounds
    }
    // Check ending position
    reader.cur_pos = new_pointer2;
    let option_count = reader.read_u32()?;
    if option_count > 0 {
        let new_offset_check = reader.cur_pos + 4 * (option_count - 1); // MAYBE overflow issues on 32bit arch????
        if new_offset_check >= reader.chunk.end_pos {
            return Ok(None); // Option count would place us out of bounds
        }
        reader.cur_pos += 4 * (option_count - 1);
        let new_offset_check = reader.read_u32()? + 12; // Jump past last option
        if new_offset_check >= reader.chunk.end_pos {
            return Ok(None); // Pointer list element would place us out of bounds
        }
        reader.cur_pos = new_offset_check;
        if ext_count == 1 {
            reader.cur_pos += 16; // skip GUID date (only one of them)
            if reader.cur_pos & 16 != 0 {
                reader.cur_pos += 16 - reader.cur_pos % 16; // Align to chunk end
            }
        }
        if reader.cur_pos != first_ext_end_ptr {
            return Ok(None);
        }
    }

    Ok(Some((2022, 6).into()))
}

pub fn check_2023_4(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let ext_count = reader.read_i32()?;
    if ext_count < 1 {
        return Ok(None);
    }
    // Go to first extension and skip the minimal amount of strings
    reader.cur_pos = reader.read_u32()? + 4 * 3;
    let files_pointer = reader.read_u32()?;
    let options_pointer = reader.read_u32()?;
    // The file list pointer should be less than the option list pointer.
    // If it's not true, then "files_pointer" is actually a string pointer, so it's GM 2023.4+.
    if files_pointer > options_pointer {
        return Ok(Some((2023, 4).into()));
    }
    Ok(None)
}
