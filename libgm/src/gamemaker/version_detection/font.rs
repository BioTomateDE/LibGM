use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        gm_version::{GMVersionReq, LTSBranch::PostLTS},
    },
    prelude::*,
    util::init::vec_with_capacity,
};

pub fn check_2022_2(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2022, 2).into()));
    let possible_font_count = reader.read_u32()?;
    if possible_font_count < 1 {
        return Ok(None);
    }

    let mut first_font_pointer = 0;
    for _ in 0..possible_font_count {
        let pointer = reader.read_u32()?;
        if pointer != 0 {
            first_font_pointer = pointer;
            break;
        }
    }
    if first_font_pointer == 0 {
        return Ok(None);
    }

    reader.cur_pos = first_font_pointer + 48;
    let glyph_count = reader.read_u32()?;
    if glyph_count * 4 > reader.chunk.length() {
        return Ok(None);
    }
    if glyph_count == 0 {
        log::warn!("Glyph count is zero while detecting FONT_2022.2; may lead to false positives");
        return target_ver; // UTMT also assumes that it is 2022.2; even if there are no glyphs
    }

    let mut glyph_pointers: Vec<u32> = vec_with_capacity(glyph_count)?;
    for _ in 0..glyph_count {
        let pointer = reader.read_u32()?;
        if pointer == 0 {
            bail!("One of the glyph pointers is null?");
        }
        glyph_pointers.push(pointer);
    }
    for pointer in glyph_pointers {
        if reader.cur_pos != pointer {
            return Ok(None);
        }
        reader.cur_pos += 14;
        let kerning_length = reader.read_u16()?;
        reader.cur_pos += u32::from(kerning_length) * 4;
        // From utmt: "combining read/write would apparently break" ???
    }

    target_ver
}

/// We already know whether the version is more or less than 2022.8 due to the FEAT chunk being present.
/// Taking advantage of that, this is basically the same as the 2022.2 check, but it:
/// - Checks for the `LineHeight` value instead of `Ascender` (added in 2023.6)
/// > `PSEM` (2023.2) is not used, as it would return a false negative on LTS (2022.9+ equivalent with no particles).
/// - Checks for `UnknownAlwaysZero` in Glyphs (added in 2024.11)
pub fn check_2023_6_and_2024_11(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    // Explicit check because the logic is very scuffed
    if !reader.general_info.is_version_at_least((2022, 8)) {
        return Ok(None); // Version requirement (for checking 2023.6) not satisfied
    }
    if reader.general_info.is_version_at_least((2023, 6))
        && !reader.general_info.is_version_at_least((2024, 6))
    {
        return Ok(None); // 2023.6 already detected; but 2024.6 not yet detected
    }
    if reader.general_info.is_version_at_least((2024, 11)) {
        return Ok(None); // 2024.11 already detected
    }

    let possible_font_count = reader.read_i32()?;
    let mut first_two_pointers: Vec<u32> = Vec::with_capacity(2);
    for _ in 0..possible_font_count {
        let ptr = reader.read_u32()?;
        if ptr == 0 {
            continue;
        }
        first_two_pointers.push(ptr);
        if first_two_pointers.len() >= 2 {
            break;
        }
    }
    if first_two_pointers.is_empty() {
        return Ok(None); // Nothing to detect
    }
    if first_two_pointers.len() == 1 {
        // Add in the position of the padding i.e. the end of the font list
        first_two_pointers.push(reader.chunk.end_pos - 512);
    }

    reader.cur_pos = first_two_pointers[0] + 52; // Also the LineHeight value. 48 + 4 = 52
    if reader
        .general_info
        .is_version_at_least((2023, 2, 0, 0, PostLTS))
    {
        // SDFSpread is present from 2023.2 non-LTS onward
        reader.cur_pos += 4; // (detected by PSEM/PSYS chunk existence)
    }

    let glyph_count = reader.read_u32()?;
    if glyph_count * 4 > first_two_pointers[1] - reader.cur_pos || glyph_count < 1 {
        return Ok(None);
    }

    let mut glyph_pointers: Vec<u32> = vec_with_capacity(glyph_count)?;
    for _ in 0..glyph_count {
        let ptr = reader.read_u32()?;
        if ptr == 0 {
            bail!("One of the glyph pointers is zero");
        }
        glyph_pointers.push(ptr);
    }

    // let mut detecting_2024_11_failed: bool = false;
    if let Some((i, glyph_pointer)) = glyph_pointers.iter().enumerate().next() {
        if reader.cur_pos != *glyph_pointer {
            return Ok(None);
        }
        reader.cur_pos += 14;
        let kerning_count = reader.read_u16()?;

        // Hopefully the last thing in a UTFont is the glyph list
        let next_glyph_pointer = if i < glyph_pointers.len() - 1 {
            glyph_pointers[i + 1]
        } else {
            first_two_pointers[1]
        };
        // And hopefully the last thing in a glyph is the kerning list
        // Note that we're actually skipping all items of the Glyph.Kerning SimpleList here;
        // 4 is supposed to be the size of a GlyphKerning object
        let pointer_after_kerning_list = reader.cur_pos + 4 * u32::from(kerning_count);
        // If we don't land on the next glyph/font after skipping the Kerning list,
        // KerningLength is probably bogus and UnknownAlwaysZero may be present
        if next_glyph_pointer == pointer_after_kerning_list {
            return Ok(Some((2023, 6).into())); // 2023.6 succeeded; 2024.11 failed
        }
        // Discard last read, which would be of UnknownAlwaysZero
        let kerning_count = reader.read_u16()?;
        let pointer_after_kerning_list = reader.cur_pos + 4 * u32::from(kerning_count);
        if next_glyph_pointer != pointer_after_kerning_list {
            log::warn!(
                "There appears to be more/fewer values than UnknownAlwaysZero before \
                the kerning list in GMFontGlyph; data file potentially corrupted"
            );
        }
        return Ok(Some((2024, 11).into())); // 2024.11 succeeded (2023.6 did too but doesn't matter)
    }

    Ok(Some((2023, 6).into())) // 2024.11 failed or could not be detected; 2023.6 succeeded
}

pub fn check_2024_14(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    // Check for new padding added (and final chunk "padding" removed) in 2024.14
    let font_count = reader.read_u32()?;
    let mut last_font_position = 0;
    for _ in 0..font_count {
        let ptr = reader.read_u32()?;
        if ptr != 0 {
            last_font_position = ptr;
        }
    }

    // If we have a last font, advance to the end of its data (ignoring the new alignment added in 2024.14)
    if last_font_position != 0 {
        reader.cur_pos = last_font_position + 56;

        // Advance to last glyph in pointer list
        let glyph_count = reader.read_u32()?;
        reader.cur_pos += (glyph_count - 1) * 4;
        reader.cur_pos = reader.read_u32()? + 16;

        // Advance past kerning
        let kerning_count = reader.read_u16()?;
        reader.cur_pos += u32::from(kerning_count) * 4;
    }

    // Check for the final chunk padding being missing
    if reader.cur_pos + 512 > reader.chunk.end_pos {
        // No padding can fit, so this is 2024.14
        return Ok(Some((2024, 14).into()));
    }

    Ok(None)
}
