use crate::deserialize::all::GMData;
use crate::deserialize::backgrounds::GMBackgroundGMS2Data;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_bgnd(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("BGND")?;
    let len: usize = gm_data.backgrounds.backgrounds_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(GMPointer::Background(i))?;
    }

    for (i, background) in gm_data.backgrounds.backgrounds_by_index.iter().enumerate() {
        builder.align(8);
        builder.resolve_pointer(GMPointer::Background(i))?;

        builder.write_gm_string(&background.name)?;
        builder.write_bool32(background.transparent);
        builder.write_bool32(background.smooth);
        builder.write_bool32(background.preload);
        if let Some(ref texture) = background.texture {
            builder.write_placeholder(GMPointer::TexturePageItem(texture.index))?;
        } else {
            builder.write_usize(0);
        }

        if gm_data.general_info.is_version_at_least(2, 0, 0, 0) {
            let gms2_data: &GMBackgroundGMS2Data = background.gms2_data.as_ref()
                .ok_or_else(|| format!("GMS2 data not set for Background \"{}\"", background.name.display(&gm_data.strings)))?;

            if gms2_data.tile_ids.len() % gms2_data.items_per_tile_count != 0 {
                return Err(format!(
                    "Too many or too few Tile items for Background \"{}\": {} Tile IDs/items, which would have leftovers with {} items per tile",
                    background.name.display(&gm_data.strings), gms2_data.tile_ids.len(), gms2_data.items_per_tile_count,
                ))
            }
            let tile_count: usize = gms2_data.tile_ids.len() / gms2_data.items_per_tile_count;

            builder.write_u32(gms2_data.unknown_always2);
            builder.write_u32(gms2_data.tile_width);
            builder.write_u32(gms2_data.tile_height);
            builder.write_u32(gms2_data.output_border_x);
            builder.write_u32(gms2_data.output_border_y);
            builder.write_u32(gms2_data.tile_columns);
            builder.write_usize(gms2_data.items_per_tile_count);
            builder.write_usize(tile_count);
            builder.write_u32(gms2_data.unknown_always_zero);
            builder.write_i64(gms2_data.frame_length);

            for tile_id in &gms2_data.tile_ids {
                builder.write_u32(*tile_id);
            }
        }
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


