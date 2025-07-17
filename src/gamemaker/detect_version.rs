use crate::utility::{num_enum_from, vec_with_capacity};
use crate::gamemaker::deserialize::{DataReader, GMChunk};
use crate::gamemaker::elements::embedded_textures::MAGIC_BZ2_QOI_HEADER;
use crate::gamemaker::elements::rooms::GMRoomLayerType;
use crate::gamemaker::gm_version::{GMVersion, GMVersionReq};
use crate::gamemaker::gm_version::LTSBranch::{PostLTS, PreLTS, LTS};

/// If `check_fn` can detect multiple versions, `required_version` should be set to its _lowest_ required version
/// whereas `target_version` should be set to the _highest_ possible version it can detect.
fn try_check<R: Into<GMVersionReq>, T: Into<GMVersionReq>>(
    reader: &mut DataReader,
    chunk_name: &str,
    check_fn: fn(&mut DataReader) -> Result<Option<GMVersionReq>, String>,
    required_version: R,
    target_version: T,
) -> Result<(), String> {
    let target_version: GMVersionReq = target_version.into();
    if reader.general_info.is_version_at_least(target_version.clone()) {
        return Ok(())    // no need to check if already
    }

    if let Some(chunk) = reader.chunks.get(chunk_name) {
        let required_version: GMVersionReq = required_version.into();
        if !reader.general_info.is_version_at_least(required_version.clone()) {
            return Ok(())   // could not detect version
        }
        reader.chunk = chunk.clone();
        reader.cur_pos = chunk.start_pos;
        if let Some(version_req) = check_fn(reader)? {
            log::debug!("Manually checking for version {} in chunk '{}' successful; upgraded from version {}", version_req, chunk_name, reader.general_info.version);
            reader.general_info.set_version_at_least(version_req.clone())?;
        }
    }
    Ok(())
}


#[derive(Debug, Clone)]
struct VersionCheck {
    chunk_name: &'static str,
    checker_fn: fn(&mut DataReader) -> Result<Option<GMVersionReq>, String>,
    /// The (lowest) gamemaker version required for the checker to perform the detection 
    required_version: GMVersionReq,
    /// The (highest) gamemaker version the checker can detect
    target_version: GMVersionReq,
}
impl VersionCheck {
    fn new<R: Into<GMVersionReq>, V: Into<GMVersionReq>>(
        chunk_name: &'static str,
        checker_fn: fn(&mut DataReader) -> Result<Option<GMVersionReq>, String>,
        required_version: R,
        target_version: V,
    ) -> Self {
        Self {
            chunk_name,
            checker_fn,
            required_version: required_version.into(),
            target_version: target_version.into(),
        }
    }
}


pub fn detect_gamemaker_version(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let original_version: GMVersion = reader.general_info.version.clone();
    let saved_pos: usize = reader.cur_pos;
    let saved_chunk: GMChunk = reader.chunk.clone();
    
    if reader.chunks.contains_key("TGIN") {
        reader.general_info.set_version_at_least((2, 2, 1, PreLTS))?;
    }
    if reader.chunks.contains_key("SEQN") {
        reader.general_info.set_version_at_least((2, 3, PreLTS))?;
    }
    if reader.chunks.contains_key("FEDS") {
        reader.general_info.set_version_at_least((2, 3, 6, PreLTS))?;
    }
    if reader.chunks.contains_key("FEAT") {
        reader.general_info.set_version_at_least((2022, 8, PreLTS))?;
    }
    if reader.chunks.contains_key("PSEM") {
        reader.general_info.set_version_at_least((2023, 2, PostLTS))?;
    }
    if reader.chunks.contains_key("UILR") {
        reader.general_info.set_version_at_least((2024, 13, PostLTS))?;
    }

    // cv means 'check version'
    if reader.general_info.bytecode_version >= 14 {
        try_check(reader, "FUNC", cv_func_2024_8, GMVersionReq::none(), (2024, 8))?;
    }
    if reader.general_info.bytecode_version >= 17 {
        try_check(reader, "FONT", cv_font_2022_2, GMVersionReq::none(), (2022, 2))?;
    }
    
    let mut checks: Vec<VersionCheck> = vec![
        VersionCheck::new("ACRV", cv_acrv_2_3_1, (2, 3), (2, 3, 1)),
        VersionCheck::new("PSEM", cv_psem_2023_x, (2023, 2), (2023, 8)),
        VersionCheck::new("TXTR", cv_txtr_2_0_6, (2, 0), (2, 0, 6)),
        VersionCheck::new("TGIN", cv_tgin_2022_9, (2, 3), (2022, 9)),
        VersionCheck::new("SPRT", cv_sprt_2_3_2, (2, 0), (2, 3, 2)),
        VersionCheck::new("OBJT", cv_objt_2022_5, (2, 3), (2022, 5)),
        VersionCheck::new("TGIN", cv_tgin_2023_1, (2022, 9), (2023, 1)),
        VersionCheck::new("EXTN", cv_extn_2023_4, (2022, 6), (2023, 4)),
        VersionCheck::new("AGRP", cv_agrp_2024_14, (2024, 13), (2024, 14)),
        VersionCheck::new("FONT", cv_font_2024_14, (2024, 13), (2024, 14)),
        VersionCheck::new("TXTR", cv_txtr_2022_3, (2, 3), (2022, 3)),
        VersionCheck::new("TXTR", cv_txtr_2022_5, (2022, 3), (2022, 5)),
        VersionCheck::new("EXTN", cv_extn_2022_6, (2, 3), (2022, 6)),
        VersionCheck::new("ROOM", cv_room_2_2_2_302, (2, 0), (2, 2, 2, 302)),
        VersionCheck::new("ROOM", cv_room_2024_2_and_2024_4, (2023, 2), (2024, 4)),
        VersionCheck::new("ROOM", cv_room_2022_1, (2, 3), (2022, 1)),
        VersionCheck::new("FONT", cv_font_2023_6_and_2024_11, (2022, 8), (2023, 6)),
        VersionCheck::new("FONT", cv_font_2023_6_and_2024_11, (2024, 6), (2024, 11)),
        VersionCheck::new("SPRT", cv_sprt_2024_6, (2022, 2, PostLTS), (2024, 6)),
        VersionCheck::new("SOND", cv_sond_2024_6, (2022, 2, PostLTS), (2024, 6)),
        VersionCheck::new("CODE", cv_code_2023_8_and_2024_4, GMVersionReq::none(), (2024, 4)),
    ];
    
    loop {
        // permanently filter out already detected versions
        checks.retain(|i| !reader.general_info.is_version_at_least(i.target_version.clone()));

        let mut updated_version: bool = false;
        let mut checks_to_remove: Vec<bool> = vec![false; checks.len()];
        
        for (i, check) in checks.iter().enumerate().rev() {
            // for this iteration, filter out versions whose version requirements are not (yet) met
            if !reader.general_info.is_version_at_least(check.required_version.clone()) {
                continue
            }

            // permanently remove check; no matter if successful or not
            checks_to_remove[i] = true;

            // if chunk doesn't exist; just skip the check
            let Some(chunk) = reader.chunks.get(check.chunk_name) else {continue};
            reader.chunk = chunk.clone();
            reader.cur_pos = reader.chunk.start_pos;
            
            let detected_version_opt: Option<GMVersionReq> = (check.checker_fn)(reader)
                .map_err(|e| format!("{e}\n↳ while trying to detect version {} in chunk '{}'", check.target_version, check.chunk_name))?;

            if let Some(detected_version) = detected_version_opt {
                log::debug!("Checking for version {} in chunk '{}' successful; upgraded from version {}",
                    detected_version, check.chunk_name, reader.general_info.version);
                reader.general_info.set_version_at_least(detected_version)?;
                updated_version = true;
            }
        }

        // remove all performed checks
        for (i, should_remove) in checks_to_remove.into_iter().enumerate().rev() {
            if should_remove {
                checks.remove(i);
            }
        }
        
        if !updated_version {
            break   // no more checks left; stop
        }
    }

    if reader.general_info.is_version_at_least((2023, 1)) && reader.general_info.version.branch == PreLTS {
        reader.general_info.version.branch = LTS;
    }
    
    reader.cur_pos = saved_pos;
    reader.chunk = saved_chunk;
    let ver: &GMVersion = &reader.general_info.version;
    if *ver == original_version {
        return Ok(None)
    }
    if original_version.branch == ver.branch {
        Ok(Some((ver.major, ver.minor, ver.release, ver.build).into()))
    } else {
        Ok(Some((ver.major, ver.minor, ver.release, ver.build, ver.branch).into()))
    }
}


fn cv_extn_2022_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
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
fn cv_extn_2023_4(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
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
fn cv_sond_2024_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
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
fn cv_agrp_2024_14(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
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
fn cv_sprt_2024_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
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
            log::warn!("full_end_pos == expected_end_offset while detecting SPRT_2024.6; may lead to false positives");
            return Ok(None)   // "Full" mask data is valid   (TODO no idea why it returns here tbh; check if there is bug in utmt pls)
        }
        if bbox_end_pos == expected_end_offset {
            return target_ver   // "Bbox" mask data is valid
        }
    }

    Ok(None)
}


fn cv_font_2022_2(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
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
    if glyph_count * 4 > reader.get_chunk_length() {
        return Ok(None)
    }
    if glyph_count == 0 {
        log::warn!("Glyph count is zero while detecting FONT_2022.2; may lead to false positives");
        return target_ver   // UTMT also assumes that it is 2022.2; even if there are no glyphs
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



/// We already know whether the version is more or less than 2022.8 due to the FEAT chunk being present.
/// Taking advantage of that, this is basically the same as the 2022.2 check, but it:
/// - Checks for the LineHeight value instead of Ascender (added in 2023.6)
///     PSEM (2023.2) is not used, as it would return a false negative on LTS (2022.9+ equivalent with no particles).
/// - Checks for UnknownAlwaysZero in Glyphs (added in 2024.11)
///     It's possible for the null pointer check planted in UTPointerList deserialisation to not be triggered:
///     for example, if SDF is enabled for any fonts, the shaders related to SDF will not be stripped;
///     it's also possible to prevent audiogroup_default from being stripped by doing
///         audio_group_name(audiogroup_default)
///     So we check for the presence of UnknownAlwaysZero as a last resort.
fn cv_font_2023_6_and_2024_11(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    // explicit check because the logic is very scuffed
    if !reader.general_info.is_version_at_least((2022, 8)) {
        return Ok(None)     // version requirement (for checking 2023.6) not satisfied
    }
    if reader.general_info.is_version_at_least((2023, 6)) && !reader.general_info.is_version_at_least((2024, 6)) {
        return Ok(None)     // 2023.6 already detected; but 2024.6 not yet detected
    }
    if reader.general_info.is_version_at_least((2024, 11)) {
        return Ok(None)     // 2024.11 already detected
    }

    let possible_font_count = reader.read_i32()?;
    let mut first_two_pointers: Vec<usize> = Vec::with_capacity(2);
    for _ in 0..possible_font_count {
        let ptr = reader.read_usize()?;
        if ptr == 0 {
            continue
        }
        first_two_pointers.push(ptr);
        if first_two_pointers.len() >= 2 {
            break
        }
    }
    if first_two_pointers.len() < 1 {
        return Ok(None)     // nothing to detect
    }
    if first_two_pointers.len() == 1 {
        // Add in the position of the padding i.e. the end of the font list
        first_two_pointers.push(reader.chunk.end_pos - 512);
    }

    reader.cur_pos = first_two_pointers[0] + 52;    // Also the LineHeight value. 48 + 4 = 52
    if reader.general_info.is_version_at_least((2023, 2, 0, 0, PostLTS)) {
        // SDFSpread is present from 2023.2 non-LTS onward
        reader.cur_pos += 4;    // (detected by PSEM/PSYS chunk existence)
    }

    let glyph_count: usize = reader.read_usize()?;
    if glyph_count * 4 > first_two_pointers[1] - reader.cur_pos || glyph_count < 1 {
        return Ok(None)
    }

    let mut glyph_pointers: Vec<usize> = vec_with_capacity(glyph_count)?;
    for _ in 0..glyph_count {
        let ptr = reader.read_usize()?;
        if ptr == 0 {
            return Err("One of the glyph pointers is zero".to_string())
        }
        glyph_pointers.push(ptr);
    }

    // let mut detecting_2024_11_failed: bool = false;
    for (i, glyph_pointer) in glyph_pointers.iter().enumerate() {
        if reader.cur_pos != *glyph_pointer {
            return Ok(None)
        }
        reader.cur_pos += 14;
        let kerning_count = reader.read_u16()?;

        // Hopefully the last thing in a UTFont is the glyph list
        let next_glyph_pointer = if i < glyph_pointers.len()-1 { glyph_pointers[i+1] } else { first_two_pointers[1] };
        // And hopefully the last thing in a glyph is the kerning list
        // Note that we're actually skipping all items of the Glyph.Kerning SimpleList here;
        // 4 is supposed to be the size of a GlyphKerning object
        let pointer_after_kerning_list = reader.cur_pos + 4*kerning_count as usize;
        // If we don't land on the next glyph/font after skipping the Kerning list,
        // kerningLength is probably bogus and UnknownAlwaysZero may be present
        if next_glyph_pointer == pointer_after_kerning_list {
            return Ok(Some((2023, 6).into()))   // 2023.6 succeeded; 2024.11 failed
        }
        // Discard last read, which would be of UnknownAlwaysZero
        let kerning_count: u16 = reader.read_u16()?;
        let pointer_after_kerning_list = reader.cur_pos + 4*kerning_count as usize;
        if next_glyph_pointer != pointer_after_kerning_list {
            return Err(
                "There appears to be more/less values than UnknownAlwaysZero before \
                the kerning list in GMFontGlyph; data file potentially corrupted".to_string()
            )
        }
        return Ok(Some((2024, 11).into()))      // 2024.11 succeeded (2023.6 did too but doesn't matter)
    }

    Ok(Some((2023, 6).into()))  // 2024.11 failed or could not be detected; 2023.6 succeeded
}


fn cv_font_2024_14(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    // Check for new padding added (and final chunk "padding" removed) in 2024.14
    let font_count = reader.read_u32()?;
    let mut last_font_position: usize = 0;
    for _ in 0..font_count {
        let ptr = reader.read_usize()?;
        if ptr != 0 {
            last_font_position = ptr;
        }
    }

    // If we have a last font, advance to the end of its data (ignoring the new alignment added in 2024.14)
    if last_font_position != 0 {
        reader.cur_pos = last_font_position + 56;

        // Advance to last glyph in pointer list
        let glyph_count = reader.read_usize()?;
        reader.cur_pos += (glyph_count - 1) * 4;
        reader.cur_pos = reader.read_usize()? + 16;

        // Advance past kerning
        let kerning_count = reader.read_u16()?;
        reader.cur_pos += kerning_count as usize * 4;
    }

    // Check for the final chunk padding being missing
    if reader.cur_pos + 512 > reader.chunk.end_pos {
        // No padding can fit, so this is 2024.14
        return Ok(Some((2024, 14).into()))
    }

    Ok(None)
}


fn cv_objt_2022_5(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2022, 5).into()));
    let object_count = reader.read_u32()?;
    if object_count < 1 {
        return Ok(None)     // no objects; nothing to detect
    }
    let first_object_pointer = reader.read_usize()?;
    reader.cur_pos = first_object_pointer + 64;
    let vertex_count = reader.read_usize()?;

    if reader.cur_pos + 12 + 8*vertex_count >= reader.chunk.end_pos {
        return target_ver      // Bounds check on vertex data "failed" => 2022.5
    }

    reader.cur_pos += 12 + 8*vertex_count;
    if reader.read_u32()? == 15 {   // !! 15 has to equal variant count of GMGameObjectEventType enum !!
        let sub_event_pointer = reader.read_usize()?;
        if reader.cur_pos + 56 == sub_event_pointer {
            return Ok(None)     // subevent pointer check "succeeded" (Should start right after the list) => not 2022.5
        }
    }

    target_ver
}


fn cv_room_2022_1(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2022, 1).into()));
    // Iterate over all rooms until a length check is performed

    let room_count: usize = reader.read_usize()?;
    for room_index in 0..room_count {
        // Advance to room data we're interested in (and grab pointer for next room)
        reader.set_rel_cur_pos(4*room_index + 4)?;
        let room_pointer = reader.read_usize()?;
        reader.cur_pos = room_pointer + 22*4;

        // Get the pointer for this room's layer list, as well as pointer to sequence list
        let layer_list_pointer = reader.read_usize()?;
        let sequence_pointer = reader.read_usize()?;
        reader.cur_pos = layer_list_pointer;
        let layer_count = reader.read_i32()?;
        if layer_count < 1 {
            continue    // no layers to detect; go to next room
        }

        // Get pointer into the individual layer data (plus 8 bytes) for the first layer in the room
        let jump_pointer = reader.read_usize()? + 8;

        // Find the offset for the end of this layer
        let next_pointer = if layer_count == 1 {
            sequence_pointer
        } else {
            reader.read_usize()?    // pointer to next element in the layer list
        };

        // Actually perform the length checks, depending on layer data
        reader.cur_pos = jump_pointer;
        let layer_type = reader.read_u32()?;
        let Ok(layer_type) = GMRoomLayerType::try_from(layer_type) else { continue };

        match layer_type {
            GMRoomLayerType::Path | GMRoomLayerType::Path2 => continue,
            GMRoomLayerType::Background => if next_pointer - reader.cur_pos > 16*4 {
                return target_ver
            }
            GMRoomLayerType::Instances => {
                reader.cur_pos += 6*4;
                let instance_count = reader.read_usize()?;
                if next_pointer - reader.cur_pos != instance_count*4 {
                    return target_ver
                }
            }
            GMRoomLayerType::Assets => {
                reader.cur_pos += 6*4;
                let tile_pointer = reader.read_usize()?;
                if tile_pointer != reader.cur_pos+8 && tile_pointer != reader.cur_pos+12 {
                    return target_ver
                }
            }
            GMRoomLayerType::Tiles => {
                reader.cur_pos += 6*4;
                let tile_map_width = reader.read_usize()?;
                let tile_map_height = reader.read_usize()?;
                if next_pointer - reader.cur_pos != (tile_map_width * tile_map_height * 4) {
                    return target_ver
                }
            }
            GMRoomLayerType::Effect => {
                reader.cur_pos += 7*4;
                let property_count = reader.read_usize()?;
                if next_pointer - reader.cur_pos != (property_count * 3 * 4) {
                    return target_ver
                }
            }
        }
        return Ok(None)   // Check complete, found and tested a layer (but didn't detect 2022.1)
    }

    Ok(None)
}


fn cv_room_2_2_2_302(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    // Check the size of the first GameObject in a room
    let room_count = reader.read_usize()?;

    for room_index in 0..room_count {
        // Advance to room data we're interested in (and grab pointer for next room)
        reader.set_rel_cur_pos(4*room_index + 4)?;
        let room_pointer = reader.read_usize()?;
        reader.cur_pos = room_pointer + 12*4;

        // Get the pointer for this room's object list, as well as pointer to tile list
        let object_list_pointer = reader.read_usize()?;
        let tile_list_pointer = reader.read_usize()?;
        reader.cur_pos = object_list_pointer;
        let object_count = reader.read_usize()?;
        if object_count < 1 {
            continue    // no objects => nothing to detect; go to next room
        }

        let pointer1 = reader.read_usize()?;
        let pointer2 = if object_count == 1 {
            tile_list_pointer   // Tile list starts right after, so it works as an alternate
        } else {
            reader.read_usize()?
        };
        if pointer2 - pointer1 == 48 {
            return Ok(Some((2, 2, 2, 302).into()))
        }
    }

    Ok(None)
}


fn cv_room_2024_2_and_2024_4(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    // check for tile compression
    let room_count: usize = reader.read_usize()?;
    let mut any_layers_misaligned: bool = false;

    for room_index in 0..room_count {
        // Advance to room data we're interested in (and grab pointer for next room)
        reader.set_rel_cur_pos(4*room_index + 4)?;
        let room_pointer = reader.read_usize()?;
        reader.cur_pos = room_pointer + 22*4;

        // Get the pointer for this room's layer list, as well as pointer to sequence list
        let layer_list_ptr = reader.read_usize()?;
        let sequence_ptr = reader.read_usize()?;
        reader.cur_pos = layer_list_ptr;
        let layer_count = reader.read_usize()?;
        if layer_count < 1 {
            continue    // no layers to detect; go to next room
        }

        let mut check_next_layer_offset: bool = false;
        for layer_index in 0..layer_count {
            let layer_ptr = layer_list_ptr + 4*layer_index;
            if check_next_layer_offset && layer_ptr %4 != 0 {
                any_layers_misaligned = true;
            }

            reader.cur_pos = layer_ptr + 4;
            // Get pointer into the individual layer data
            let layer_data_ptr: usize = reader.read_usize()?;

            // Find the offset for the end of this layer
            let next_pointer: usize = if layer_index == layer_count - 1 {
                sequence_ptr
            } else {
                reader.read_usize()?   // pointer to next element in the layer list
            };

            // Actually perform the length checks
            reader.cur_pos = layer_data_ptr + 8;
            let layer_type: GMRoomLayerType = num_enum_from(reader.read_u32()?)?;
            if layer_type != GMRoomLayerType::Tiles {
                check_next_layer_offset = false;
                continue
            }
            check_next_layer_offset = true;
            reader.cur_pos += 32;
            let effect_count: usize = reader.read_usize()?;
            reader.cur_pos += 12*effect_count + 4;

            let tile_map_width = reader.read_usize()?;
            let tile_map_height = reader.read_usize()?;
            if next_pointer - reader.cur_pos != (tile_map_width * tile_map_height * 4) {
                return if any_layers_misaligned {
                    Ok(Some((2024, 2).into()))
                } else {
                    Ok(Some((2024, 4).into()))
                }
            }
        }
    }
    
    Ok(None)
}


fn cv_func_2024_8(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2024, 8).into()));
    if reader.get_chunk_length() == 0 {
        return Ok(None)
    }
    
    // The CodeLocals list was removed in 2024.8, so we check if Functions is the only thing in here.
    let function_count = reader.read_usize()?;
    // Skip over the (Simple)List
    // (3*4 is the size of a GMFunction object)
    reader.cur_pos += function_count * 3*4;

    if reader.cur_pos == reader.chunk.end_pos {
        // Directly reached the end of the chunk after the function list, so code locals are definitely missing
        return target_ver
    }

    // align position
    let mut padding_bytes_read: u32 = 0;

    while reader.cur_pos & (reader.chunk_padding - 1) != 0 {
        if reader.cur_pos >= reader.chunk.end_pos || reader.read_u8()? != 0 {
            return Ok(None)   // If we hit a non-zero byte (or exceed chunk boundaries), it can't be padding
        }
        padding_bytes_read += 1;
    }

    // If we're at the end of the chunk after aligning padding, code locals are either empty or do not exist altogether.
    if reader.cur_pos != reader.chunk.end_pos {
        return Ok(None)
    }

    if padding_bytes_read < 4 {
        return target_ver
    }

    // If we read at least 4 padding bytes, we don't know for sure unless we have at least one code entry.
    if let Some(chunk_code) = reader.chunks.get("CODE") {
        reader.chunk = chunk_code.clone();
        reader.cur_pos = chunk_code.start_pos;
        let code_count = reader.read_usize()?;
        if code_count < 1 {
            return Ok(None)
        }
    }

    target_ver
}


fn cv_txtr_2022_3(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2022, 3).into()));
    let texture_count = reader.read_usize()?;
    if texture_count < 1 {
        return Ok(None)    // can't detect if there are no texture pages
    }
    if texture_count == 1 {
        reader.cur_pos += 16;   // Jump to either padding or length, depending on version
        if reader.read_u32()? > 0 {   // Check whether it's padding or length
            return target_ver
        }
    } else {
        let pointer1 = reader.read_usize()?;
        let pointer2 = reader.read_usize()?;
        if pointer1 + 16 == pointer2 {
            return target_ver
        }
    }

    Ok(None)
}


fn cv_txtr_2022_5(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let target_ver = Ok(Some((2022, 5).into()));
    let texture_count = reader.read_usize()?;
    for i in 0..texture_count {
        // Go to each texture, and then to each texture's data
        reader.cur_pos = 4*i + 4;
        reader.cur_pos = reader.read_usize()? + 12;    // go to texture; at an offset
        reader.cur_pos = reader.read_usize()?;    // go to texture data
        let header: &[u8; 4] = reader.read_bytes_const()?;
        if header != MAGIC_BZ2_QOI_HEADER {
            continue    // Nothing useful, check the next texture
        }
        reader.cur_pos += 4;    // skip width/height
        // now check actual bz2 headers
        if reader.read_bytes_const::<3>()? != b"BZh" {
            return target_ver
        }
        reader.cur_pos += 1;
        if *reader.read_bytes_const::<6>()? != [0x31, 0x41, 0x59, 0x26, 0x53, 0x59] {   // Digits of pi (block header)
            return target_ver
        }
        return Ok(None)  // if first bzip2+qoi texture page version check was unsuccessful, don't bother with other ones
    }

    Ok(None)
}


fn cv_txtr_2_0_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let texture_count = reader.read_usize()?;
    if texture_count < 1 {
        return Ok(None)
    }
    if texture_count == 1 {
        // Go to the first texture pointer (+ minimal texture entry size)
        reader.cur_pos = reader.read_usize()? + 8;
        if reader.read_u32()? == 0 {
            return Ok(None)   // If there is a zero instead of texture data pointer; it's not 2.0.6
        }
    }
    if texture_count >= 2 {
        let pointer1 = reader.read_u32()?;
        let pointer2 = reader.read_u32()?;
        if pointer2 - pointer1 == 8 {   // "Scaled" + "_textureData" -> 8
            return Ok(None)
        }
    }

    Ok(Some((2, 0, 6).into()))
}


fn cv_tgin_2022_9(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    if reader.general_info.is_version_at_least((2023, 1, PostLTS)) {
        return Ok(None)
    }
    
    let tgin_version: u32 = reader.read_u32()?;
    if tgin_version != 1 {
        return Err(format!("Expected TGIN version 1; got {tgin_version}"))
    }
    
    let tgin_count = reader.read_usize()?;
    if tgin_count < 1 {
        return Ok(None)
    }
    let pointer1 = reader.read_usize()?;
    let pointer2 = if tgin_count >= 2 { reader.read_usize()? } else { reader.chunk.end_pos };
    reader.cur_pos = pointer1 + 4;

    // Check to see if the pointer located at this address points within this object
    // If not, then we know we're using a new format!
    let ptr = reader.read_usize()?;
    if ptr < pointer1 || ptr >= pointer2 {
        return Ok(Some((2022, 9).into()))
    }
    
    Ok(None)
}


fn cv_tgin_2023_1(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    if reader.general_info.is_version_at_least((2023, 1, PostLTS)) {
        return Ok(None)
    }

    let tgin_version: u32 = reader.read_u32()?;
    if tgin_version != 1 {
        return Err(format!("Expected TGIN version 1; got {tgin_version}"))
    }

    let tgin_count = reader.read_usize()?;
    if tgin_count < 1 {
        return Ok(None)
    }
    let pointer1 = reader.read_usize()?;
    
    // Go to the 4th list pointer of the first TGIN entry.
    // (either to "Fonts" or "SpineTextures" depending on the version)
    reader.cur_pos = pointer1 + 16 + 4*3;
    let pointer4 = reader.read_usize()?;
    
    // If there's a "TexturePages" count instead of the 5th list pointer.
    // The count can't be greater than the pointer.
    // (the list could be either "Tilesets" or "Fonts").
    if reader.read_usize()? <= pointer4 {
        return Ok(Some((2023, 1, PostLTS).into()))
    }

    Ok(None)
}


fn cv_acrv_2_3_1(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let count = reader.read_u32()?;
    if count < 1 {
        return Ok(None)
    }
    
    // go to the first "point"
    reader.cur_pos = reader.read_usize()? + 8;
    for _ in 0..2 {
        if reader.read_u32()? != 0 {
            // In 2.3 an int with the value of 0 would be set here, it cannot be version 2.3 if this value isn't 0
            return Ok(Some((2, 3, 1).into()))
        }
    }
    
    Ok(None)
}


fn cv_sprt_2_3_2(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let pointers: Vec<usize> = reader.read_simple_list()?;
    for pointer in pointers {
        if pointer == 0 { continue }
        reader.cur_pos = pointer + 14*4;
        if reader.read_i32()? != -1 {
            continue        // sprite is not special type
        }
        let special_version: u32 = reader.read_u32()?;
        if special_version >= 3 {
            return Ok(Some((2, 3, 2).into()))
        }
    }
    Ok(None)
}


fn cv_psem_2023_x(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    let mut target_ver = None;
    reader.align(4)?;
    let psem_version = reader.read_u32()?;
    if psem_version != 1 {
        return Err(format!("Expected PSEM version 1; got {psem_version}"))
    }
    let count: u32 = reader.read_u32()?;
    if count < 11 {   // 2023.2 automatically adds eleven, later versions don't
        target_ver = Some((2023, 4).into());
    }
    if count == 0 {
        return Ok(target_ver)   // nothing more to detect
    }
    if count == 1 {
        match reader.chunk.end_pos - reader.chunk.start_pos {
            0xF8 => target_ver = Some((2023, 8).into()),
            0xD8 => target_ver = Some((2023, 6).into()),
            0xC8 => target_ver = Some((2023, 4).into()),
            elem_size => return Err(format!("Unrecognized PSEM size {elem_size} with only one element"))
        }
    } else {
        let pointer1 = reader.read_u32()?;
        let pointer2 = reader.read_u32()?;
        match pointer2 - pointer1 {
            0xEC => target_ver = Some((2023, 8).into()),
            0xC0 => target_ver = Some((2023, 6).into()),
            0xBC => target_ver = Some((2023, 4).into()),
            0xB0 => {},   // 2023.2
            elem_size => return Err(format!("Unrecognized PSEM size {elem_size} with {count} elements"))
        }
    }
    Ok(target_ver)
}


fn cv_code_2023_8_and_2024_4(reader: &mut DataReader) -> Result<Option<GMVersionReq>, String> {
    fn get_chunk_elem_count(reader: &mut DataReader, chunk_name: &'static str) -> Result<u32, String> {
        let chunk = match reader.chunks.get(chunk_name) {
            Some(c) => c,
            None => return Ok(0),
        };
        let saved_chunk = reader.chunk.clone();
        let saved_pos = reader.cur_pos;
        reader.chunk = chunk.clone();
        reader.cur_pos = chunk.start_pos;
        let count = reader.read_u32()?;
        reader.chunk = saved_chunk;
        reader.cur_pos = saved_pos;
        Ok(count)
    }
    fn get_chunk_elem_count_weird(reader: &mut DataReader, chunk_name: &'static str) -> Result<u32, String> {
        let chunk = match reader.chunks.get(chunk_name) {
            Some(c) => c,
            None => return Ok(0),
        };
        let saved_chunk = reader.chunk.clone();
        let saved_pos = reader.cur_pos;
        reader.chunk = chunk.clone();
        reader.cur_pos = chunk.start_pos;
        reader.align(4)?;
        if reader.read_u32()? != 1 {
            return Err("Expected version 1".to_string())
        }
        let count = reader.read_u32()?;
        reader.chunk = saved_chunk;
        reader.cur_pos = saved_pos;
        Ok(count)
    }

    let background_count = get_chunk_elem_count(reader, "BGND")?;
    let path_count = get_chunk_elem_count(reader, "PATH")?;
    let script_count = get_chunk_elem_count(reader, "SCPT")?;
    let font_count = get_chunk_elem_count(reader, "FONT")?;
    let timeline_count = get_chunk_elem_count(reader, "TMLN")?;
    let shader_count = get_chunk_elem_count(reader, "SHDR")?;
    let sequence_count = get_chunk_elem_count_weird(reader, "SEQN")?;
    let particle_system_count = get_chunk_elem_count_weird(reader, "SEQN")?;

    let check_if_asset_type_2024_4 = |reader: &mut DataReader| -> Result<bool, String> {
        let int_argument = reader.read_u32()?;
        let resource_id = int_argument & 0xffffff;
        Ok(match (int_argument >> 24) as u8 {
            4 => resource_id >= background_count,
            5 => resource_id >= path_count,
            6 => resource_id >= script_count,
            7 => resource_id >= font_count,
            8 => resource_id >= timeline_count,
            9 => true,  // used to be unused, now are sequences
            10 => resource_id >= shader_count,
            11 => resource_id >= sequence_count,
            // case 12 used to be animcurves, but now is unused (so would actually mean earlier than 2024.4)
            13 => resource_id >= particle_system_count,
            _ => false,
        })
    };

    let code_count = reader.read_usize()?;
    let mut code_pointers = vec_with_capacity(code_count)?;
    for _ in 0..code_count {
        let ptr: usize = reader.read_usize()?;
        if ptr != 0 {
            code_pointers.push(ptr);
        }
    }
    let mut detected_2023_8: bool = false;

    if reader.general_info.bytecode_version <= 14 {
        for code_ptr in code_pointers {
            reader.cur_pos = code_ptr + 4;
            let length = reader.read_usize()?;
            let end = reader.cur_pos + length;
            while reader.cur_pos < end {
                let first_word = reader.read_u32()?;
                let opcode = (first_word >> 24) as u8;
                let type1 = ((first_word & 0x00FF0000) >> 16) as u8;
                if matches!(opcode, 0x41|0xDA) {    // pop, call
                    reader.cur_pos += 4;
                }
                if matches!(opcode, 0xC0|0xC1|0xC2|0xC3) && type1 != 0x0f {   // push variants; account for int16
                    reader.cur_pos += 4;
                }
                if opcode != 0xFF {continue}    // break instruction
                if type1 == 2 {   // if type1 is int32
                    if check_if_asset_type_2024_4(reader)? {
                        return Ok(Some((2024, 4).into()))   //  return immediately if highest detectable version (2024.4) is found
                    } else {
                        detected_2023_8 = true;
                    }
                }
            }
        }
    } else {    // bytecode >= 15
        for code_ptr in code_pointers {
            reader.cur_pos = code_ptr + 4;  // skip name
            let instructions_length: usize = reader.read_usize()?;
            reader.cur_pos += 4;    // skip locals and arguments count
            let instructions_start_relative: i32 = reader.read_i32()?;
            let instructions_start: usize = (reader.cur_pos as i32 - 4 + instructions_start_relative) as usize;
            let instructions_end: usize = instructions_start + instructions_length;
            reader.cur_pos = instructions_start;
            
            while reader.cur_pos < instructions_end {
                let first_word = reader.read_u32()?;
                let opcode = (first_word >> 24) as u8;
                let type1 = ((first_word & 0x00FF0000) >> 16) as u8;
                if matches!(opcode, 0x45|0xD9) {    // pop, call
                    reader.cur_pos += 4;
                }
                if matches!(opcode, 0xC0|0xC1|0xC2|0xC3) && type1 != 0x0f {   // push variants; account for int16
                    reader.cur_pos += 4;
                }
                if opcode != 0xFF {continue}  // break instruction
                if type1 == 2 {   // if type1 is int32
                    if check_if_asset_type_2024_4(reader)? {
                        return Ok(Some((2024, 4).into()))   // return immediately if highest detectable version (2024.4) is found
                    } else {
                        detected_2023_8 = true;
                    }
                }
            }
        }
    }

    if detected_2023_8 {
        Ok(Some((2023, 8).into()))
    } else {
        Ok(None)
    }
}

