use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::serialize::DataBuilder;
use crate::utility::vec_with_capacity;

const ALIGNMENT: usize = 8;

#[derive(Debug, Clone)]
pub struct GMBackgrounds {
    pub backgrounds: Vec<GMBackground>,
    pub is_aligned: bool,
    pub exists: bool,
}
impl GMChunkElement for GMBackgrounds {
    fn empty() -> Self {
        Self { backgrounds: vec![], is_aligned: true, exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMBackgrounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let mut is_aligned: bool = true;
        let backgrounds: Vec<GMBackground> = reader.read_aligned_list_chunk(ALIGNMENT, &mut is_aligned)?;
        Ok(Self { backgrounds, is_aligned, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        if self.is_aligned {
            builder.write_aligned_list_chunk(&self.backgrounds, ALIGNMENT)?;
        } else {
            builder.write_pointer_list(&self.backgrounds)?;
        }
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
        let gms2_data: Option<GMBackgroundGMS2Data> = reader.deserialize_if_gm_version((2, 0))?;
        
        Ok(GMBackground { name, transparent, smooth, preload, texture, gms2_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_bool32(self.transparent);
        builder.write_bool32(self.smooth);
        builder.write_bool32(self.preload);
        builder.write_gm_texture_opt(&self.texture)?;
        self.gms2_data.serialize_if_gm_ver(builder, "GMS2 data", (2, 0))?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMBackgroundGMS2Data {
    /// The width of a tile in this tileset.
    pub tile_width: u32,
    /// The height of a tile in this tileset.
    pub tile_height: u32,
    /// The amount of extra empty pixels left and right a tile in this tileset.
    pub output_border_x: u32,
    /// The amount of extra empty pixels above and below a tile in this tileset.
    pub output_border_y: u32,
    /// The amount of columns this tileset has.
    pub tile_columns: u32,
    /// The number of frames of the tileset animation.
    pub items_per_tile_count: usize,
    /// The time for each frame in microseconds.
    pub frame_length: i64,
    /// All tile ids of this tileset.
    pub tile_ids: Vec<u32>,
}
impl GMElement for GMBackgroundGMS2Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let unknown_always_two: u32 = reader.read_u32()?;
        if unknown_always_two != 2 {
            return Err(format!("Expected UnknownAlwaysTwo but got {unknown_always_two} in Background GMS2 data"))
        }
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
        if unknown_always_zero != 0 {
            return Err(format!("Expected UnknownAlwaysZero but got {unknown_always_zero} in Background GMS2 data"))
        }
        let frame_length: i64 = reader.read_i64()?;

        let total_tile_count: usize = tile_count * items_per_tile_count;
        let mut tile_ids: Vec<u32> = vec_with_capacity(total_tile_count)?;
        for _ in 0..total_tile_count {
            tile_ids.push(reader.read_u32()?);
        }

        Ok(GMBackgroundGMS2Data {
            tile_width,
            tile_height,
            output_border_x,
            output_border_y,
            tile_columns,
            items_per_tile_count,
            frame_length,
            tile_ids,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(2);       // UnknownAlwaysTwo
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
        builder.write_usize(items_per_tile)?;
        builder.write_usize(tile_count)?;
        
        builder.write_u32(0);   // UnknownAlwaysZero
        builder.write_i64(self.frame_length);
        for tile_id in &self.tile_ids {
            builder.write_u32(*tile_id);
        }
        Ok(())
    }
}

