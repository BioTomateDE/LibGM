// SPDX-License-Identifier: GPL-3.0-only
use super::target_version;
use crate::prelude::*;
use crate::wad::elem::game_object::event::EventGroup;
use crate::wad::load::reader::DataReader;
use crate::wad::version::GMVersion;

pub fn check_2022_5(reader: &mut DataReader) -> Result<Option<GMVersion>> {
    let target_ver = target_version!(GM2022_5);
    let object_count = reader.read_u32()?;
    if object_count < 1 {
        return Ok(None); // No objects; nothing to detect
    }
    let first_object_pointer = reader.read_u32()?;
    reader.cur_pos = first_object_pointer + 64;
    let vertex_count = reader.read_u32()?;

    // i hate integer safety
    let Some(position) =
        vertex_count.checked_mul(8).and_then(|x| x.checked_add(reader.cur_pos + 12))
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

/// 2026 LTS (2026.1) removed the `managed` boolean field for game objects.
pub fn check_2026_1(reader: &mut DataReader) -> Result<Option<GMVersion>> {
    // this check might not work idk
    if reader.version >= GMVersion::GM2022_5 && reader.version < GMVersion::GM2024_13 {
        return Ok(None);
    }

    let obj_count = reader.read_u32()?;
    let mut obj_ptr = 0;
    for _ in 0..obj_count {
        let ptr = reader.read_u32()?;
        if ptr != 0 {
            if obj_ptr != 0 {
                obj_ptr = ptr;
                break;
            }
            obj_ptr = ptr;
        }
    }
    if obj_ptr == 0 {
        // no objects; can't detect anything
        return Ok(None);
    }

    reader.cur_pos = obj_ptr + 64;
    let vertex_count = reader.read_u32()?;

    let mut managed_field_present = true;

    // Bounds check on vertex data
    if let Some(pos) = get_vertex_end_pos(reader.cur_pos, vertex_count)
        && pos < reader.chunk.end_pos
    {
        reader.cur_pos = pos;
        if reader.read_u32()? == EventGroup::COUNT {
            let subevent_ptr = reader.read_u32()?;
            if reader.cur_pos + 56 == subevent_ptr {
                managed_field_present = false;
            }
        }
    }

    if managed_field_present && reader.version < GMVersion::GM2022_5 {
        target_version!(GM2022_5)
    } else if !managed_field_present && reader.version >= GMVersion::GM2024_13 {
        target_version!(GM2026_1)
    } else {
        Ok(None)
    }
}

fn get_vertex_end_pos(cur_pos: u32, vertex_count: u32) -> Option<u32> {
    // reader.cur_pos + 12 + (vertex_count * 8)
    cur_pos.checked_add(12)?.checked_add(vertex_count.checked_mul(12)?)
}
