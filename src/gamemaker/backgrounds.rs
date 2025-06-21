use crate::gm_deserialize::{vec_with_capacity, GMChunkElement, GMElement, DataReader, GMRef};
use crate::gamemaker::texture_page_items::GMTexturePageItem;
use crate::serialize_old::chunk_writing::DataBuilder;


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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let backgrounds: Vec<GMBackground> = reader.read_pointer_list()?;
        Ok(Self { backgrounds, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.backgrounds)?;
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let transparent: bool = reader.read_bool32()?;
        let smooth: bool = reader.read_bool32()?;
        let preload: bool = reader.read_bool32()?;
        let texture: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
        
        let gms2_data: Option<GMBackgroundGMS2Data> = if reader.general_info.is_version_at_least((2, 0, 0, 0)) {
            Some(GMBackgroundGMS2Data::deserialize(reader)?)
        } else { None };
        
        Ok(GMBackground { name, transparent, smooth, preload, texture, gms2_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_bool32(self.transparent);
        builder.write_bool32(self.smooth);
        builder.write_bool32(self.preload);
        builder.write_resource_id_opt(&self.texture);
        if builder.is_gm_version_at_least((2, 0, 0, 0)) {
            let gms2_data = self.gms2_data.as_ref().ok_or("GMS2 data not set")?;
            gms2_data.serialize(builder)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMBackgroundGMS2Data {
    pub unknown_always_two: u32,
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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let unknown_always_two: u32 = reader.read_u32()?;
        let tile_width: u32 = reader.read_u32()?;
        let tile_height: u32 = reader.read_u32()?;
        let output_border_x: u32 = reader.read_u32()?;
        let output_border_y: u32 = reader.read_u32()?;
        let tile_columns: u32 = reader.read_u32()?;
        let items_per_tile_count: usize = reader.read_usize()?;
        if items_per_tile_count == 0 {
            return Err("Items per tile count cannot be zero".to_string())
        }
        let tile_count: usize = reader.read_usize()?;
        let unknown_always_zero: u32 = reader.read_u32()?;
        let frame_length: i64 = reader.read_i64()?;

        let total_tile_count: usize = tile_count * items_per_tile_count;
        let mut tile_ids: Vec<u32> = vec_with_capacity(total_tile_count)?;
        for _ in 0..total_tile_count {
            tile_ids.push(reader.read_u32()?);
        }

        Ok(GMBackgroundGMS2Data {
            unknown_always_two,
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.unknown_always_two);
        builder.write_u32(self.tile_width);
        builder.write_u32(self.tile_height);
        builder.write_u32(self.output_border_x);
        builder.write_u32(self.output_border_y);
        builder.write_u32(self.tile_columns);

        let total_tile_count: usize = self.tile_ids.len();
        let items_per_tile: usize = self.items_per_tile_count;
        if items_per_tile == 0 {
            return Err("Items per tile is zero".to_string());
        }
        if total_tile_count % items_per_tile != 0 {
            return Err(format!(
                "Background Tiles do not add up: {} total tiles, {} items per tile leaves a remainder of {}",
                total_tile_count, items_per_tile, total_tile_count % items_per_tile,
            ));
        }
        let tile_count: usize = total_tile_count / items_per_tile;
        builder.write_usize(items_per_tile);
        builder.write_usize(tile_count);
        
        builder.write_u32(self.unknown_always_zero);
        builder.write_i64(self.frame_length);
        for tile_id in &self.tile_ids {
            builder.write_u32(*tile_id);
        }
        Ok(())
    }
}

