use crate::deserialize::chunk_reading::{vec_with_capacity, GMChunkElement, GMElement, GMReader, GMRef};
use crate::deserialize::texture_page_items::GMTexturePageItem;


#[derive(Debug, Clone)]
pub struct GMBackgrounds {
    pub backgrounds: Vec<GMBackground>,
    pub exists: bool,
}
impl GMChunkElement for GMBackgrounds {
    fn empty() -> Self {
        Self { backgrounds: vec![], exists: false }
    }
}
impl GMElement for GMBackgrounds {
    fn deserialize(reader: &mut GMReader) -> Result<Self, String> {
        let backgrounds: Vec<GMBackground> = reader.read_pointer_list()?;
        Ok(Self { backgrounds, exists: true })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMBackground {
    pub name: GMRef<String>,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub texture: Option<GMRef<GMTexturePageItem>>,
    pub gms2_data: Option<GMBackgroundGMS2Data>,
}


impl GMElement for GMBackground {
    fn deserialize(reader: &mut GMReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let transparent: bool = reader.read_bool32()?;
        let smooth: bool = reader.read_bool32()?;
        let preload: bool = reader.read_bool32()?;
        let texture: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
        
        let gms2_data: Option<GMBackgroundGMS2Data> = if reader.general_info.is_version_at_least(2, 0, 0, 0) {
            Some(GMBackgroundGMS2Data::deserialize(reader)?)
        } else { None };
        
        Ok(GMBackground {
            name,
            transparent,
            smooth,
            preload,
            texture,
            gms2_data,
        })
    }
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
impl GMElement for GMBackgroundGMS2Data {
    fn deserialize(reader: &mut GMReader) -> Result<Self, String> {
        let unknown_always2: u32 = reader.read_u32()?;
        let tile_width: u32 = reader.read_u32()?;
        let tile_height: u32 = reader.read_u32()?;
        let output_border_x: u32 = reader.read_u32()?;
        let output_border_y: u32 = reader.read_u32()?;
        let tile_columns: u32 = reader.read_u32()?;
        let items_per_tile_count: usize = reader.read_usize()?;
        let tile_count: usize = reader.read_usize()?;
        let unknown_always_zero: u32 = reader.read_u32()?;
        let frame_length: i64 = reader.read_i64()?;

        let tile_count: usize = tile_count * items_per_tile_count;
        let mut tile_ids: Vec<u32> = vec_with_capacity(tile_count)?;
        for _ in 0..tile_count {
            tile_ids.push(reader.read_u32()?);
        }

        Ok(GMBackgroundGMS2Data {
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
}

