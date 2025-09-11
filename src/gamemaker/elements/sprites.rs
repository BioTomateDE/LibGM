use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::sequence::{GMAnimSpeedType, GMSequence};
use crate::gamemaker::elements::sprites_yyswf::{GMSpriteTypeSWF, GMSpriteYYSWFStyleGroup, GMSpriteYYSWFTimeline};
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::serialize::DataBuilder;
use crate::utility::{num_enum_from, vec_with_capacity};

#[derive(Debug, Clone)]
pub struct GMSprites {
    pub sprites: Vec<GMSprite>,
    pub exists: bool,
}
impl GMChunkElement for GMSprites {
    fn stub() -> Self {
        Self { sprites: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
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
        let sep_masks: GMSpriteSepMaskType = num_enum_from(reader.read_u32()?)?;
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
            let playback_speed_type: GMAnimSpeedType = num_enum_from(reader.read_u32()?)?;
            // both of these seem to be not an offset but instead an absolute position (see UndertaleModLib/Models/UndertaleSprite.cs@507)
            let sequence_offset: i32 = if special_version >= 2 { reader.read_i32()? } else { 0 };
            let nine_slice_offset: i32 = if special_version >= 3 { reader.read_i32()? } else { 0 };
            // {~~} set gms version to at least 2.3.2 if nine slice offset

            let special_data: GMSpriteSpecialData = match &special_sprite_type {
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
                    GMSpriteSpecialData::Normal
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
                    let jpeg_len: usize = (reader.read_i32()? & (!i32::MIN)) as usize;
                    let yyswf_version: i32 = reader.read_i32()?;
                    if !matches!(yyswf_version, 7|8) {
                        return Err(format!("Expected YYSWF Version 7 or 8 but got {yyswf_version}"))
                    }
                    let jpeg_table: Vec<u8> = reader.read_bytes_dyn(jpeg_len).map_err(|e| format!("Trying to read YYSWF JPEG Table {e}"))?.to_vec();
                    reader.align(4)?;
                    let timeline = GMSpriteYYSWFTimeline::deserialize(reader)?;
                    GMSpriteSpecialData::SWF(GMSpriteTypeSWF { swf_version, yyswf_version, jpeg_table, timeline })
                },

                2 => {      // Spine
                    reader.align(4)?;
                    if reader.general_info.is_version_at_least((2023, 1)) {
                        textures = Self::read_texture_list(reader)?;
                    }

                    let spine_version: i32 = reader.read_i32()?;
                    if spine_version >= 3 {
                        let spine_cache_version: i32 = reader.read_i32()?;
                        if spine_cache_version != 1 {
                            return Err(format!("Expected Spine Cache Version 1 but got {spine_cache_version} for Special Sprite"))
                        }
                    }

                    let json_length: usize = reader.read_usize()?;
                    let atlas_length: usize = reader.read_usize()?;
                    let texture_thing = reader.read_usize()?;   // In spine version 1: size of texture data in bytes. Post v1: texture count.
                    let mut spine_textures: Vec<GMSpriteSpineTextureEntry> = Vec::new();
                    let spine_json: String;
                    let spine_atlas: String;

                    // Version 1 - only one single PNG atlas.
                    // Version 2 - can be multiple atlases.
                    // Version 3 - an atlas can be a QOI blob.
                    match spine_version {
                        1 => {
                            let page_width = reader.read_u32()?;
                            let page_height = reader.read_u32()?;
                            
                            spine_json = GMSpriteTypeSpine::read_weird_string(reader, json_length)?;
                            spine_atlas = GMSpriteTypeSpine::read_weird_string(reader, atlas_length)?;
                            let texture_blob: Vec<u8> = reader.read_bytes_dyn(texture_thing)?.to_vec();
                            
                            spine_textures.push(GMSpriteSpineTextureEntry {
                                page_width,
                                page_height,
                                data: GMSpriteSpineTextureEntryData::Pre2023_1(texture_blob),
                            })
                        }
                        2 | 3 => {
                            spine_json = GMSpriteTypeSpine::read_weird_string(reader, json_length)?;
                            spine_atlas = GMSpriteTypeSpine::read_weird_string(reader, atlas_length)?;
                            
                            spine_textures.reserve(texture_thing);
                            for _ in 0..texture_thing {
                                spine_textures.push(GMSpriteSpineTextureEntry::deserialize(reader)?);
                            }
                        }
                        _ => return Err(format!("Expected Spine Version 1, 2 or 3 but got {spine_version} for Special Sprite"))
                    }
                    
                    GMSpriteSpecialData::Spine(GMSpriteTypeSpine {
                        version: spine_version,
                        textures: spine_textures,
                        json: spine_json,
                        atlas: spine_atlas,
                    })
                }
                3 => {   // Vector
                    // let vector_version: i32 = reader.read_i32()?;
                    // if vector_version != 1 {
                    //     return Err(format!("Expected Sprite Special Vector data version to be 1 but got {vector_version}"))
                    // }
                    // textures = Self::read_texture_list(reader)?;
                    // reader.align(4)?;
                    // let shape_version: i32 = reader.read_i32()?;
                    // if shape_version != 3 {
                    //     return Err(format!("Expected Sprite Special Vector shape version to be 3 but got {vector_version}"))
                    // }
                    // let vector_shape: GMSpriteShapeData<> = GMSpriteShapeData::deserialize(reader)?; 
                    // TODO: implement vector eventually
                    return Err("Vector Sprite Type not yet supported; will be implemented when UTMT stops using raw ints for this".to_string())
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

            special_fields = Some(GMSpriteSpecial { special_version, data: special_data, playback_speed, playback_speed_type, sequence, nine_slice, yyswf });
        } else {
            reader.cur_pos -= 4;  // unread the not -1
            // read into `textures`
            textures = Self::read_texture_list(reader)?;
            // read mask data
            let mut mask_width = width as usize;
            let mut mask_height = height as usize;
            if reader.general_info.is_version_at_least((2024, 6)) {
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
        builder.write_u32(match special_fields.data {
            GMSpriteSpecialData::Normal => 0,
            GMSpriteSpecialData::SWF(_) => 1,
            GMSpriteSpecialData::Spine(_) => 2,
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

        match &special_fields.data {
            GMSpriteSpecialData::Normal => {
                Self::build_texture_list(builder, &self.textures)?;
                self.build_mask_data(builder, &self.collision_masks)?;
            }
            GMSpriteSpecialData::SWF(swf) => {
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
            GMSpriteSpecialData::Spine(spine) => {
                builder.align(4);
                let json_blob: Vec<u8> = GMSpriteTypeSpine::build_weird_string(&spine.json);
                let atlas_blob: Vec<u8> = GMSpriteTypeSpine::build_weird_string(&spine.atlas);
                if builder.is_gm_version_at_least((2023, 1)) {
                    builder.write_simple_list(&spine.textures)?;
                }
                builder.write_i32(spine.version);
                if spine.version >= 3 {
                    builder.write_i32(1);   // spine cache version 1
                }
                builder.write_usize(json_blob.len())?;
                builder.write_usize(atlas_blob.len())?;
                
                match spine.version {
                    1 => {
                        let atlas: &GMSpriteSpineTextureEntry = spine.textures.first()
                            .ok_or("Spine Sprite's texture list empty in Spine Version 1")?;
                        let GMSpriteSpineTextureEntryData::Pre2023_1(ref texture_blob) = atlas.data else {
                            return Err("Expected Pre2023_1 texture data in Sprite Spine Version 1 but got Post2023_1".to_string())
                        };
                        builder.write_usize(texture_blob.len())?;
                        builder.write_u32(atlas.page_width);
                        builder.write_u32(atlas.page_height);
                        builder.write_bytes(&json_blob);
                        builder.write_bytes(&atlas_blob);
                        builder.write_bytes(texture_blob);
                    }
                    2 | 3 => {
                        builder.write_usize(spine.textures.len())?;
                        builder.write_bytes(&json_blob);
                        builder.write_bytes(&atlas_blob);
                        for texture_entry in &spine.textures {
                            builder.write_u32(texture_entry.page_width);
                            builder.write_u32(texture_entry.page_height);
                            if builder.is_gm_version_at_least((2023, 1)) {
                                if let GMSpriteSpineTextureEntryData::Post2023_1(length) = texture_entry.data {
                                    builder.write_u32(length);
                                } else {
                                    return Err("Expected Post2023_1 Sprite Spine texture data in 2023.1+".to_string())
                                }
                            } else {
                                if let GMSpriteSpineTextureEntryData::Pre2023_1(ref texture_blob) = texture_entry.data {
                                    builder.write_usize(texture_blob.len())?;
                                    builder.write_bytes(texture_blob);
                                } else {
                                    return Err("Expected Pre2023_1 Sprite Spine texture data in pre 2023.1".to_string())
                                }
                            }
                        }
                    }
                    other => return Err(format!("Invalid Sprite Spine Version {other}; should be 1, 2 or 3"))
                }
            }
        }
        
        if builder.is_gm_version_at_least((2, 0)) {
            if special_fields.special_version >= 2 && matches!(special_fields.data, GMSpriteSpecialData::Normal) {
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

    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<(), String> {
        reader.align(4)?;
        Ok(())
    }

    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.align(4);
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
            builder.write_gm_texture_opt(texture_page_item_ref_opt)?;
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
pub enum GMSpriteSpecialData {
    Normal,
    SWF(GMSpriteTypeSWF),
    Spine(GMSpriteTypeSpine),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteTypeSpine {
    pub version: i32,
    pub textures: Vec<GMSpriteSpineTextureEntry>,
    pub json: String,
    pub atlas: String,
}

impl GMSpriteTypeSpine {
    fn decode_spine_blob(blob: &mut Vec<u8>) {
        // don't ask me, ask Nikita Krapivin (or don't)
        let mut k: u32 = 42;
        for byte in blob {
            // if this panics in debug profile, replace with wrapping operations
            *byte -= k as u8;
            k *= k + 1;
        }
    }

    fn encode_spine_blob(blob: &mut Vec<u8>) {
        // don't ask me, ask Nikita Krapivin (or don't)
        let mut k: u32 = 42;
        for byte in blob {
            // if this panics in debug profile, replace with wrapping operations
            *byte += k as u8;
            k *= k + 1;
        }
    }
    
    fn read_weird_string(reader: &mut DataReader, size: usize) -> Result<String, String> {
        let mut blob: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
        Self::decode_spine_blob(&mut blob);
        let string: String = String::from_utf8(blob)
            .map_err(|e| format!("Could not get UTF-8 String while reading weird Sprite Spine data: {e}"))?;
        Ok(string)
    }
    
    fn build_weird_string(string: &String) -> Vec<u8> {
        let mut blob: Vec<u8> = string.as_bytes().to_vec();
        Self::encode_spine_blob(&mut blob);
        blob
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteSpineTextureEntry {
    pub page_width: u32,
    pub page_height: u32,
    pub data: GMSpriteSpineTextureEntryData,
}
impl GMElement for GMSpriteSpineTextureEntry {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let page_width: u32 = reader.read_u32()?;
        let page_height: u32 = reader.read_u32()?;
        let data = if reader.general_info.is_version_at_least((2023, 1)) {
            let texture_entry_length: u32 = reader.read_u32()?;
            GMSpriteSpineTextureEntryData::Post2023_1(texture_entry_length)
        } else {
            let size: usize = reader.read_usize()?;
            let texture_blob: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
            GMSpriteSpineTextureEntryData::Pre2023_1(texture_blob)
        };
        Ok(Self { page_width, page_height, data, })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.page_width);
        builder.write_u32(self.page_height);
        if builder.is_gm_version_at_least((2023, 1)) {
            if let GMSpriteSpineTextureEntryData::Post2023_1(texture_entry_length) = self.data {
                builder.write_u32(texture_entry_length);
            } else {
                return Err("Expected Post2023_1 Spine Texture Entry data but got Pre2023_1 for some reason".to_string())
            };
        } else {
            if let GMSpriteSpineTextureEntryData::Pre2023_1(ref texture_blob) = self.data {
                builder.write_usize(texture_blob.len())?;
                builder.write_bytes(texture_blob);
            } else {
                return Err("Expected Pre2023_1 Spine Texture Entry data but got Post2023_1 for some reason".to_string())
            };
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMSpriteSpineTextureEntryData {
    /// Texture blob raw data.
    /// > implementing [`serde::Serialize`] for this probably isn't the best idea
    Pre2023_1(Vec<u8>),
    /// Texture entry count.
    Post2023_1(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteShapeData<T: GMElement> {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub style_groups: Vec<GMSpriteYYSWFStyleGroup<T>>
}
impl<T: GMElement> GMElement for GMSpriteShapeData<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let min_x: f32 = reader.read_f32()?;
        let max_x: f32 = reader.read_f32()?;
        let min_y: f32 = reader.read_f32()?;
        let max_y: f32 = reader.read_f32()?;
        let style_groups: Vec<GMSpriteYYSWFStyleGroup<T>> = reader.read_simple_list()?;
        Ok(GMSpriteShapeData { min_x, max_x, min_y, max_y, style_groups })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        builder.write_simple_list(&self.style_groups)?;
        Ok(())
    }
}


// #[derive(Debug, Clone, PartialEq)]
// pub struct GMSpriteVectorSubShapeData {
//     fill_style1: i32,
// }


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

        let mut tile_modes: [GMSpriteNineSliceTileMode; 5] = [GMSpriteNineSliceTileMode::Stretch; 5];   // ignore default value
        for tile_mode in &mut tile_modes {
            *tile_mode = num_enum_from(reader.read_i32()?)?;
        }

        Ok(GMSpriteNineSlice { left, top, right, bottom, enabled, tile_modes })
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
    pub data: GMSpriteSpecialData,
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

