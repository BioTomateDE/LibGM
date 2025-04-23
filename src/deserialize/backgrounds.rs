use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::{GMStringRef, GMStrings};
use crate::deserialize::texture_page_items::{GMTextureRef, GMTextures};

#[derive(Debug, Clone)]
pub struct GMBackground {
    pub name: GMStringRef,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub texture: GMTextureRef,
    pub gms2_unknown_always2: Option<u32>,
    pub gms2_tile_width: Option<u32>,
    pub gms2_tile_height: Option<u32>,
    pub gms2_outpgm_border_x: Option<u32>,
    pub gms2_outpgm_border_y: Option<u32>,
    pub gms2_tile_columns: Option<u32>,
    pub gms2_items_per_tile_count: Option<u32>,
    pub gms2_tile_count: Option<u32>,
    pub gms2_unknown_always_zero: Option<u32>,
    pub gms2_frame_length: Option<i64>,
    pub gms2_tile_ids: Vec<u32>,
}


#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct GMBackgroundRef {
    pub index: usize,
}

impl GMBackgroundRef {
    pub fn resolve<'a>(&self, backgrounds: &'a GMBackgrounds) -> Result<&'a GMBackground, String> {
        match backgrounds.backgrounds_by_index.get(self.index) {
            Some(background) => Ok(background),
            None => Err(format!(
                "Could not resolve background with index {} in list with length {}.",
                self.index, backgrounds.backgrounds_by_index.len()
            )),
        }
    }
}


#[derive(Debug, Clone)]
pub struct GMBackgrounds {
    pub backgrounds_by_index: Vec<GMBackground>,    // strings by index/order in chunk BGND
}
impl GMBackgrounds {
    pub fn get_background_by_index(&self, index: usize) -> Option<GMBackgroundRef> {
        if index >= self.backgrounds_by_index.len() {
            return None;
        }
        Some(GMBackgroundRef {
            index,
        })
    }
    pub fn len(&self) -> usize {
        self.backgrounds_by_index.len()
    }
}



pub fn parse_chunk_bgnd(
    chunk: &mut GMChunk,
    general_info: &GMGeneralInfo,
    strings: &GMStrings,
    textures: &GMTextures,
) -> Result<GMBackgrounds, String> {
    chunk.file_index = 0;
    let backgrounds_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(backgrounds_count);
    for _ in 0..backgrounds_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut backgrounds_by_index: Vec<GMBackground> = Vec::with_capacity(backgrounds_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
        let name: GMStringRef = chunk.read_gm_string(strings)?;
        let transparent: bool = chunk.read_u32()? != 0;
        let smooth: bool = chunk.read_u32()? != 0;
        let preload: bool = chunk.read_u32()? != 0;
        let texture_abs_pos: usize = chunk.read_usize()?;
        let texture: GMTextureRef = match textures.get_texture_by_pos(texture_abs_pos) {
            Some(texture) => texture,
            None => return Err(format!(
                "Could not find texture with absolute position {} for Background with name \"{}\" at position {} in chunk 'BGND'.",
                texture_abs_pos, name.resolve(strings)?, start_position,
            )),
        };

        let mut gms2_unknown_always2: Option<u32> = None;
        let mut gms2_tile_width: Option<u32> = None;
        let mut gms2_tile_height: Option<u32> = None;
        let mut gms2_outpgm_border_x: Option<u32> = None;
        let mut gms2_outpgm_border_y: Option<u32> = None;
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
            gms2_outpgm_border_x = Some(chunk.read_u32()?);
            gms2_outpgm_border_y = Some(chunk.read_u32()?);
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

        let background: GMBackground = GMBackground {
            name,
            transparent,
            smooth,
            preload,
            texture,
            gms2_unknown_always2,
            gms2_tile_width,
            gms2_tile_height,
            gms2_outpgm_border_x,
            gms2_outpgm_border_y,
            gms2_tile_columns,
            gms2_items_per_tile_count,
            gms2_tile_count,
            gms2_unknown_always_zero,
            gms2_frame_length,
            gms2_tile_ids,
        };
        backgrounds_by_index.push(background);
    }

    Ok(GMBackgrounds{ backgrounds_by_index })
}


