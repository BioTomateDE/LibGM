// SPDX-License-Identifier: GPL-3.0-only
use super::target_version;
use crate::prelude::*;
use crate::wad::load::reader::DataReader;
use crate::wad::version::GMVersion;

pub fn check_2_3_2(reader: &mut DataReader) -> Result<Option<GMVersion>> {
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
            return target_version!(GMS2_3_2);
        }
    }
    Ok(None)
}

pub fn check_2024_6(reader: &mut DataReader) -> Result<Option<GMVersion>> {
    let ver = target_version!(GM2024_6);
    let sprite_count = reader.read_u32()?;
    for i in 0..sprite_count {
        reader.set_rel_cur_pos(i * 4 + 4)?;
        let sprite_pointer = reader.read_u32()?;
        if sprite_pointer == 0 {
            continue;
        }

        let mut next_sprite_ptr = 0;
        for _ in i + 1..sprite_count {
            let pointer = reader.read_u32()?;
            if pointer != 0 {
                next_sprite_ptr = pointer;
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
            // 0 <=> SpriteType::Normal
            continue; // We can't determine anything from this sprite
        }
        let sequence_ptr = reader.read_u32()?;
        let nine_slice_ptr = reader.read_u32()?;
        let texture_count = reader.read_u32()?;
        reader.cur_pos += texture_count * 4; // Skip past texture pointers
        let mask_count = reader.read_u32()?;
        if mask_count == 0 {
            continue; // We can't determine anything from this sprite
        }

        let pos = reader.cur_pos;
        let full_end_pos = calc_end_pos(pos, width, height, mask_count).unwrap_or(0);
        let bbox_end_pos = calc_end_pos(pos, bbox_width, bbox_height, mask_count).unwrap_or(0);

        let expected_end_ptr = first_nonzero([sequence_ptr, nine_slice_ptr, next_sprite_ptr]);
        if let Some(end) = expected_end_ptr {
            // If the Full mask data runs past the expected end offset
            // and the Bbox mask data does not, then this is 2024.6.
            if full_end_pos == end {
                return Ok(None); // "Full" mask data is valid
            }
            if bbox_end_pos == end {
                return ver; // "Bbox" mask data is valid
            }
        } else {
            // Use chunk length, and be lenient with it (due to chunk padding),
            // NOTE: These checks used to also check that it wasn't a multiple of 16. Bring back if necessary.
            if full_end_pos.next_multiple_of(16) == reader.chunk.end_pos {
                // "Full" mask data doesn't exactly line up, but works if rounded up to the next chunk padding.
                return Ok(None);
            }
            if bbox_end_pos.next_multiple_of(16) == reader.chunk.end_pos {
                // "Bbox" mask data doesn't exactly line up, but works if rounded up to the next chunk padding.
                return ver;
            }
            bail!("Failed to detect mask type in 2024.6 detection");
        }
    }

    Ok(None)
}

fn first_nonzero<const N: usize>(pointers: [u32; N]) -> Option<u32> {
    pointers.into_iter().find(|&p| p != 0)
}

fn calc_end_pos(cur_pos: u32, width: u32, height: u32, mask_count: u32) -> Option<u32> {
    width
        .div_ceil(8)
        .checked_mul(height)?
        .checked_mul(mask_count)?
        .next_multiple_of(4)
        .checked_add(cur_pos)
}
