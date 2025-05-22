use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;
use crate::deserialize::texture_page_items::{GMTexture, GMTextures};

#[derive(Debug, Clone, PartialEq)]
pub struct GMBackground {
    pub name: GMRef<String>,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub texture: Option<GMRef<GMTexture>>,
    pub gms2_data: Option<GMBackgroundGMS2Data>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMBackgroundGMS2Data {
    pub unknown_always2: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub output_border_x: u32,
    pub output_border_y: u32,
    pub tile_columns: u32,
    pub items_per_tile_count: usize,
    pub unknown_always_zero: u32,
    pub frame_length: i64,
    pub tile_ids: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct GMBackgrounds {
    pub backgrounds_by_index: Vec<GMBackground>,    // strings by index/order in chunk BGND
}


pub fn parse_chunk_bgnd(
    chunk: &mut GMChunk,
    general_info: &GMGeneralInfo,
    strings: &GMStrings,
    textures: &GMTextures,
) -> Result<GMBackgrounds, String> {
    chunk.cur_pos = 0;
    let backgrounds_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(backgrounds_count);
    for _ in 0..backgrounds_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut backgrounds_by_index: Vec<GMBackground> = Vec::with_capacity(backgrounds_count);
    for start_position in start_positions {
        chunk.cur_pos = start_position;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let transparent: bool = chunk.read_bool32()?;
        let smooth: bool = chunk.read_bool32()?;
        let preload: bool = chunk.read_bool32()?;
        let texture_abs_pos: usize = chunk.read_usize()?;
        let texture: Option<GMRef<GMTexture>> = if texture_abs_pos == 0 { None } else { 
            Some(textures.abs_pos_to_ref.get(&texture_abs_pos)
                .ok_or_else(|| format!("Could not find texture with absolute position {} for Background with name \"{}\" at position {} in chunk 'BGND'",
                                       texture_abs_pos, name.display(strings), start_position))?
                .clone())
        };

        let mut gms2_data: Option<GMBackgroundGMS2Data> = None;
        if general_info.is_version_at_least(2, 0, 0, 0) {
            let unknown_always2: u32 = chunk.read_u32()?;
            let tile_width: u32 = chunk.read_u32()?;
            let tile_height: u32 = chunk.read_u32()?;
            let output_border_x: u32 = chunk.read_u32()?;
            let output_border_y: u32 = chunk.read_u32()?;
            let tile_columns: u32 = chunk.read_u32()?;
            let items_per_tile_count: usize = chunk.read_usize()?;
            let tile_count: usize = chunk.read_usize()?;
            let unknown_always_zero: u32 = chunk.read_u32()?;
            let frame_length: i64 = chunk.read_i64()?;

            let tile_count: usize = tile_count * items_per_tile_count;
            let mut tile_ids: Vec<u32> = Vec::with_capacity(tile_count);
            for _ in 0..tile_count {
                let tile_id: u32 = chunk.read_u32()?;
                tile_ids.push(tile_id);
            }

            gms2_data = Some(GMBackgroundGMS2Data {
                unknown_always2,
                tile_width,
                tile_height,
                output_border_x,
                output_border_y,
                tile_columns,
                items_per_tile_count,
                unknown_always_zero,
                frame_length,
                tile_ids,
            })
        }

        let background: GMBackground = GMBackground {
            name,
            transparent,
            smooth,
            preload,
            texture,
            gms2_data,
        };
        backgrounds_by_index.push(background);
    }

    Ok(GMBackgrounds{ backgrounds_by_index })
}


