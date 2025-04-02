use num_enum::TryFromPrimitive;
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::general_info::UTGeneralInfo;
use crate::deserialize::sequence::{parse_sequence, UTAnimSpeedType, UTSequence};
use crate::deserialize::sprites_yyswf::{parse_yyswf_timeline, UTSpriteYYSWF, UTSpriteYYSWFTimeline};
use crate::deserialize::strings::{UTStringRef, UTStrings};
use crate::deserialize::texture_page_item::{UTTextureRef, UTTextures};

#[derive(Debug, Clone)]
pub struct UTSprite {
    pub name: UTStringRef,
    pub width: usize,
    pub height: usize,
    pub margin_left: i32,
    pub margin_right: i32,
    pub margin_bottom: i32,
    pub margin_top: i32,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub bbox_mode: i32,
    pub sep_masks: UTSpriteSepMaskType,
    pub origin_x: i32,
    pub origin_y: i32,
    pub textures: Vec<UTTextureRef>,
    pub collision_masks: Vec<UTSpriteMaskEntry>,
    pub special_fields: Option<UTSpriteSpecial>,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum UTSpriteType {
    Normal,
    SWF,
    Spine,
}

#[derive(Debug, Clone)]
pub struct UTSpriteTypeSpine {
    /// Spine version
    pub version: i32,
    pub cache_version: i32,
    pub has_texture_data: bool,
    pub textures: Vec<UTSpriteSplineTextureEntry>,
    pub json: String,
    pub atlas: String,
}

#[derive(Debug, Clone)]
pub struct UTSpriteSplineTextureEntry {
    pub page_width: i32,
    pub page_height: i32,
    /// empty for gmVersion >= 2023.1
    pub texture_blob: Vec<u8>,
    pub texture_entry_length: usize,
    pub is_qoi: bool,
}

#[derive(Debug, Clone)]
pub struct UTSpriteNineSlice {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub enabled: bool,
    pub tile_modes: Vec<UTSpriteNineSliceTileMode>,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(i32)]
pub enum UTSpriteNineSliceTileMode {
    Stretch = 0,
    Repeat = 1,
    Mirror = 2,
    BlankRepeat = 3,
    Hide = 4,
}

#[derive(Debug, Clone)]
pub struct UTSpriteSpecial {
    /// Version of Special Thingy
    pub special_version: u32,
    pub sprite_type: UTSpriteType,
    /// GMS2
    pub playback_speed: Option<f32>,
    /// GMS 2
    pub playback_speed_type: Option<UTAnimSpeedType>,
    /// Special Version 2
    pub sequence: Option<UTSequence>,
    /// Special Version 3
    pub nine_slice: Option<UTSpriteNineSlice>,
    /// SWF
    pub swf_version: Option<i32>,
    pub yyswf: Option<UTSpriteYYSWF>,
}

#[derive(Debug, Clone)]
pub struct UTSpriteRef {
    index: usize,
}
impl UTSpriteRef {
    pub fn resolve<'a>(&self, sprites: &'a UTSprites) -> Result<&'a UTSprite, String> {
        match sprites.sprites_by_index.get(self.index) {
            Some(sprite) => Ok(sprite),
            None => Err(format!(
                "Could not resolve Sprite with index {} in list with length {}.",
                self.index, sprites.sprites_by_index.len()
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UTSprites {
    pub sprites_by_index: Vec<UTSprite>,
}
impl UTSprites {
    pub fn get_sprite_by_index(&self, index: usize) -> Option<UTSpriteRef> {
        if index >= self.sprites_by_index.len() {
            return None;
        }
        Some(UTSpriteRef {index})
    }
}


#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum UTSpriteSepMaskType {
    AxisAlignedRect = 0,
    Precise = 1,
    RotatedRect = 2,
}

#[derive(Debug, Clone)]
pub struct UTSpriteMaskEntry {
    data: Vec<u8>,
    width: usize,
    height: usize,
}


#[allow(non_snake_case)]
pub fn parse_chunk_SPRT(
    chunk: &mut UTChunk,
    general_info: &UTGeneralInfo,
    strings: &UTStrings,
    ut_textures: &UTTextures,
) -> Result<UTSprites, String> {
    chunk.file_index = 0;
    let sprites_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(sprites_count);
    for _ in 0..sprites_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut sprites_by_index: Vec<UTSprite> = Vec::with_capacity(sprites_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
        let name: UTStringRef = chunk.read_ut_string(strings)?;
        let width: usize = chunk.read_usize()?;
        let height: usize = chunk.read_usize()?;
        let margin_left: i32 = chunk.read_i32()?;
        let margin_right: i32 = chunk.read_i32()?;
        let margin_bottom: i32 = chunk.read_i32()?;
        let margin_top: i32 = chunk.read_i32()?;
        let transparent: bool = chunk.read_u32()? != 0;
        let smooth: bool = chunk.read_u32()? != 0;
        let preload: bool = chunk.read_u32()? != 0;
        let bbox_mode: i32 = chunk.read_i32()?;
        let sep_masks: u32 = chunk.read_u32()?;
        let sep_masks: UTSpriteSepMaskType = match sep_masks.try_into() {
            Ok(masks) => masks,
            Err(_) => return Err(format!(
                "Invalid Sep Masks Type 0x{:08X} at position {} while parsing Sprite at position {} in chunk '{}'.",
                sep_masks, chunk.file_index, start_position, chunk.name,
            )),
        };
        let origin_x: i32 = chunk.read_i32()?;
        let origin_y: i32 = chunk.read_i32()?;
        let mut textures: Vec<UTTextureRef> = Vec::new();
        let mut collision_masks: Vec<UTSpriteMaskEntry> = Vec::new();
        let mut special_fields: Option<UTSpriteSpecial> = None;

        if chunk.read_i32()? == -1 {
            let mut sequence_offset: i32 = 0;
            let mut nine_slice_offset: i32 = 0;
            let mut sequence: Option<UTSequence> = None;
            let mut nine_slice: Option<UTSpriteNineSlice> = None;

            let special_version: u32 = chunk.read_u32()?;
            let special_sprite_type: u32 = chunk.read_u32()?;
            let special_sprite_type: UTSpriteType = match special_sprite_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!(
                    "Invalid Special Sprite Type 0x{:08X} at position {} while parsing Sprite at position {} in chunk '{}'.",
                    special_sprite_type, chunk.file_index, start_position, chunk.name,
                )),
            };
            let mut playback_speed: Option<f32> = None;
            let mut playback_speed_type: Option<UTAnimSpeedType> = None;
            let mut swf_version: Option<i32> = None;
            let mut yyswf: Option<UTSpriteYYSWF> = None;

            if general_info.is_version_at_least(2, 0, 0, 0) {
                playback_speed = Some(chunk.read_f32()?);
                let playback_speed_type_: u32 = chunk.read_u32()?;
                let playback_speed_type_: UTAnimSpeedType = match playback_speed_type_.try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err(format!(
                        "Invalid Playback Anim Speed Type 0x{:08X} at position {} while parsing Sprite at position {} in chunk '{}'.",
                        playback_speed_type_, chunk.file_index, start_position, chunk.name,
                    )),
                };
                playback_speed_type = Some(playback_speed_type_);
                if special_version >= 2 {
                    sequence_offset = chunk.read_i32()?;
                }
                if special_version >= 3 {
                    // {~~} set gms version to at least 2.3.2
                    nine_slice_offset = chunk.read_i32()?;
                }

                match &special_sprite_type {
                    UTSpriteType::Normal => {
                        // read texture list to `textures`
                        read_texture_list(chunk, &mut textures, ut_textures, name.resolve(strings)?, start_position)?;
                        // read mask data
                        let mut mask_width: usize = width;
                        let mut mask_height: usize = height;
                        if general_info.is_version_at_least(2024, 6, 0, 0) {
                            mask_width = (margin_right - margin_left + 1) as usize;
                            mask_height = (margin_bottom - margin_top + 1) as usize;
                        }
                        collision_masks = read_mask_data(chunk, name.resolve(strings)?, mask_width, mask_height)?;
                    },

                    UTSpriteType::SWF => {
                        // [From UndertaleModTool] "This code does not work all the time for some reason."
                        swf_version = Some(chunk.read_i32()?);
                        // {~~} assert the version is 7 or 8
                        if swf_version.unwrap() == 8 {
                            read_texture_list(chunk, &mut textures, ut_textures, name.resolve(strings)?, start_position)?;
                        }

                        // read YYSWF
                        align_reader(chunk, 4, 0x00)?;
                        let jpeg_len: i32 = chunk.read_i32()? & (!0x80000000u32 as i32);    // the length is ORed with int.MinValue
                        let jpeg_len: usize = jpeg_len as usize;
                        let yyswf_version: i32 = chunk.read_i32()?;
                        let jpeg_table: Vec<u8> = match chunk.data.get(chunk.file_index .. chunk.file_index+jpeg_len) {
                            Some(bytes) => bytes.to_vec(),
                            None => return Err(format!(
                                "Trying to read YYSWF JPEG Table out of bounds while parsing \
                                Sprite with name \"{}\" in chunk '{}' at position {}: {} > {}.",
                                name.resolve(strings)?, chunk.name, chunk.file_index, chunk.file_index + jpeg_len, chunk.data.len(),
                            )),
                        };
                        chunk.file_index += jpeg_len;
                        align_reader(chunk, 4, 0x00)?;
                        let timeline: UTSpriteYYSWFTimeline = parse_yyswf_timeline(chunk, general_info)?;

                        yyswf = Some(UTSpriteYYSWF {
                            version: yyswf_version,
                            jpeg_table,
                            timeline,
                        })
                    },

                    UTSpriteType::Spine => {
                        return Err(format!(
                            "Spine format is not yet implemented for Sprite with name \"{}\" and absolute position {}!",
                            name.resolve(strings)?, start_position + chunk.abs_pos,
                        ))
                        // TODO {~~} IMPLEMENT TS
                    }
                }

                if sequence_offset != 0 {
                    let thingy: i32 = chunk.read_i32()?;
                    if thingy != 1 {
                        return Err(format!(
                            "Expected 1 but got {} while parsing Sequence for Sprite with name \"{}\" in chunk '{}'.",
                            thingy, name.resolve(strings)?, chunk.name,
                        ))
                    }
                    sequence = Some(parse_sequence(chunk, strings)?);
                }

                if nine_slice_offset != 0 {
                    nine_slice = Some(parse_nine_slice(chunk, name.resolve(strings)?, start_position)?);
                }
            }

            special_fields = Some(UTSpriteSpecial {
                special_version,
                sprite_type: special_sprite_type,
                playback_speed,
                playback_speed_type,
                sequence,
                nine_slice,
                swf_version,
                yyswf,
            });
        } else {
            chunk.file_index -= 4;  // unread the not -1
            // read into `textures`
            read_texture_list(chunk, &mut textures, ut_textures, name.resolve(strings)?, start_position)?;
            // read mask data
            let mut mask_width: usize = width;
            let mut mask_height: usize = height;
            if general_info.is_version_at_least(2024, 6, 0, 0) {
                mask_width = (margin_right - margin_left + 1) as usize;
                mask_height = (margin_bottom - margin_top + 1) as usize;
            }
            collision_masks = read_mask_data(chunk, name.resolve(strings)?, mask_width, mask_height)?;
        }

        sprites_by_index.push(UTSprite {
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


    Ok(UTSprites {sprites_by_index})
}


fn calculate_mask_data_size(width: usize, height: usize, mask_count: usize) -> usize {
    let rounded_width: usize = (width + 7) / 8 * 8;                 // round to multiple of 8
    let data_bits: usize = rounded_width * height * mask_count;
    let data_bytes: usize = ((data_bits + 31) / 32 * 32) / 8;       // round to multiple of 4 bytes
    data_bytes
}


fn read_texture_list(chunk: &mut UTChunk, textures: &mut Vec<UTTextureRef>, ut_textures: &UTTextures, sprite_name: &str, start_position: usize) -> Result<(), String> {
    let texture_count: usize = chunk.read_usize()?;
    textures.reserve(texture_count);
    for _ in 0..texture_count {
        let texture_abs_pos: usize = chunk.read_usize()?;
        let texture: UTTextureRef = match ut_textures.get_texture_by_pos(texture_abs_pos) {
            Some(texture) => texture,
            None => return Err(format!(
                "Could not find texture with absolute position {} for Sprite with name \"{}\" at position {} in chunk '{}'.",
                texture_abs_pos, sprite_name, start_position, chunk.name,
            )),
        };
        textures.push(texture);
    }
    Ok(())
}

fn parse_nine_slice(chunk: &mut UTChunk, sprite_name: &str, start_position: usize) -> Result<UTSpriteNineSlice, String> {
    let left: i32 = chunk.read_i32()?;
    let top: i32 = chunk.read_i32()?;
    let right: i32 = chunk.read_i32()?;
    let bottom: i32 = chunk.read_i32()?;
    let enabled: bool = chunk.read_i32()? != 0;

    let mut tile_modes: Vec<UTSpriteNineSliceTileMode> = Vec::with_capacity(5);
    for _ in 0..5 {
        let tile_mode: i32 = chunk.read_i32()?;
        let tile_mode: UTSpriteNineSliceTileMode = match tile_mode.try_into() {
            Ok(ok) => ok,
            Err(_) => return Err(format!(
                "Invalid Tile Mode for Nine Slice 0x{:08X} at position {} \
                while parsing Sprite with name \"{}\" at position {} in chunk '{}'.",
                tile_mode, chunk.file_index, sprite_name, start_position, chunk.name,
            )),
        };
        tile_modes.push(tile_mode);
    }

    Ok(UTSpriteNineSlice {
        left,
        top,
        right,
        bottom,
        enabled,
        tile_modes,
    })
}


fn read_mask_data(chunk: &mut UTChunk, sprite_name: &str, mask_width: usize, mask_height: usize) -> Result<Vec<UTSpriteMaskEntry>, String> {
    let mask_count: usize = chunk.read_usize()?;
    let mut collision_masks: Vec<UTSpriteMaskEntry> = Vec::with_capacity(mask_count);

    let len: usize = (mask_width + 7) / 8 * mask_height;
    let mut total: usize = 0;

    for _ in 0..mask_count {
        let data: Vec<u8> = match chunk.data.get(chunk.file_index .. chunk.file_index+len) {
            Some(bytes) => bytes.to_vec(),
            None => return Err(format!(
                "Trying to read Mask Data out of bounds while parsing \
                Sprite with name \"{}\" in chunk '{}' at position {}: {} > {}.",
                sprite_name, chunk.name, chunk.file_index, chunk.file_index + len, chunk.data.len(),
            )),
        };
        chunk.file_index += len;
        collision_masks.push(UTSpriteMaskEntry { data, width: mask_width, height: mask_height });
        total += len;
    }

    // skip padding null bytes
    while total % 4 != 0 {
        let byte: u8 = chunk.read_u8()?;
        if byte != 0 {
            return Err(format!(
                "Invalid padding byte 0x{:02X} while parsing Masks for Sprite with name \"{}\" at position {} in chunk '{}'.",
                byte, sprite_name, chunk.file_index, chunk.name,
            ))
        }
        total += 1;
    }

    let expected_size: usize = calculate_mask_data_size(mask_width, mask_height, mask_count);
    if total != expected_size {
        return Err(format!(
            "Mask data size is incorrect for Sprite with name \"{}\" at position {} in chunk '{}': Expected: {}; Actual: {}.",
            sprite_name, chunk.file_index, chunk.name, expected_size, total,
        ))
    }

    Ok(collision_masks)
}


/// no idea what this actually does
pub fn align_reader(chunk: &mut UTChunk, alignment: usize, padding_byte: u8) -> Result<(), String> {
    // maybe `alignment` needs to be i32 like in UndertaleModTool
    while ((chunk.file_index + chunk.abs_pos) & (alignment - 1)) as u8 != padding_byte {
        let byte: u8 = chunk.read_u8()?;
        if byte != padding_byte {
            return Err(format!(
                "Invalid alignment padding 0x{:02X} (expected: 0x{}) at position {} in chunk '{}' with alignment value {}.",
                byte, padding_byte, chunk.file_index - 1, chunk.name, alignment,
            ));
        }
    }
    Ok(())
}

