use num_enum::TryFromPrimitive;
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::general_info::UTGeneralInfo;
use crate::deserialize::sequence::{UTAnimSpeedType, UTSequence};
use crate::deserialize::sprites_yyswf::UTSpriteYYSWF;
use crate::deserialize::strings::{UTStringRef, UTStrings};
use crate::deserialize::texture_page_item::{UTTextureRef, UTTextures};

#[derive(Debug, Clone)]
pub struct UTSprite {
    pub name: UTStringRef,
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
    pub sep_masks: u32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub textures: Vec<UTTextureRef>,
    pub special_fields: Option<UTSpriteSpecial>,
}

#[derive(Debug, Clone)]
pub enum UTSpriteType {
    Normal,
    SWF(UTSpriteTypeSWF),
    Spine,
}

#[derive(Debug, Clone)]
pub struct UTSpriteTypeSWF {
    //// SWF version
    pub version: i32,
    pub yyswf: UTSpriteYYSWF,
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

#[derive(Debug, Clone)]
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
    pub version: u32,
    pub sprite_type: UTSpriteType,
    /// GMS2
    pub playback_speed: Option<f32>,
    /// GMS2
    pub playback_speed_type: Option<UTAnimSpeedType>,
    /// Special Version 2
    pub sequence: UTSequence,
    /// Special Version 3
    pub nine_slice: UTSpriteNineSlice,
}

#[derive(Debug, Clone)]
pub struct UTSpriteRef {
    index: usize,
}
impl UTSpriteRef {
    pub fn resolve<'a>(&self, sprites: &'a UTSprites) -> Result<&'a UTSprite, String> {
        match sprites.sprites_by_index.get(self.index) {
            Some(Sprite) => Ok(Sprite),
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


#[allow(non_snake_case)]
pub fn parse_chunk_SPRT(
    chunk: &mut UTChunk,
    general_info: &UTGeneralInfo,
    strings: &UTStrings,
    textures: &UTTextures,
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
        let width: u32 = chunk.read_u32()?;
        let height: u32 = chunk.read_u32()?;
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
        if chunk.read_i32()? == -1 {

        }
    }


    Ok(UTSprites {sprites_by_index})
}
