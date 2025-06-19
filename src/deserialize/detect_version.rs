use std::borrow::Borrow;
use std::collections::HashMap;
use crate::deserialize::chunk_reading::{vec_with_capacity, DataReader, GMChunk};
use crate::deserialize::general_info::{GMVersion, GMVersionLTS, GMVersionReq};


fn try_check<R: Into<GMVersionReq>, T: Into<GMVersionReq>>(
    reader: &mut DataReader,
    chunk_name: &str,
    check_fn: fn(&mut DataReader) -> Result<Option<GMVersionReq>, String>,
    required_version: R,
    target_version: T,
) -> Result<(), String> {
    let target_version: GMVersionReq = target_version.into();
    if reader.general_info.is_version_at_least(&target_version) {   // TODO check lts
        return Ok(())    // no need to check if already
    }

    if let Some(chunk) = reader.chunks.get(chunk_name) {
        let required_version: GMVersionReq = required_version.into();
        if !reader.general_info.is_version_at_least(&required_version) {
            return Err(format!(
                "Version requirement for checking version {} in chunk '{}' failed: {} < {}",
                target_version, chunk_name, reader.general_info.version, required_version,
            ))
        }
        reader.chunk = chunk.clone();
        reader.cur_pos = chunk.start_pos;
        if let Some(version_req) = check_fn(reader)? {
            reader.general_info.set_version_at_least(version_req.clone())?;
        }
        Ok(())
    } else {
        Ok(())
    }
}


// TODO LTS stuff :c
// TODO in utmt, the CheckFor202X_X methods check the "current" version in the beginning
// cc means check_chunk

pub fn detect_gamemaker_version(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let original_version: GMVersion = reader.general_info.version.clone();
    let saved_pos: usize = reader.cur_pos;
    let saved_chunk: GMChunk = reader.chunk.clone();
    
    if reader.chunks.contains_key("TGIN") {
        reader.general_info.set_version_at_least((2, 2, 1))?;
    }
    if reader.chunks.contains_key("SEQN") {
        reader.general_info.set_version_at_least((2, 3, 0))?;
    }
    if reader.chunks.contains_key("FEDS") {
        reader.general_info.set_version_at_least((2, 3, 6))?;
    }
    if reader.chunks.contains_key("FEAT") {
        reader.general_info.set_version_at_least((2022, 8, 6))?;
    }
    if reader.chunks.contains_key("PSEM") {
        reader.general_info.set_version_at_least((2023, 20))?;
    }
    if reader.chunks.contains_key("UILR") {
        reader.general_info.set_version_at_least((2024, 13))?;
    }

    // TODO implement FUNC:bytecode>=15 and FONT:bytecode>=17 in checker
    try_check(reader, "FUNC", cc_func_2024_8, GMVersionReq::bytecode(15), (2024, 8))?;
    try_check(reader, "FONT", cc_font_2022_2, GMVersionReq::bytecode(17), (2022, 2))?;
    try_check(reader, "ARCV", cc_acrv_2_3_1, (0, 0), (2, 3, 1))?;
    try_check(reader, "ROOM", cc_room_2_2_2_302, (2, 0), (2, 2, 2, 302))?;
    try_check(reader, "TXTR", cc_txtr_2_0_6, (2, 0), (2, 0, 6))?;
    try_check(reader, "TGIN", cc_tgin_2_0_6_and_2023_1_nl, (2, 3), (2023, 1, 0, 0, GMVersionLTS::Post2022_0))?;
    try_check(reader, "TXTR", cc_txtr_2022_3_and_2022_5, (2, 3), (2022, 5))?;
    try_check(reader, "ROOM", cc_room_2022_1, (2, 3), (2022, 1))?;
    try_check(reader, "OBJT", cc_objt_2022_5, (2, 3), (2022, 5))?;
    try_check(reader, "EXTN", cc_extn_2022_6, (2, 3), (2022, 6))?;
    try_check(reader, "EXTN", cc_extn_2023_4, (2022, 6), (2023, 4))?;
    try_check(reader, "FONT", cc_font_2023_6_and_2024_11, (2022, 6), (2024, 11))?;
    try_check(reader, "ROOM", cc_room_2024_2, (2023, 2), (2024, 2))?;
    try_check(reader, "SOND", cc_sond_2024_6, (2022, 2, 0, 0, GMVersionLTS::Post2022_0), (2024, 6))?;
    try_check(reader, "SPRT", cc_sprt_2024_6, (2022, 2, 0, 0, GMVersionLTS::Post2022_0), (2024, 6))?;
    try_check(reader, "FONT", cc_font_2024_14, (2024, 13), (2024, 14))?;
    try_check(reader, "AGRP", cc_agrp_2024_14, (2024, 13), (2024, 14))?;
    // TODO implement rest
    
    reader.cur_pos = saved_pos;
    reader.chunk = saved_chunk;
    let ver: &GMVersion = &reader.general_info.version;
    if *ver == original_version {
        return Ok(None)
    }
    let lts: Option<GMVersionLTS> = if original_version.lts == ver.lts { None } else { Some(ver.lts) };
    Ok(Some((ver.major, ver.minor, ver.release, ver.build, lts).into()))
}

/// assert version >= 2.3
fn cc_extn_2022_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2022, 6).into()));

    let ext_count = reader.read_i32()?;
    if ext_count < 1 {
        return Ok(None)
    }
    let first_ext_ptr: usize = reader.read_usize()?;
    let first_ext_end_ptr: usize = if ext_count >= 2 { reader.read_usize()? } else { reader.chunk.end_pos };
    reader.cur_pos = first_ext_ptr + 12;
    let new_pointer1: usize = reader.read_usize()?;
    let new_pointer2: usize = reader.read_usize()?;
    if new_pointer1 != reader.cur_pos {
        return Ok(None)  // first pointer mismatch
    }
    if new_pointer2 <= reader.cur_pos || new_pointer2 >= reader.chunk.end_pos {
        return Ok(None)  // second pointer out of bounds
    }
    // check ending position
    reader.cur_pos = new_pointer2;
    let option_count: usize = reader.read_usize()?;
    if option_count > 0 {
        let new_offset_check: usize = reader.cur_pos + 4*(option_count-1);     // MAYBE overflow issues on 32bit arch????
        if new_offset_check >= reader.chunk.end_pos {
            return Ok(None)   // Option count would place us out of bounds
        }
        reader.cur_pos += 4*(option_count-1);
        let new_offset_check: usize = reader.read_usize()? + 12;    // jump past last option
        if new_offset_check >= reader.chunk.end_pos {
            return Ok(None)   // Pointer list element would place us out of bounds
        }
        reader.cur_pos = new_offset_check;
        if ext_count == 1 {
            reader.cur_pos += 16;    // skip GUID date (only one of them)
            if reader.cur_pos & 16 != 0 {
                reader.cur_pos += 16 - reader.cur_pos%16;   // align to chunk end
            }
        }
        if reader.cur_pos != first_ext_end_ptr {
            return Ok(None)
        }
    }

    target_ver
}


/// assert version >= 2022.6
fn cc_extn_2023_4(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2023, 4).into()));
    let ext_count = reader.read_i32()?;
    if ext_count < 1 {
        return Ok(None)
    }
    // go to first extension and skip the minimal amount of strings
    reader.cur_pos = reader.read_usize()? + 4*3;
    let files_pointer: u32 = reader.read_u32()?;
    let options_pointer: u32 = reader.read_u32()?;
    // The file list pointer should be less than the option list pointer.
    // If it's not true, then "files_pointer" is actually a string pointer, so it's GM 2023.4+.
    if files_pointer > options_pointer {
        return target_ver
    }
    Ok(None)
}


/// assert version >= 2023.2 NON_LTS
fn cc_sond_2024_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2024, 6).into()));
    let possible_sound_count: usize = reader.read_usize()?;
    let mut sound_pointers: Vec<u32> = Vec::with_capacity(2);

    for _ in 0..possible_sound_count {
        let pointer: u32 = reader.read_u32()?;
        if pointer == 0 { continue }
        sound_pointers.push(pointer);
        if sound_pointers.len() >= 2 { break }
    }

    if sound_pointers.len() >= 2 {
        // If first sound's theoretical (old) end offset is below the start offset of
        // the next sound by exactly 4 bytes, then this is 2024.6.
        if sound_pointers[0] + 4*9 == sound_pointers[1] - 4 {
            return target_ver;
        } else if sound_pointers.len() == 1 {
            // If there's a nonzero value where padding should be at the
            // end of the sound, then this is 2024.6.
            let abs_pos: u32 = sound_pointers[0] + 4*9;
            if abs_pos % 16 != 4 {
                return Err("Expected to be on specific alignment at this point".to_string())
            }
            reader.cur_pos = abs_pos as usize;
            if reader.read_u32()? != 0 {
                return target_ver;
            }
        }
    }
    Ok(None)
}


/// assert version >= 2024.13
fn cc_agrp_2024_14(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2024, 14).into()));

    // Check for new field added in 2024.14
    let audio_group_count: u32 = reader.read_u32()?;
    if audio_group_count == 0 {
        return Ok(None)    // No way to check when there's no audio groups
    }

    // Scan for up to two valid audio group pointers
    let mut i: u32 = 0;
    let mut position1: u32 = 0;
    while position1 == 0 {
        if i >= audio_group_count { break }
        position1 = reader.read_u32()?;
        i += 1;
    }
    let mut position2: u32 = 0;
    while position2 == 0 {
        if i >= audio_group_count { break }
        position2 = reader.read_u32()?;
        i += 1;
    }
    if position1 == 0 && position2 == 0 {
        return Ok(None)     // no groups to check
    }
    if position2 == 0 {     // only one group
        // Look for non-null bytes in the 4 bytes after the audio group name (and within bounds of the chunk)
        reader.cur_pos = position1 as usize + 4;
        if reader.cur_pos+4 > reader.chunk.end_pos {
            return Ok(None)     // new field can't fit in remaining space
        }
        let path_pointer = reader.read_u32()?;
        if path_pointer == 0 {
            return Ok(None)     // If the field data is zero, it's not 2024.14
        }
    } else {    // >= 2 groups
        if position2 - position1 == 4 {
            return Ok(None)     // if offset is 4; it's not 2024.14
        }
    }

    target_ver
}


/// assert version >= 2023.2 NON_LTS
fn cc_sprt_2024_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2024, 6).into()));
    let sprite_count: usize = reader.read_usize()?;
    for i in 0..sprite_count {
        reader.cur_pos = reader.chunk.start_pos + i*4 + 4;
        let sprite_pointer: usize = reader.read_usize()?;
        if sprite_pointer == 0 { continue }

        let mut next_sprite_pointer: usize = 0;
        for _ in i+1..sprite_count {
            let pointer = reader.read_usize()?;
            if pointer != 0 {
                next_sprite_pointer = pointer;
                break
            }
        }

        reader.cur_pos += 4;     // Skip past "Name"
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
            continue    // We can't determine anything from this sprite
        }
        reader.cur_pos += 28;
        if reader.read_i32()? != -1 {    // not special type
            continue    // or return?
        }
        let special_version = reader.read_u32()?;
        if special_version != 3 {
            continue    // or return?
        }
        let sprite_type = reader.read_u32()?;
        if sprite_type != 0 {   // 0 <=> GMSpriteType::Normal
            continue    // We can't determine anything from this sprite
        }
        let sequence_offset = reader.read_usize()?;
        let nine_slice_offset = reader.read_usize()?;
        let texture_count = reader.read_usize()?;
        reader.cur_pos += texture_count*4;   // Skip past texture pointers
        let mask_count = reader.read_usize()?;
        if mask_count == 0 {
            continue    // We can't determine anything from this sprite
        }
        let mut full_length: usize = ((width + 7) / 8 * height * mask_count as u32) as usize;
        if full_length % 4 != 0 {
            full_length += 4 - full_length % 4;   // idk
        }
        let mut bbox_length: usize = ((bbox_width + 7) / 8 * bbox_height * mask_count as u32) as usize;
        if bbox_length % 4 != 0 {
            bbox_length += 4 - bbox_length % 4;   // idk
        }

        let full_end_pos = reader.cur_pos + full_length;
        let bbox_end_pos = reader.cur_pos + bbox_length;
        let expected_end_offset: usize;
        if sequence_offset != 0 {
            expected_end_offset = sequence_offset;
        } else if nine_slice_offset != 0 {
            expected_end_offset = nine_slice_offset;
        } else if next_sprite_pointer != 0 {
            expected_end_offset = next_sprite_pointer;
        } else {
            // Use chunk length, and be lenient with it (due to chunk padding)
            if full_end_pos%16 != 0 && full_end_pos + (16 - full_end_pos%16) == reader.chunk.end_pos {
                return Ok(None)   // "Full" mask data doesn't exactly line up, but works if rounded up to the next chunk padding
            }
            if bbox_end_pos%16 != 0 && bbox_end_pos + (16 - bbox_end_pos%16) == reader.chunk.end_pos {
                return target_ver   // "Bbox" mask data doesn't exactly line up, but works if rounded up to the next chunk padding
            }
            return Err("Failed to detect mask type in 2024.6 detection".to_string())
        }
        
        if full_end_pos == expected_end_offset {
            return Ok(None)   // "Full" mask data is valid   (TODO no idea why it returns here tbh; check if there is bug in utmt pls)
        }
        if bbox_end_pos == expected_end_offset {
            return target_ver   // "Bbox" mask data is valid
        }
    }

    Ok(None)
}


/// assert BYTECODE version >= 17
fn cc_font_2022_2(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    // {~~} return if bytecode<17 or at gm >= 2022.2
    let target_ver = Ok(Some((2022, 2).into()));
    let possible_font_count: u32 = reader.read_u32()?;
    if possible_font_count < 1 {
        return Ok(None)
    }

    let mut first_font_pointer: usize = 0;
    for _ in 0..possible_font_count {
        let pointer: usize = reader.read_usize()?;
        if pointer != 0 {
            first_font_pointer = pointer;
            break
        }
    };
    if first_font_pointer == 0 {
        return Ok(None)
    }

    reader.cur_pos = first_font_pointer + 48;
    let glyph_count: usize = reader.read_usize()?;
    if glyph_count * 4 > reader.chunk.end_pos - reader.chunk.start_pos {
        return Ok(None)
    }
    if glyph_count == 0 {
        return target_ver   // FIXME: seems stupid but i think the logic in utmt is equivalent to this
    }

    let mut glyph_pointers: Vec<usize> = vec_with_capacity(glyph_count)?;
    for _ in 0..glyph_count {
        let pointer: usize = reader.read_usize()?;
        if pointer == 0 {
            return Err("One of the glyph pointers is null?".to_string())
        }
        glyph_pointers.push(pointer);
    }
    for pointer in glyph_pointers {
        if reader.cur_pos != pointer {
            return Ok(None)
        }
        reader.cur_pos += 14;
        let kerning_length: u16 = reader.read_u16()?;
        reader.cur_pos += kerning_length as usize * 4;
        // from utmt: "combining read/write would apparently break" ???
    }

    target_ver
}

// TODO: FONT::CheckForGM2023_6AndGM2024_11


