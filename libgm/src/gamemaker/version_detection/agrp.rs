use crate::{
    gamemaker::{deserialize::reader::DataReader, version::GMVersionReq},
    prelude::*,
};

pub fn check_2024_14(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    // Check for new field added in 2024.14
    let audio_group_count = reader.read_u32()?;
    if audio_group_count == 0 {
        return Ok(None); // No way to check when there's no audio groups
    }

    // Scan for up to two valid audio group pointers
    let mut i: u32 = 0;
    let mut position1: u32 = 0;
    while position1 == 0 {
        if i >= audio_group_count {
            break;
        }
        position1 = reader.read_u32()?;
        i += 1;
    }
    let mut position2: u32 = 0;
    while position2 == 0 {
        if i >= audio_group_count {
            break;
        }
        position2 = reader.read_u32()?;
        i += 1;
    }
    if position1 == 0 && position2 == 0 {
        return Ok(None); // No groups to check
    }
    if position2 == 0 {
        // Only one group
        // Look for non-null bytes in the 4 bytes after the audio group name (and within bounds of the chunk)
        reader.cur_pos = position1 + 4;
        if reader.cur_pos + 4 > reader.chunk.end_pos {
            return Ok(None); // New field can't fit in remaining space
        }
        let path_pointer = reader.read_u32()?;
        if path_pointer == 0 {
            return Ok(None); // If the field data is zero, it's not 2024.14
        }
    } else {
        // >= 2 groups
        if position2 - position1 == 4 {
            return Ok(None); // If offset is 4; it's not 2024.14
        }
    }

    Ok(Some((2024, 14).into()))
}
