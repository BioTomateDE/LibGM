use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use crate::gamemaker::sequence::{GMAnimSpeedType, GMSequence};
use crate::gamemaker::sprites_yyswf::{GMSpriteTypeSWF, GMSpriteYYSWFTimeline};
use crate::gamemaker::texture_page_items::GMTexturePageItem;
use crate::gm_serialize::DataBuilder;
use crate::utility::vec_with_capacity;

#[derive(Debug, Clone)]
pub struct GMSprites {
    pub sprites: Vec<GMSprite>,
    pub exists: bool,
}
impl GMChunkElement for GMSprites {
    fn empty() -> Self {
        Self { sprites: vec![], exists: false }
    }
}
impl GMElement for GMSprites {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String>{
        let sprites: Vec<GMSprite> = reader.read_pointer_list()?;
        Ok(Self { sprites, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.sprites)?;
        Ok(())
    }
}


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
    pub textures: Vec<Option<GMRef<GMTexturePageItem>>>,
    pub collision_masks: Vec<GMSpriteMaskEntry>,
    pub special_fields: Option<GMSpriteSpecial>,
}
impl GMElement for GMSprite {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let name_str: String = reader.display_gm_str(name).to_string();
        let width: u32 = reader.read_u32()?;
        let height: u32 = reader.read_u32()?;
        let margin_left: i32 = reader.read_i32()?;
        let margin_right: i32 = reader.read_i32()?;
        let margin_bottom: i32 = reader.read_i32()?;
        let margin_top: i32 = reader.read_i32()?;
        let transparent: bool = reader.read_bool32()?;
        let smooth: bool = reader.read_bool32()?;
        let preload: bool = reader.read_bool32()?;
        let bbox_mode: i32 = reader.read_i32()?;
        let sep_masks: u32 = reader.read_u32()?;
        let sep_masks: GMSpriteSepMaskType = sep_masks.try_into().map_err(|_| format!(
            "Invalid Sep Masks Type 0x{:08X} at position {} while parsing Sprite",
            sep_masks, reader.cur_pos,
        ))?;
        let origin_x: i32 = reader.read_i32()?;
        let origin_y: i32 = reader.read_i32()?;
        let mut textures: Vec<Option<GMRef<GMTexturePageItem>>> = Vec::new();
        let mut collision_masks: Vec<GMSpriteMaskEntry> = Vec::new();
        let mut special_fields: Option<GMSpriteSpecial> = None;

        // combination of these conditions may be incorrect
        if reader.read_i32()? == -1 && reader.general_info.is_version_at_least((2, 0)) {
            let special_version: u32 = reader.read_u32()?;
            let special_sprite_type: u32 = reader.read_u32()?;

            let mut sequence: Option<GMSequence> = None;
            let mut nine_slice: Option<GMSpriteNineSlice> = None;
            let yyswf: Option<GMSpriteTypeSWF> = None;

            let playback_speed: f32 = reader.read_f32()?;
            let playback_speed_type: u32 = reader.read_u32()?;
            let playback_speed_type: GMAnimSpeedType = playback_speed_type.try_into().map_err(|_| format!(
                "Invalid Playback Anim Speed Type 0x{:08X} at position {} while parsing Sprite",
                playback_speed_type, reader.cur_pos,
            ))?;
            // both of these seem to be not an offset but instead an absolute position (see UndertaleModLib/Models/UndertaleSprite.cs@507)
            let sequence_offset: i32 = if special_version >= 2 { reader.read_i32()? } else { 0 };
            let nine_slice_offset: i32 = if special_version >= 3 { reader.read_i32()? } else { 0 };
            // {~~} set gms version to at least 2.3.2 if nine slice offset

            let sprite_type: GMSpriteType = match &special_sprite_type {
                0 => {      // Normal
                    textures = Self::read_texture_list(reader)?;
                    // read mask data
                    let mut mask_width = width as usize;
                    let mut mask_height = height as usize;
                    if reader.general_info.is_version_at_least((2024, 6, 0, 0)) {
                        mask_width = (margin_right - margin_left + 1) as usize;
                        mask_height = (margin_bottom - margin_top + 1) as usize;
                    }
                    collision_masks = read_mask_data(reader, mask_width, mask_height)?;
                    GMSpriteType::Normal(GMSpriteTypeNormal{})
                },

                1 => {      // SWF
                    // [From UndertaleModTool] "This code does not work all the time for some reason"
                    let swf_version: i32 = reader.read_i32()?;
                    // assert swf version is either 7 or 8
                    if !(swf_version == 7 || swf_version == 8) {
                        return Err(format!("Invalid SWF version {swf_version} for Sprite \"{name_str}\""))
                    }
                    if swf_version == 8 {
                        textures = Self::read_texture_list(reader)?;
                    }

                    // read YYSWF
                    reader.align(4)?;
                    let jpeg_len: i32 = reader.read_i32()? & (!0x80000000u32 as i32);    // the length is `OR`ed with int.MinValue
                    let jpeg_len: usize = jpeg_len as usize;
                    let yyswf_version: i32 = reader.read_i32()?;
                    let jpeg_table: Vec<u8> = reader.read_bytes_dyn(jpeg_len).map_err(|e| format!("Trying to read YYSWF JPEG Table {e}"))?.to_vec();
                    reader.align(4)?;
                    let timeline = GMSpriteYYSWFTimeline::deserialize(reader)?;
                    GMSpriteType::SWF(GMSpriteTypeSWF { swf_version, yyswf_version, jpeg_table, timeline })
                },

                2 => {      // Spine
                    return Err(format!("Spine format is not yet implemented for Sprite with name \"{name_str}\""))
                    // TODO {~~} IMPLEMENT TS
                }

                other => return Err(format!("Invalid Sprite Type {other} for Sprite with name \"{name_str}\"")),
            };

            if sequence_offset != 0 {
                let sequence_version: i32 = reader.read_i32()?;
                if sequence_version != 1 {
                    return Err(format!("Expected SEQN version 1 but got {sequence_version} while parsing Sequence for Sprite with name \"{name_str}\""))
                }
                sequence = Some(GMSequence::deserialize(reader)?);
            }

            if nine_slice_offset != 0 {
                nine_slice = Some(GMSpriteNineSlice::deserialize(reader)?);
            }

            special_fields = Some(GMSpriteSpecial { special_version, sprite_type, playback_speed, playback_speed_type, sequence, nine_slice, yyswf });
        } else {
            reader.cur_pos -= 4;  // unread the not -1
            // read into `textures`
            textures = Self::read_texture_list(reader)?;
            // read mask data
            let mut mask_width = width as usize;
            let mut mask_height = height as usize;
            if reader.general_info.is_version_at_least((2024, 6, 0, 0)) {
                mask_width = (margin_right - margin_left + 1) as usize;
                mask_height = (margin_bottom - margin_top + 1) as usize;
            }
            collision_masks = read_mask_data(reader, mask_width, mask_height)?;
        }

        Ok(GMSprite {
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.resolve_pointer(self)?;
        builder.write_gm_string(&self.name)?;
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        builder.write_i32(self.margin_left);
        builder.write_i32(self.margin_right);
        builder.write_i32(self.margin_top);
        builder.write_i32(self.margin_bottom);
        builder.write_bool32(self.transparent);
        builder.write_bool32(self.smooth);
        builder.write_bool32(self.preload);
        builder.write_i32(self.bbox_mode);
        builder.write_u32(self.sep_masks.into());
        builder.write_i32(self.origin_x);
        builder.write_i32(self.origin_y);
        if self.special_fields.is_none() {
            Self::build_texture_list(builder, &self.textures)?;
            self.build_mask_data(builder, &self.collision_masks)?;
            return Ok(())
        }

        let special_fields: &GMSpriteSpecial = self.special_fields.as_ref().unwrap();
        builder.write_i32(-1);
        builder.write_u32(special_fields.special_version);
        builder.write_u32(match special_fields.sprite_type {
            GMSpriteType::Normal(_) => 0,
            GMSpriteType::SWF(_) => 1,
            GMSpriteType::Spine(_) => 2,
        });

        if builder.is_gm_version_at_least((2, 0)) {
            builder.write_f32(special_fields.playback_speed);
            builder.write_u32(special_fields.playback_speed_type.into());
            if special_fields.special_version >= 2 {
                if special_fields.sequence.is_some() {
                    builder.write_pointer(&special_fields.sequence)?;
                } else {
                    builder.write_u32(0);
                }
            }
            if special_fields.special_version >= 3 {
                if special_fields.nine_slice.is_some() {
                    builder.write_pointer(&special_fields.nine_slice)?;
                } else {
                    builder.write_u32(0);
                }
            }
        }

        match &special_fields.sprite_type {
            GMSpriteType::Normal(_) => {
                Self::build_texture_list(builder, &self.textures)?;
                self.build_mask_data(builder, &self.collision_masks)?;
            }
            GMSpriteType::SWF(swf) => {
                builder.write_i32(swf.swf_version);
                if swf.swf_version == 8 {
                    Self::build_texture_list(builder, &self.textures)?;
                }
                builder.align(4);
                builder.write_usize(swf.jpeg_table.len())?;   // can be unset?
                builder.write_i32(swf.yyswf_version);
                builder.write_bytes(&swf.jpeg_table);
                builder.align(4);
                swf.timeline.serialize(builder)?;
            }
            GMSpriteType::Spine(spine) => {
                return Err(format!(  // TODO implement spine and vector
                    "Spine Sprites not yet implemented while trying to build Sprite \"{}\", please report this error",
                    builder.display_gm_str(&self.name),
                ))
            }
        }
        
        if builder.is_gm_version_at_least((2, 0)) {
            if special_fields.special_version >= 2 && matches!(special_fields.sprite_type, GMSpriteType::Normal(_)) {
                if let Some(ref sequence) = special_fields.sequence {
                    builder.resolve_pointer(&special_fields.sequence)?;
                    builder.write_u32(1);   // SEQN version
                    sequence.serialize(builder)?;
                }
            }
            if special_fields.special_version >= 3 {
                if let Some(ref nine_slice) = special_fields.nine_slice {
                    builder.resolve_pointer(&special_fields.nine_slice)?;
                    nine_slice.serialize(builder)?;
                }
            }
        }
        Ok(())
    }
}
impl GMSprite {
    fn read_texture_list(reader: &mut DataReader) -> Result<Vec<Option<GMRef<GMTexturePageItem>>>, String> {
        let count: usize = reader.read_usize()?;
        let mut textures: Vec<Option<GMRef<GMTexturePageItem>>> = Vec::with_capacity(count);
        for _ in 0..count {
            textures.push(reader.read_gm_texture_opt()?);
        }
        Ok(textures)
    }

    fn build_texture_list(builder: &mut DataBuilder, texture_list: &Vec<Option<GMRef<GMTexturePageItem>>>) -> Result<(), String> {
        builder.write_usize(texture_list.len())?;
        for texture_page_item_ref_opt in texture_list {
            if let Some(texture_page_item_ref) = texture_page_item_ref_opt {
                builder.write_gm_texture(texture_page_item_ref)?;
            } else {
                builder.write_u32(0);
            }
        }
        Ok(())
    }

    fn build_mask_data(&self, builder: &mut DataBuilder, masks: &Vec<GMSpriteMaskEntry>) -> Result<(), String> {
        builder.write_usize(masks.len())?;
        let mut total: usize = 0;

        for mask in masks {
            builder.write_bytes(&mask.data);
            total += mask.data.len();
        }

        while total % 4 != 0 {
            builder.write_u8(0);
            total += 1;
        }

        let (width, height) = if builder.is_gm_version_at_least((2024, 6)) {
            (self.margin_right as u32 - self.margin_left as u32 + 1, self.margin_bottom as u32 - self.margin_top as u32 + 1)
        } else {
            (self.width, self.height)
        };
        let rounded_width = (width + 7) / 8 * 8;   // round to multiple of 8
        let data_bits = rounded_width * height * masks.len() as u32;
        let data_bytes = (data_bits + 31) / 32 * 32 / 8;    // round to multiple of 4 bytes
        if total != data_bytes as usize {
            return Err(format!("Invalid mask data for Sprite: expected {data_bytes} bytes; got {total} bytes"))
        }
        Ok(())
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum GMSpriteType {
    Normal(GMSpriteTypeNormal),
    SWF(GMSpriteTypeSWF),
    Spine(GMSpriteTypeSpine),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteTypeNormal {}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteTypeSpine {
    /// Spine version
    pub version: i32,
    pub cache_version: i32,
    pub has_texture_data: bool,
    pub textures: Vec<GMSpriteSplineTextureEntry>,
    pub json: String,
    pub atlas: String,
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteSplineTextureEntry {
    pub page_width: i32,
    pub page_height: i32,
    /// empty for gmVersion >= 2023.1
    pub texture_blob: Vec<u8>,  // implementing Serialize for this probably isn't the best idea
    pub texture_entry_length: usize,
    pub is_qoi: bool,
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteNineSlice {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub enabled: bool,
    pub tile_modes: [GMSpriteNineSliceTileMode; 5],
}
impl GMElement for GMSpriteNineSlice {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let left: i32 = reader.read_i32()?;
        let top: i32 = reader.read_i32()?;
        let right: i32 = reader.read_i32()?;
        let bottom: i32 = reader.read_i32()?;
        let enabled: bool = reader.read_bool32()?;

        let mut tile_modes: Vec<GMSpriteNineSliceTileMode> = Vec::with_capacity(5);
        for _ in 0..5 {
            let tile_mode: i32 = reader.read_i32()?;
            let tile_mode: GMSpriteNineSliceTileMode = tile_mode.try_into().map_err(|_| format!(
                "Invalid Tile Mode for Nine Slice 0x{:08X} at position {} in chunk '{}'",
                tile_mode, reader.cur_pos, reader.chunk.name,
            ))?;
            tile_modes.push(tile_mode);
        }

        Ok(GMSpriteNineSlice {
            left,
            top,
            right,
            bottom,
            enabled,
            tile_modes: tile_modes.try_into().unwrap(),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.left);
        builder.write_i32(self.top);
        builder.write_i32(self.right);
        builder.write_i32(self.bottom);
        builder.write_bool32(self.enabled);
        for tile_mode in &self.tile_modes {
            builder.write_i32((*tile_mode).into());
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
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


fn read_mask_data(reader: &mut DataReader, mask_width: usize, mask_height: usize) -> Result<Vec<GMSpriteMaskEntry>, String> {
    let mask_count: usize = reader.read_usize()?;
    let mut collision_masks: Vec<GMSpriteMaskEntry> = vec_with_capacity(mask_count)?;

    let len: usize = (mask_width + 7) / 8 * mask_height;
    let mut total: usize = 0;

    for _ in 0..mask_count {
        let data: Vec<u8> = reader.read_bytes_dyn(len).map_err(|e| format!("Trying to read Mask Data {e}"))?.to_vec();
        collision_masks.push(GMSpriteMaskEntry { data, width: mask_width, height: mask_height });
        total += len;
    }

    // skip padding null bytes
    while total % 4 != 0 {
        let byte: u8 = reader.read_u8()?;
        if byte != 0 {
            return Err(format!("Invalid padding byte 0x{byte:02X} while parsing Masks at position {}", reader.cur_pos))
        }
        total += 1;
    }

    let expected_size: usize = calculate_mask_data_size(mask_width, mask_height, mask_count);
    if total != expected_size {
        return Err(format!("Mask data size is incorrect for Mask at position {}: Expected: {}; Actual: {}", reader.cur_pos, expected_size, total))
    }

    Ok(collision_masks)
}

fn calculate_mask_data_size(width: usize, height: usize, mask_count: usize) -> usize {
    let rounded_width: usize = (width + 7) / 8 * 8;                 // round to multiple of 8
    let data_bits: usize = rounded_width * height * mask_count;
    let data_bytes: usize = ((data_bits + 31) / 32 * 32) / 8;       // round to multiple of 4 bytes
    data_bytes
}

