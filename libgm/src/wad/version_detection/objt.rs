use super::target_version;
use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::version::GMVersion;

pub fn check_2022_5(reader: &mut DataReader) -> Result<Option<GMVersion>> {
    let target_ver = target_version!(2022, 5);
    let object_count = reader.read_u32()?;
    if object_count < 1 {
        return Ok(None); // No objects; nothing to detect
    }
    let first_object_pointer = reader.read_u32()?;
    reader.cur_pos = first_object_pointer + 64;
    let vertex_count = reader.read_u32()?;

    // i hate integer safety
    let Some(position) = vertex_count
        .checked_mul(8)
        .and_then(|x| x.checked_add(reader.cur_pos + 12))
    else {
        return Ok(None);
    };
    if position >= reader.chunk.end_pos {
        return target_ver; // Bounds check on vertex data "failed" => 2022.5
    }
    reader.cur_pos = position;

    if reader.read_u32()? == 15 {
        // !! 15 has to equal variant count of GMGameObjectEventType enum !!
        let sub_event_pointer = reader.read_u32()?;
        if reader.cur_pos + 56 == sub_event_pointer {
            // Subevent pointer check "succeeded"
            // (Should start right after the list) => not 2022.5
            return Ok(None);
        }
    }

    target_ver
}
