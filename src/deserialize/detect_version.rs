use std::collections::HashMap;
use crate::deserialize::chunk_reading::{DataReader, GMChunk};
use crate::deserialize::general_info::{GMVersion, GMVersionLTS};


macro_rules! try_check {
    ($chunk:expr, $check:ident) => {
        if let Some(ver) = $check($chunk)? { return Ok(Some(ver)) }
    };
}

fn clone_chunk<'a>(chunk: &GMChunk<'a>) -> GMChunk<'a> {
    GMChunk {
        name: chunk.name.clone(),
        abs_pos: chunk.abs_pos,
        data: chunk.data,   // data is a reference; so this operation is not very expensive
        cur_pos: 0,         // automatically set to zero
        total_data_len: chunk.total_data_len,
    }
}

fn get_chunk_required<'a>(chunks: &HashMap<String, GMChunk<'a>>, chunk_name: &'static str) -> Result<GMChunk<'a>, String> {
    // cloning chunk reader is fast and prevents having to preserve original chunk position
    if let Some(chunk) = chunks.get(chunk_name) {
        Ok(clone_chunk(chunk))
    } else {
        Err(format!("Could not get required chunk '{chunk_name}' in HashMap with length {}", chunks.len()))
    }
}

fn get_chunk_optional<'a>(chunks: &HashMap<String, GMChunk>, chunk_name: &'static str) -> Option<GMChunk<'a>> {
    chunks.get(chunk_name).map(clone_chunk)
}

// TODO LTS stuff :c
// TODO in utmt, the CheckFor202X_X methods check the "current" version in the beginning
// cc means check_chunk

pub fn detect_gamemaker_version(reader: &mut DataReader) -> Result<Option<GMVersion>, String> {
    if reader.chunks.contains_key("UILR") {
        return Ok(Some(GMVersion::new(2024, 13, 0, 0, GMVersionLTS::Post2022_0)))
    }
    
    let mut chunk_sond: GMChunk = get_chunk_required(chunks, "SOND")?;
    try_check!(&mut chunk_sond, cc_sond_2024_6);
    
    let mut chunk_sprt: GMChunk = get_chunk_required(chunks, "SPRT")?;
    try_check!(&mut chunk_sprt, cc_sprt_2024_6);
    
    let mut chunk_extn: GMChunk = get_chunk_required(chunks, "EXTN")?;
    try_check!(&mut chunk_extn, cc_extn_2023_4);
    
    if chunks.contains_key("PSEM") {
        return Ok(Some(GMVersion::new(2023, 2, 0, 0, GMVersionLTS::Post2022_0)))
    }
    if chunks.contains_key("FEAT") {
        return Ok(Some(GMVersion::new(2022, 8, 0, 0, GMVersionLTS::Pre2022_0)))
    }
    
    let mut chunk_font: GMChunk = get_chunk_required(chunks, "FONT")?;
    try_check!(&mut chunk_font, cc_font_2022_2);

    if chunks.contains_key("FEDS") {
        return Ok(Some(GMVersion::new(2, 3, 6, 0, GMVersionLTS::Pre2022_0)))
    }
    if chunks.contains_key("SEQN") {
        return Ok(Some(GMVersion::new(2, 3, 0, 0, GMVersionLTS::Pre2022_0)))
    }
    if chunks.contains_key("TGIN") {
        return Ok(Some(GMVersion::new(2, 2, 1, 0, GMVersionLTS::Pre2022_0)))
    }
    
    // TODO implement rest
    Ok(None)
}


fn cc_extn_2023_4(chunk: &mut GMChunk) -> Result<Option<GMVersion>, String> {
    let target_ver = Ok(Some(GMVersion::new(2023, 4, 0, 0)));
    let ext_count = chunk.read_i32()?;
    if ext_count < 1 {
        return Ok(None)
    }
    // go to first extension and skip the minimal amount of strings
    chunk.cur_pos = chunk.read_relative_pointer()? + 4*3;
    let files_pointer: u32 = chunk.read_u32()?;
    let options_pointer: u32 = chunk.read_u32()?;
    // The file list pointer should be less than the option list pointer.
    // If it's not true, then "files_pointer" is actually a string pointer, so it's GM 2023.4+.
    if files_pointer > options_pointer {
        return target_ver
    }
    Ok(None)
}

fn cc_sond_2024_6(reader: &mut DataReader) -> Result<Option<GMVersion>, String> {
    // {~~} check lts shit
    let target_ver = Ok(Some(GMVersion::new(2024, 6, 0, 0, GMVersionLTS::Post2022_0)));
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

fn cc_sprt_2024_6(chunk: &mut GMChunk) -> Result<Option<GMVersion>, String> {
    // {~~} assert >= 2023.2 nonlts
    let target_ver = Ok(Some(GMVersion::new(2024, 6, 0, 0)));
    let sprite_count: usize = chunk.read_usize_count()?;
    for i in 0..sprite_count {
        chunk.cur_pos = i*4 + 4;
        let sprite_pointer: usize = chunk.read_relative_pointer()?;
        if sprite_pointer == 0 { continue }

        let mut next_sprite_pointer: usize = 0;
        for _ in i+1..sprite_count {
            let pointer = chunk.read_relative_pointer()?;
            if pointer != 0 {
                next_sprite_pointer = pointer;
                break
            }
        }

        chunk.cur_pos += 4;     // Skip past "Name"
        // Check if bbox size differs from width/height
        let width = chunk.read_u32()?;
        let height = chunk.read_u32()?;
        let margin_left = chunk.read_i32()?;
        let margin_right = chunk.read_i32()?;
        let margin_bottom = chunk.read_i32()?;
        let margin_top = chunk.read_i32()?;
        let bbox_width = (margin_right - margin_left + 1) as u32;
        let bbox_height = (margin_bottom - margin_top + 1) as u32;
        if bbox_width == width && bbox_height == height {
            continue    // We can't determine anything from this sprite
        }
        chunk.cur_pos += 28;
        if chunk.read_i32()? != -1 {    // not special type
            continue    // or return?
        }
        let special_version = chunk.read_u32()?;
        if special_version != 3 {
            continue    // or return?
        }
        let sprite_type = chunk.read_u32()?;
        if sprite_type != 0 {   // 0 <=> GMSpriteType::Normal
            continue    // We can't determine anything from this sprite
        }
        let sequence_offset = chunk.read_usize()?;
        let nine_slice_offset = chunk.read_usize()?;
        let texture_count = chunk.read_usize_count()?;
        chunk.cur_pos += texture_count*4;   // Skip past texture pointers
        let mask_count = chunk.read_usize_count()?;
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

        let full_end_pos = chunk.cur_pos + chunk.abs_pos + full_length;
        let bbox_end_pos = chunk.cur_pos + chunk.abs_pos + bbox_length;
        let expected_end_offset: usize;
        if sequence_offset != 0 {
            expected_end_offset = sequence_offset;
        } else if nine_slice_offset != 0 {
            expected_end_offset = nine_slice_offset;
        } else if next_sprite_pointer != 0 {
            expected_end_offset = next_sprite_pointer;
        } else {
            // Use chunk length, and be lenient with it (due to chunk padding)
            let expected_end = chunk.cur_pos + chunk.abs_pos + chunk.data.len();
            if full_end_pos%16 != 0 && full_end_pos + (16 - full_end_pos%16) == expected_end {
                return Ok(None)   // "Full" mask data doesn't exactly line up, but works if rounded up to the next chunk padding
            }
            if bbox_end_pos%16 != 0 && bbox_end_pos + (16 - bbox_end_pos%16) == expected_end {
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

fn cc_font_2022_2(chunk: &mut GMChunk) -> Result<Option<GMVersion>, String> {
    // {~~} return if bytecode<17 or at gm >= 2022.2
    let target_ver = Ok(Some(GMVersion::new(2022, 2, 0, 0)));
    let possible_font_count: usize = chunk.read_usize_count()?;
    if possible_font_count < 1 {
        return Ok(None)
    }

    let mut first_font_pointer: usize = 0;
    for _ in 0..possible_font_count {
        let pointer: usize = chunk.read_relative_pointer()?;
        if pointer != 0 {
            first_font_pointer = pointer;
            break
        }
    };
    if first_font_pointer == 0 {
        return Ok(None)
    }

    chunk.cur_pos = first_font_pointer + 48;
    let glyph_count: usize = chunk.read_usize()?;
    if glyph_count*4 > chunk.data.len() {
        return Ok(None)
    }
    if glyph_count == 0 {
        return target_ver   // seems stupid but i think the logic in utmt is equivalent to this
    }

    let mut glyph_pointers = Vec::with_capacity(glyph_count);
    for _ in 0..glyph_count {
        let pointer: usize = chunk.read_relative_pointer()?;
        if pointer == 0 {
            return Err("One of the glyph pointers is null?".to_string())
        }
        glyph_pointers.push(pointer);
    }
    for pointer in glyph_pointers {
        if chunk.cur_pos != pointer {
            return Ok(None)
        }
        chunk.cur_pos += 14;
        let kerning_length: u16 = chunk.read_u16()?;
        chunk.cur_pos += kerning_length as usize * 4;
        // from utmt: "combining read/write would apparently break" ???
    }

    target_ver
}

