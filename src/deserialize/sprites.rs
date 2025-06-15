use crate::deserialize::chunk_reading::GMRef;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::sequence::{parse_sequence, GMAnimSpeedType, GMSequence};
use crate::deserialize::sprites_yyswf::{parse_yyswf_timeline, GMSpriteTypeSWF, GMSpriteYYSWFTimeline};
use crate::deserialize::strings::GMStrings;
use crate::deserialize::texture_page_items::{GMTexturePageItem, GMTextures};

#[derive(Debug, Clone, PartialEq)]
pub struct GMSprite {
    pub name: GMRef<String>,
    pub width: u32,
    pub height: u32,
    pub margin_left: i32,
    pub margin_right: i32,
    pub margin_bottom: i32,
    pub margin_top: i32,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub bbox_mode: i32,
    pub sep_masks: GMSpriteSepMaskType,
    pub origin_x: i32,
    pub origin_y: i32,
    pub textures: Vec<GMRef<GMTexturePageItem>>,
    pub collision_masks: Vec<GMSpriteMaskEntry>,
    pub special_fields: Option<GMSpriteSpecial>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GMSpriteType {
    Normal(GMSpriteTypeNormal),
    SWF(GMSpriteTypeSWF),
    Spine(GMSpriteTypeSpine),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteTypeNormal {
    pub collision_masks: Vec<GMSpriteMaskEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteTypeSpine {
    /// Spine version
    pub version: i32,
    pub cache_version: i32,
    pub has_texture_data: bool,
    pub textures: Vec<GMSpriteSplineTextureEntry>,
    pub json: String,
    pub atlas: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteSplineTextureEntry {
    pub page_width: i32,
    pub page_height: i32,
    /// empty for gmVersion >= 2023.1
    pub texture_blob: Vec<u8>,  // implementing Serialize for this probably isn't the best idea
    pub texture_entry_length: usize,
    pub is_qoi: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteNineSlice {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub enabled: bool,
    pub tile_modes: Vec<GMSpriteNineSliceTileMode>,
}

#[derive(Debug, Clone, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum GMSpriteNineSliceTileMode {
    Stretch = 0,
    Repeat = 1,
    Mirror = 2,
    BlankRepeat = 3,
    Hide = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteSpecial {
    /// Version of Special Thingy
    pub special_version: u32,
    pub sprite_type: GMSpriteType,
    /// GMS 2
    pub playback_speed: f32,
    /// GMS 2
    pub playback_speed_type: GMAnimSpeedType,
    /// Special Version 2
    pub sequence: Option<GMSequence>,
    /// Special Version 3
    pub nine_slice: Option<GMSpriteNineSlice>,
    /// SWF
    pub yyswf: Option<GMSpriteTypeSWF>,
}

#[derive(Debug, Clone)]
pub struct GMSprites {
    pub sprites_by_index: Vec<GMSprite>,
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum GMSpriteSepMaskType {
    AxisAlignedRect = 0,
    Precise = 1,
    RotatedRect = 2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteMaskEntry {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}


pub fn parse_chunk_sprt(
    chunk: &mut GMChunk,
    general_info: &GMGeneralInfo,
    strings: &GMStrings,
    gm_textures: &GMTextures,
) -> Result<GMSprites, String> {
    chunk.cur_pos = 0;
    let sprites_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(sprites_count);
    for _ in 0..sprites_count {
        start_positions.push(chunk.read_usize_pos()? - chunk.abs_pos);
    }

    let mut sprites_by_index: Vec<GMSprite> = Vec::with_capacity(sprites_count);
    for start_position in start_positions {
        chunk.cur_pos = start_position;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let name_resolved: &String = name.resolve(&strings.strings_by_index)?;
        let width: u32 = chunk.read_u32()?;
        let height: u32 = chunk.read_u32()?;
        let margin_left: i32 = chunk.read_i32()?;
        let margin_right: i32 = chunk.read_i32()?;
        let margin_bottom: i32 = chunk.read_i32()?;
        let margin_top: i32 = chunk.read_i32()?;
        let transparent: bool = chunk.read_bool32()?;
        let smooth: bool = chunk.read_bool32()?;
        let preload: bool = chunk.read_bool32()?;
        let bbox_mode: i32 = chunk.read_i32()?;
        let sep_masks: u32 = chunk.read_u32()?;
        let sep_masks: GMSpriteSepMaskType = sep_masks.try_into().map_err(|_| format!(
            "Invalid Sep Masks Type 0x{:08X} at position {} while parsing Sprite at position {} in chunk '{}'",
            sep_masks, chunk.cur_pos, start_position, chunk.name,
        ))?;
        let origin_x: i32 = chunk.read_i32()?;
        let origin_y: i32 = chunk.read_i32()?;
        let mut textures: Vec<GMRef<GMTexturePageItem>> = Vec::new();
        let mut collision_masks: Vec<GMSpriteMaskEntry> = Vec::new();
        let mut special_fields: Option<GMSpriteSpecial> = None;

        // combination of these conditions may be incorrect
        if chunk.read_i32()? == -1 && general_info.is_version_at_least(2, 0, 0, 0) {
            let special_version: u32 = chunk.read_u32()?;
            let special_sprite_type: u32 = chunk.read_u32()?;

            let mut sequence: Option<GMSequence> = None;
            let mut nine_slice: Option<GMSpriteNineSlice> = None;
            let yyswf: Option<GMSpriteTypeSWF> = None;

            let playback_speed: f32 = chunk.read_f32()?;
            let playback_speed_type: u32 = chunk.read_u32()?;
            let playback_speed_type: GMAnimSpeedType = playback_speed_type.try_into().map_err(|_| format!(
                "Invalid Playback Anim Speed Type 0x{:08X} at position {} while parsing Sprite at position {} in chunk '{}'",
                playback_speed_type, chunk.cur_pos, start_position, chunk.name,
            ))?;
            // both of these seem to be not an offset but instead an absolute position (see UndertaleModLib/Models/UndertaleSprite.cs@507)
            let sequence_offset: i32 = if special_version >= 2 { chunk.read_i32()? } else { 0 };
            let nine_slice_offset: i32 = if special_version >= 3 { chunk.read_i32()? } else { 0 };
            // {~~} set gms version to at least 2.3.2 if nine slice offset

            let sprite_type: GMSpriteType = match &special_sprite_type {
                0 => {      // Normal
                    textures = read_texture_list(chunk, gm_textures, name_resolved)?;
                    // read mask data
                    let mut mask_width = width as usize;
                    let mut mask_height = height as usize;
                    if general_info.is_version_at_least(2024, 6, 0, 0) {
                        mask_width = (margin_right - margin_left + 1) as usize;
                        mask_height = (margin_bottom - margin_top + 1) as usize;
                    }
                    let collision_masks: Vec<GMSpriteMaskEntry> = read_mask_data(chunk, name_resolved, mask_width, mask_height)?;
                    GMSpriteType::Normal(GMSpriteTypeNormal { collision_masks })
                },

                1 => {      // SWF
                    // [From UndertaleModTool] "This code does not work all the time for some reason"
                    let swf_version: i32 = chunk.read_i32()?;
                    // assert swf version is either 7 or 8
                    if !(swf_version == 7 || swf_version == 8) {
                        return Err(format!(
                            "Invalid SWF version {swf_version} for Sprite \"{}\" at absolute position {}",
                            name.display(strings), start_position + chunk.abs_pos,
                        ))
                    }
                    if swf_version == 8 {
                        textures = read_texture_list(chunk, gm_textures, name_resolved)?
                    }

                    // read YYSWF
                    chunk.align(4)?;
                    let jpeg_len: i32 = chunk.read_i32()? & (!0x80000000u32 as i32);    // the length is `OR`ed with int.MinValue
                    let jpeg_len: usize = jpeg_len as usize;
                    let yyswf_version: i32 = chunk.read_i32()?;
                    let jpeg_table: Vec<u8> = chunk.data.get(chunk.cur_pos.. chunk.cur_pos+jpeg_len).ok_or_else(|| format!(
                        "Trying to read YYSWF JPEG Table out of bounds while parsing \
                        Sprite with name \"{}\" in chunk '{}' at position {}: {} > {}",
                        name_resolved, chunk.name, chunk.cur_pos, chunk.cur_pos + jpeg_len, chunk.data.len(),
                    ))?.to_vec();
                    chunk.cur_pos += jpeg_len;
                    chunk.align(4)?;
                    let timeline: GMSpriteYYSWFTimeline = parse_yyswf_timeline(chunk, general_info)?;

                    GMSpriteType::SWF(GMSpriteTypeSWF {
                        swf_version,
                        yyswf_version,
                        jpeg_table,
                        timeline,
                    })
                },

                2 => {      // Spine
                    return Err(format!(
                        "Spine format is not yet implemented for Sprite with name \"{}\" and absolute position {}",
                        name.resolve(&strings.strings_by_index)?, start_position + chunk.abs_pos,
                    ))
                    // TODO {~~} IMPLEMENT TS
                }

                other => {
                    return Err(format!(
                        "Invalid Sprite Type {other} for Sprite with name \"{}\" and absolute position {}",
                        name_resolved, start_position + chunk.abs_pos,
                    ))
                }
            };

            if sequence_offset != 0 {
                let thingy: i32 = chunk.read_i32()?;
                if thingy != 1 {
                    return Err(format!(
                        "Expected 1 but got {} while parsing Sequence for Sprite with name \"{}\" in chunk '{}'",
                        thingy, name_resolved, chunk.name,
                    ))
                }
                sequence = Some(parse_sequence(chunk, general_info, strings)?);
            }

            if nine_slice_offset != 0 {
                nine_slice = Some(parse_nine_slice(chunk, name_resolved, start_position)?);
            }

            special_fields = Some(GMSpriteSpecial {
                special_version,
                sprite_type,
                playback_speed,
                playback_speed_type,
                sequence,
                nine_slice,
                yyswf,
            });
        } else {
            chunk.cur_pos -= 4;  // unread the not -1
            // read into `textures`
            textures = read_texture_list(chunk, gm_textures, name_resolved)?;
            // read mask data
            let mut mask_width = width as usize;
            let mut mask_height = height as usize;
            if general_info.is_version_at_least(2024, 6, 0, 0) {
                mask_width = (margin_right - margin_left + 1) as usize;
                mask_height = (margin_bottom - margin_top + 1) as usize;
            }
            collision_masks = read_mask_data(chunk, name_resolved, mask_width, mask_height)?;
        }

        sprites_by_index.push(GMSprite {
            name,
            width,
            height,
            margin_left,
            margin_right,
            margin_bottom,
            margin_top,
            transparent,
            smooth,
            preload,
            bbox_mode,
            sep_masks,
            origin_x,
            origin_y,
            textures,
            collision_masks,
            special_fields,
        })
    }


    Ok(GMSprites {sprites_by_index})
}


fn calculate_mask_data_size(width: usize, height: usize, mask_count: usize) -> usize {
    let rounded_width: usize = (width + 7) / 8 * 8;                 // round to multiple of 8
    let data_bits: usize = rounded_width * height * mask_count;
    let data_bytes: usize = ((data_bits + 31) / 32 * 32) / 8;       // round to multiple of 4 bytes
    data_bytes
}


fn read_texture_list(chunk: &mut GMChunk, gm_textures: &GMTextures, sprite_name: &str) -> Result<Vec<GMRef<GMTexturePageItem>>, String> {
    let texture_count: usize = chunk.read_usize_count()?;
    let mut textures: Vec<GMRef<GMTexturePageItem>> = Vec::with_capacity(texture_count);

    // if texture_count == 0 {
    //     log::debug!("Texture count is zero for Sprite \"{sprite_name}\"")
    // }

    for i in 0..texture_count {
        let texture_abs_pos: usize = chunk.read_usize_pos()?;
        if texture_abs_pos == 0 {
            // technically, it's "wrong" to just ignore these instead since there are null texture entries but also empty texture lists
            // log::warn!("Null Texture Page Item reference for texture #{i}/{texture_count} of Sprite \"{sprite_name}\"; skipping");
            continue
        }
        let texture: &GMRef<GMTexturePageItem> = gm_textures.abs_pos_to_ref.get(&texture_abs_pos)
            .ok_or_else(|| format!("Could not get texture with absolute position {} in map with length {} while \
            reading texture list of sprite {sprite_name}", texture_abs_pos, gm_textures.abs_pos_to_ref.len()))?;
        textures.push(texture.clone());
    }

    Ok(textures)
}

fn parse_nine_slice(chunk: &mut GMChunk, sprite_name: &str, start_position: usize) -> Result<GMSpriteNineSlice, String> {
    let left: i32 = chunk.read_i32()?;
    let top: i32 = chunk.read_i32()?;
    let right: i32 = chunk.read_i32()?;
    let bottom: i32 = chunk.read_i32()?;
    let enabled: bool = chunk.read_bool32()?;

    let mut tile_modes: Vec<GMSpriteNineSliceTileMode> = Vec::with_capacity(5);
    for _ in 0..5 {
        let tile_mode: i32 = chunk.read_i32()?;
        let tile_mode: GMSpriteNineSliceTileMode = tile_mode.try_into().map_err(|_| format!(
            "Invalid Tile Mode for Nine Slice 0x{:08X} at position {} \
            while parsing Sprite with name \"{}\" at position {} in chunk '{}'",
            tile_mode, chunk.cur_pos, sprite_name, start_position, chunk.name,
        ))?;
        tile_modes.push(tile_mode);
    }

    Ok(GMSpriteNineSlice {
        left,
        top,
        right,
        bottom,
        enabled,
        tile_modes,
    })
}


fn read_mask_data(chunk: &mut GMChunk, sprite_name: &str, mask_width: usize, mask_height: usize) -> Result<Vec<GMSpriteMaskEntry>, String> {
    let mask_count: usize = chunk.read_usize_count()?;
    let mut collision_masks: Vec<GMSpriteMaskEntry> = Vec::with_capacity(mask_count);

    let len: usize = (mask_width + 7) / 8 * mask_height;
    let mut total: usize = 0;

    for _ in 0..mask_count {
        let data: Vec<u8> = chunk.data.get(chunk.cur_pos.. chunk.cur_pos+len).ok_or_else(|| format!(
            "Trying to read Mask Data out of bounds while parsing \
            Sprite with name \"{}\" in chunk '{}' at position {}: {} > {}",
            sprite_name, chunk.name, chunk.cur_pos, chunk.cur_pos + len, chunk.data.len(),
        ))?.to_vec();
        chunk.cur_pos += len;
        collision_masks.push(GMSpriteMaskEntry { data, width: mask_width, height: mask_height });
        total += len;
    }

    // skip padding null bytes
    while total % 4 != 0 {
        let byte: u8 = chunk.read_u8()?;
        if byte != 0 {
            return Err(format!(
                "Invalid padding byte 0x{:02X} while parsing Masks for Sprite with name \"{}\" at position {} in chunk '{}'",
                byte, sprite_name, chunk.cur_pos, chunk.name,
            ))
        }
        total += 1;
    }

    let expected_size: usize = calculate_mask_data_size(mask_width, mask_height, mask_count);
    if total != expected_size {
        return Err(format!(
            "Mask data size is incorrect for Sprite with name \"{}\" at position {} in chunk '{}': Expected: {}; Actual: {}",
            sprite_name, chunk.cur_pos, chunk.name, expected_size, total,
        ))
    }

    Ok(collision_masks)
}

