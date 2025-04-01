use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::general_info::UTGeneralInfo;
use crate::deserialize::strings::{UTStringRef, UTStrings};
use crate::deserialize::texture_page_item::{UTTexture, UTTextures};

#[derive(Debug, Clone)]
pub struct UTBackground {
    pub name: UTStringRef,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub texture: UTTexture,
    pub gms2_unknown_always2: Option<u32>,
    pub gms2_tile_width: Option<u32>,
    pub gms2_tile_height: Option<u32>,
    pub gms2_output_border_x: Option<u32>,
    pub gms2_output_border_y: Option<u32>,
    pub gms2_tile_columns: Option<u32>,
    pub gms2_items_per_tile_count: Option<u32>,
    pub gms2_tile_count: Option<u32>,
    pub gms2_unknown_always_zero: Option<u32>,
    pub gms2_frame_length: Option<i64>,
    pub gms2_tile_ids: Vec<u32>,
}


#[allow(non_snake_case)]
pub fn parse_chunk_BGND(
    chunk: &mut UTChunk,
    general_info: &UTGeneralInfo,
    strings: &UTStrings,
    textures: &UTTextures,
) -> Result<Vec<UTBackground>, String> {
    chunk.file_index = 0;
    let backgrounds_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(backgrounds_count);
    for _ in 0..backgrounds_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut backgrounds: Vec<UTBackground> = Vec::with_capacity(backgrounds_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
        let name: UTStringRef = chunk.read_ut_string(strings)?;
        let transparent: bool = chunk.read_u32()? != 0;
        let smooth: bool = chunk.read_u32()? != 0;
        let preload: bool = chunk.read_u32()? != 0;
        let texture_abs_pos: usize = chunk.read_usize()?;
        let texture: UTTexture = match textures.get_texture_by_pos(texture_abs_pos) {
            Some(texture) => texture.clone(),
            None => return Err(format!(
                "Could not find texture with absolute position {} for Background with name \"{}\" at position {} in chunk 'BGND'.",
                texture_abs_pos, name.resolve(strings)?, start_position,
            )),
        };

        let mut gms2_unknown_always2: Option<u32> = None;
        let mut gms2_tile_width: Option<u32> = None;
        let mut gms2_tile_height: Option<u32> = None;
        let mut gms2_output_border_x: Option<u32> = None;
        let mut gms2_output_border_y: Option<u32> = None;
        let mut gms2_tile_columns: Option<u32> = None;
        let mut gms2_items_per_tile_count: Option<u32> = None;
        let mut gms2_tile_count: Option<u32> = None;
        let mut gms2_unknown_always_zero: Option<u32> = None;
        let mut gms2_frame_length: Option<i64> = None;
        let mut gms2_tile_ids: Vec<u32> = vec![];     // empty --> `None`

        if general_info.is_version_at_least(2, 0, 0, 0) {
            gms2_unknown_always2 = Some(chunk.read_u32()?);
            gms2_tile_width = Some(chunk.read_u32()?);
            gms2_tile_height = Some(chunk.read_u32()?);
            gms2_output_border_x = Some(chunk.read_u32()?);
            gms2_output_border_y = Some(chunk.read_u32()?);
            gms2_tile_columns = Some(chunk.read_u32()?);
            gms2_items_per_tile_count = Some(chunk.read_u32()?);
            gms2_tile_count = Some(chunk.read_u32()?);
            gms2_unknown_always_zero = Some(chunk.read_u32()?);
            gms2_frame_length = Some(chunk.read_i64()?);

            let tile_count: usize = gms2_tile_count.unwrap() as usize * gms2_items_per_tile_count.unwrap() as usize;
            gms2_tile_ids.reserve(tile_count);
            for _ in 0..tile_count {
                let tile_id: u32 = chunk.read_u32()?;
                gms2_tile_ids.push(tile_id);
            }
        }

        let background: UTBackground = UTBackground {
            name,
            transparent,
            smooth,
            preload,
            texture,
            gms2_unknown_always2,
            gms2_tile_width,
            gms2_tile_height,
            gms2_output_border_x,
            gms2_output_border_y,
            gms2_tile_columns,
            gms2_items_per_tile_count,
            gms2_tile_count,
            gms2_unknown_always_zero,
            gms2_frame_length,
            gms2_tile_ids,
        };
        backgrounds.push(background);
    }

    Ok(backgrounds)
}


