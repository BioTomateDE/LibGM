use crate::deserialize::all::GMData;
use crate::deserialize::backgrounds::{GMBackground, GMBackgroundGMS2Data};
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_bgnd(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "BGND", abs_pos: data_builder.len() };
    let len: usize = gm_data.backgrounds.backgrounds_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::background(i))?;
    }

    for i in 0..len {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::background(i))?;
        let background: &GMBackground = &gm_data.backgrounds.backgrounds_by_index[i];

        builder.write_gm_string(data_builder, &background.name)?;
        builder.write_bool(background.transparent);
        builder.write_bool(background.smooth);
        builder.write_bool(background.preload);
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::texture(background.texture.index))?;

        if gm_data.general_info.is_version_at_least(2, 0, 0, 0) {
            let gms2_data: &GMBackgroundGMS2Data = background.gms2_data.as_ref()
                .ok_or(format!("GMS2 data not set for Background \"{}\"", background.name.display(&gm_data.strings)))?;

            if gms2_data.tile_ids.len() % gms2_data.items_per_tile_count != 0 {
                return Err(format!(
                    "Too many or too few Tile items for Background \"{}\": {} Tile IDs/items, which would have leftovers with {} items per tile.",
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

    build_chunk(data_builder, builder)?;
    Ok(())
}


