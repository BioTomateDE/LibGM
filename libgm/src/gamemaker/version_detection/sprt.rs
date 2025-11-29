use crate::{
    gamemaker::{deserialize::reader::DataReader, gm_version::GMVersionReq},
    prelude::*,
};

pub fn check_2_3_2(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let pointers: Vec<u32> = reader.read_simple_list()?;
    for pointer in pointers {
        if pointer == 0 {
            continue;
        }
        reader.cur_pos = pointer + 14 * 4;
        if reader.read_i32()? != -1 {
            continue; // Sprite is not special type
        }
        let special_version = reader.read_u32()?;
        if special_version >= 3 {
            return Ok(Some((2, 3, 2).into()));
        }
    }
    Ok(None)
}

pub fn check_2024_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2024, 6).into()));
    let sprite_count = reader.read_u32()?;
    for i in 0..sprite_count {
        reader.cur_pos = reader.chunk.start_pos + i * 4 + 4;
        let sprite_pointer = reader.read_u32()?;
        if sprite_pointer == 0 {
            continue;
        }

        let mut next_sprite_pointer = 0;
        for _ in i + 1..sprite_count {
            let pointer = reader.read_u32()?;
            if pointer != 0 {
                next_sprite_pointer = pointer;
                break;
            }
        }

        reader.cur_pos += 4; // Skip past "Name"
        // Check if bbox size differs from width/height
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let margin_left = reader.read_i32()?;
        let margin_right = reader.read_i32()?;
        let margin_bottom = reader.read_i32()?;
        let margin_top = reader.read_i32()?;
        let bbox_width = (margin_right - margin_left + 1) as u32;
        let bbox_height = (margin_bottom - margin_top + 1) as u32;
        if bbox_width == width && bbox_height == height {
            continue; // We can't determine anything from this sprite
        }
        reader.cur_pos += 28;
        if reader.read_i32()? != -1 {
            // Not special type
            continue;
        }
        let special_version = reader.read_u32()?;
        if special_version != 3 {
            continue;
        }
        let sprite_type = reader.read_u32()?;
        if sprite_type != 0 {
            // 0 <=> GMSpriteType::Normal
            continue; // We can't determine anything from this sprite
        }
        let sequence_offset = reader.read_u32()?;
        let nine_slice_offset = reader.read_u32()?;
        let texture_count = reader.read_u32()?;
        reader.cur_pos += texture_count * 4; // Skip past texture pointers
        let mask_count = reader.read_u32()?;
        if mask_count == 0 {
            continue; // We can't determine anything from this sprite
        }
        let mut full_length = width.div_ceil(8) * height * mask_count;
        if full_length % 4 != 0 {
            full_length += 4 - full_length % 4; // Idk
        }
        let mut bbox_length = bbox_width.div_ceil(8) * bbox_height * mask_count;
        if !bbox_length.is_multiple_of(4) {
            bbox_length += 4 - bbox_length % 4; // Idk
        }

        let full_end_pos = reader.cur_pos + full_length;
        let bbox_end_pos = reader.cur_pos + bbox_length;
        let expected_end_offset;
        if sequence_offset != 0 {
            expected_end_offset = sequence_offset;
        } else if nine_slice_offset != 0 {
            expected_end_offset = nine_slice_offset;
        } else if next_sprite_pointer != 0 {
            expected_end_offset = next_sprite_pointer;
        } else {
            // Use chunk length, and be lenient with it (due to chunk padding)
            if !full_end_pos.is_multiple_of(16)
                && full_end_pos + (16 - full_end_pos % 16) == reader.chunk.end_pos
            {
                return Ok(None); // "Full" mask data doesn't exactly line up, but works if rounded up to the next chunk padding
            }
            if !bbox_end_pos.is_multiple_of(16)
                && bbox_end_pos + (16 - bbox_end_pos % 16) == reader.chunk.end_pos
            {
                return target_ver; // "Bbox" mask data doesn't exactly line up, but works if rounded up to the next chunk padding
            }
            bail!("Failed to detect mask type in 2024.6 detection");
        }

        if full_end_pos == expected_end_offset {
            log::warn!(
                "full_end_pos == expected_end_offset while detecting SPRT_2024.6; may lead to false negatives"
            );
            return Ok(None); // "Full" mask data is valid   (TODO no idea why it returns here tbh; check if there is bug in utmt pls)
        }
        if bbox_end_pos == expected_end_offset {
            return target_ver; // "Bbox" mask data is valid
        }
    }

    Ok(None)
}
