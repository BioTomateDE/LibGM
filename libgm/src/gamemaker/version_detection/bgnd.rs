use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        version::{GMVersionReq, LTSBranch},
    },
    prelude::*,
};

/// Go through each background, and check to see if it ends at
/// the expected position. If not, this is probably 2024.14.1.
pub fn check_2024_14_1(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2024, 14, 1, LTSBranch::PostLTS).into()));
    let count = reader.read_u32()?;

    for i in 0..count {
        // Find background's start position, and calculate next background position (if available).
        reader.set_rel_cur_pos(4 * i + 4)?;
        let bg_ptr = reader.read_u32()?;
        if bg_ptr == 0 {
            continue;
        }

        let mut next_bg_ptr = 0;
        for _ in i..count {
            next_bg_ptr = reader.read_u32()?;
            if next_bg_ptr != 0 {
                break;
            }
        }

        // Skip all the way to "GMS2ItemsPerTileCount" (at its pre-2024.14.1 location), which is what we actually care about.
        reader.cur_pos = bg_ptr + 44;
        let items_per_tile_count = reader.read_u32()?;
        let tile_count = reader.read_u32()?;

        // Calculate the theoretical end position given the above info, and compare to the actual end position (with padding).
        let end_pos = bg_ptr + 4 * items_per_tile_count * tile_count + 60;
        if next_bg_ptr == 0 {
            // Align to 16 bytes, and compare against chunk end position
            let end_pos = end_pos.next_multiple_of(16);
            if end_pos != reader.chunk.end_pos {
                // Probably 2024.14.1!
                return target_ver;
            }
        } else {
            // Align to 8 bytes, and compare against next background start position
            let end_pos = end_pos.next_multiple_of(8);
            if end_pos != next_bg_ptr {
                // Probably 2024.14.1!
                return target_ver;
            }
        }
    }

    Ok(None)
}
