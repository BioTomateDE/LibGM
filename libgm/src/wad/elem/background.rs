// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

const ALIGNMENT: u32 = 8;

/// Backgrounds / tileset entries.
///
/// For GameMaker Studio 2, these will only ever be a tileset.
/// For GameMaker Studio 1, these are usually a background,
/// but are sometimes repurposed as use for a tileset as well.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Backgrounds {
    pub elems: Vec<Option<Background>>,
    /// Semi-internal flag that tracks whether to
    /// align the pointer list to 8 when serializing.
    pub align: bool,
    pub exists: bool,
}

gm_named_list_chunk!(BGND, Backgrounds, Background, nullable);

impl GMElement for Backgrounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let align = pointers.iter().all(|&p| p % ALIGNMENT == 0);
        let mut backgrounds: Vec<Option<Background>> = vec![None; pointers.len()];

        for (idx, pointer) in pointers.into_iter().enumerate() {
            if pointer == 0 {
                continue;
            }
            if align {
                reader.align(ALIGNMENT)?;
            }

            reader.assert_pos(pointer, "Background Pointer")?;
            let background = Background::deserialize(reader)?;
            backgrounds[idx] = Some(background);
        }

        Ok(Self { elems: backgrounds, align, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let count: usize = self.elems.len();
        let ctx = || format!("building nullable pointer list of {count} Backgrounds/Tilesets");

        builder.write_usize(count).ctx(ctx)?;
        let pointer_list_pos: u32 = builder.pos();
        for _ in 0..count {
            builder.write_u32(0);
        }

        for (i, background_opt) in self.elems.iter().enumerate() {
            let Some(background) = background_opt else {
                continue;
            };
            if self.align {
                builder.align(ALIGNMENT);
            }
            builder
                .overwrite_pointer_with_cur_pos(pointer_list_pos, i)
                .ctx(ctx)?;
            background.serialize(builder).ctx(ctx)?;
        }
        Ok(())
    }
}

/// A background or tileset entry in a data file.
///
/// For GameMaker Studio 2, this will only ever be a tileset.
/// For GameMaker Studio 1, this is usually a background,
/// but is sometimes repurposed as use for a tileset as well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Background {
    /// The name of the background.
    pub name: GMRef<String>,
    /// Whether the background should be transparent.
    pub transparent: bool,
    /// Whether the background should get smoothed.
    pub smooth: bool,
    /// Whether to preload the background.
    pub preload: bool,
    /// The [`GMTexturePageItem`] this background uses.
    pub texture: GMRef<TexturePageItem>,
    /// Only set in GMS 2.0+.
    pub gms2_data: Option<GMS2Data>,
}

impl GMElement for Background {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let transparent = reader.read_bool32()?;
        let smooth = reader.read_bool32()?;
        let preload = reader.read_bool32()?;
        let texture: GMRef<TexturePageItem> = reader.read_gm_texture()?;
        let gms2_data: Option<GMS2Data> = reader.deserialize_if_gm_version(2)?;

        Ok(Self {
            name,
            transparent,
            smooth,
            preload,
            texture,
            gms2_data,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_bool32(self.transparent);
        builder.write_bool32(self.smooth);
        builder.write_bool32(self.preload);
        builder.write_gm_texture(self.texture)?;
        builder.write_if_ver(&self.gms2_data, "GMS2 data", 2)?;
        Ok(())
    }

    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(ALIGNMENT);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMS2Data {
    /// The width of a tile in this tileset.
    pub tile_width: u32,

    /// The height of a tile in this tileset.
    pub tile_height: u32,

    /// GM 2024.14.1+ only.
    pub tile_separation_x: u32,

    /// GM 2024.14.1+ only.
    pub tile_separation_y: u32,

    /// The amount of extra empty pixels left and right a tile in this tileset.
    pub output_border_x: u32,

    /// The amount of extra empty pixels above and below a tile in this tileset.
    pub output_border_y: u32,

    /// The amount of columns this tileset has.
    pub tile_columns: u32,

    /// The number of frames of the tileset animation.
    pub items_per_tile_count: u32,

    /// Exported sprite index, if the background's corresponding sprite was
    /// marked to still be exported. Will be either 0 or -1 (depending on GM
    /// version) when the sprite is not exported, which makes this a bit
    /// ambiguous.
    ///
    /// In newer versions (2024.13), this seems to be used? see <https://github.com/BioTomateDE/LibGM/issues/5>
    pub exported_sprite_index: u32,

    /// The time for each frame in microseconds.
    pub frame_length: i64,

    /// All tile ids of this tileset.
    pub tile_ids: Vec<u32>,
}

impl GMElement for GMS2Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let unknown_always_two = reader.read_u32()?;
        reader.assert_int(unknown_always_two, 2, "Unknown Always Two")?;

        let tile_width = reader.read_u32()?;
        let tile_height = reader.read_u32()?;

        let mut tile_separation_x = 0;
        let mut tile_separation_y = 0;
        if reader.general_info.version >= (2024, 14, 1) {
            tile_separation_x = reader.read_u32()?;
            tile_separation_y = reader.read_u32()?;
        }

        let output_border_x = reader.read_u32()?;
        let output_border_y = reader.read_u32()?;
        let tile_columns = reader.read_u32()?;

        let items_per_tile_count = reader.read_u32()?;
        if items_per_tile_count == 0 {
            bail!("Items per tile count cannot be zero");
        }

        let tile_count = reader.read_u32()?;
        let exported_sprite_index = reader.read_u32()?;
        let frame_length = reader.read_i64()?;

        let total_tile_count = tile_count
            .checked_mul(items_per_tile_count)
            .ok_or("Total Tile count multiplication overflowed")?;

        let mut tile_ids: Vec<u32> = vec_with_capacity(total_tile_count)?;
        for _ in 0..total_tile_count {
            tile_ids.push(reader.read_u32()?);
        }

        Ok(Self {
            tile_width,
            tile_height,
            tile_separation_x,
            tile_separation_y,
            output_border_x,
            output_border_y,
            tile_columns,
            items_per_tile_count,
            exported_sprite_index,
            frame_length,
            tile_ids,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(2); // UnknownAlwaysTwo
        builder.write_u32(self.tile_width);
        builder.write_u32(self.tile_height);

        if builder.version() >= (2024, 14, 1) {
            builder.write_u32(self.tile_separation_x);
            builder.write_u32(self.tile_separation_y);
        }

        builder.write_u32(self.output_border_x);
        builder.write_u32(self.output_border_y);
        builder.write_u32(self.tile_columns);

        let total_tile_count: usize = self.tile_ids.len();
        let items_per_tile = self.items_per_tile_count as usize;

        if !total_tile_count.is_multiple_of(items_per_tile) {
            bail!(
                "Background Tiles do not add up: {} total tiles, {} items per tile leaves a \
                 remainder of {}",
                total_tile_count,
                items_per_tile,
                total_tile_count % items_per_tile,
            );
        }

        let tile_count: usize = total_tile_count / items_per_tile;
        builder.write_usize(items_per_tile)?;
        builder.write_usize(tile_count)?;

        builder.write_u32(0); // UnknownAlwaysZero
        builder.write_i64(self.frame_length);
        for tile_id in &self.tile_ids {
            builder.write_u32(*tile_id);
        }
        Ok(())
    }
}
