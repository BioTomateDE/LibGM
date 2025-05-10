use crate::deserialize::all::GMData;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::rooms::{GMRoomBackground, GMRoomFlags, GMRoomGameObject, GMRoomTile, GMRoomTileTexture, GMRoomView};
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_room(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "PATH", abs_pos: data_builder.len() };

    let room_count: usize = gm_data.rooms.rooms_by_index.len();
    builder.write_usize(room_count);

    for i in 0..room_count {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::room(i))?;
    }

    for (i, room) in gm_data.rooms.rooms_by_index.iter().enumerate() {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::room(i))?;
        builder.write_gm_string(data_builder, &room.name)?;
        builder.write_gm_string(data_builder, &room.name)?;
        builder.write_gm_string(data_builder, &room.caption)?;
        builder.write_u64(room.width);
        builder.write_u64(room.height);
        builder.write_u64(room.speed);
        builder.write_bool32(room.persistent);
        builder.write_u64(room.background_color);
        builder.write_bool32(room.draw_background_color);
        builder.write_u64(room.creation_code_id);
        builder.write_u64(build_room_flags(&room.flags));
        build_room_backgrounds(data_builder, &mut builder, i, &room.backgrounds)?;
        build_room_views(data_builder, &mut builder, i, &room.views)?;
        build_room_objects(data_builder, &mut builder, &gm_data.general_info, i, &room.game_objects)?;
        build_room_tiles(data_builder, &mut builder, &gm_data.general_info, i, &room.tiles)?;
        builder.write_bool32(room.world);
        builder.write_u64(room.top);
        builder.write_u64(room.left);
        builder.write_u64(room.right);
        builder.write_u64(room.bottom);
        builder.write_f32(room.gravity_x);
        builder.write_f32(room.gravity_y);
        builder.write_f32(room.meters_per_pixel);
    }


    build_chunk(data_builder, builder)?;
    Ok(())
}


fn build_room_flags(flags: &GMRoomFlags) -> u32 {
    let mut raw: u32 = 0;
    if flags.enable_views {raw |= 1}
    if flags.show_color {raw |= 2}
    if flags.dont_clear_display_buffer {raw |= 4}
    if flags.is_gms2 {raw |= 131072}
    if flags.is_gms2_3 {raw |= 65536}
    raw
}


fn build_room_backgrounds(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, room_index: usize, backgrounds: &Vec<GMRoomBackground>) -> Result<(), String> {
    builder.write_usize(backgrounds.len());

    for i in 0..backgrounds.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::room_background(room_index, i))?;
    }

    for (i, background) in backgrounds.iter().enumerate() {
        data_builder.push_pointer_resolve(builder, GMPointer::room_background(room_index, i))?;
        builder.write_bool32(background.enabled);
        builder.write_bool32(background.foreground);
        if let Some(ref background) = background.background_definition {
            data_builder.push_pointer_placeholder(builder, GMPointer::background(background.index))?;
        } else {
            builder.write_i32(-1);
        }
        builder.write_i32(background.x);
        builder.write_i32(background.y);
        builder.write_i32(background.tile_x);
        builder.write_i32(background.tile_y);
        builder.write_i32(background.speed_x);
        builder.write_i32(background.speed_y);
        builder.write_bool32(background.stretch);
    }

    Ok(())
}


fn build_room_views(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, room_index: usize, views: &Vec<GMRoomView>) -> Result<(), String> {
    builder.write_usize(views.len());

    for i in 0..views.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::room_view(room_index, i))?;
    }

    for (i, view) in views.iter().enumerate() {
        data_builder.push_pointer_resolve(builder, GMPointer::room_view(room_index, i))?;
        builder.write_bool32(view.enabled);
        builder.write_i32(view.view_x);
        builder.write_i32(view.view_y);
        builder.write_i32(view.view_width);
        builder.write_i32(view.view_height);
        builder.write_i32(view.port_x);
        builder.write_i32(view.port_y);
        builder.write_i32(view.port_width);
        builder.write_i32(view.port_height);
        builder.write_u64(view.border_x);
        builder.write_u64(view.border_y);
        builder.write_i32(view.speed_x);
        builder.write_i32(view.speed_y);
        builder.write_i32(view.object_id);
    }

    Ok(())
}


fn build_room_objects(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    general_info: &GMGeneralInfo,
    room_index: usize,
    views: &Vec<GMRoomGameObject>,
) -> Result<(), String> {
    builder.write_usize(views.len());

    for i in 0..views.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::room_game_object(room_index, i))?;
    }

    for (i, game_object) in views.iter().enumerate() {
        data_builder.push_pointer_resolve(builder, GMPointer::room_game_object(room_index, i))?;
        builder.write_i32(game_object.x);
        builder.write_i32(game_object.y);
        data_builder.push_pointer_placeholder(builder, GMPointer::game_object(game_object.object_definition.index))?;
        builder.write_u64(game_object.instance_id);
        builder.write_i32(game_object.creation_code);
        builder.write_f32(game_object.scale_x);
        builder.write_f32(game_object.scale_y);

        if general_info.is_version_at_least(2, 2, 2, 302) {
            let image_speed: f32 = game_object.image_speed.ok_or_else(|| format!(
                "Image Speed not set for Room Object with Instance ID {} at position ({}; {}) in room with index {}.",
                game_object.instance_id, game_object.x, game_object.y, room_index))?;
            let image_index: usize = game_object.image_index.ok_or_else(|| format!(
                "Image Index not set for Room Object with Instance ID {} at position ({}; {}) in room with index {}.",
                game_object.instance_id, game_object.x, game_object.y, room_index))?;
            builder.write_f32(image_speed);
            builder.write_usize(image_index);
        }

        builder.write_u64(game_object.color);
        builder.write_f32(game_object.rotation);

        if general_info.bytecode_version >= 16 {
            let pre_create_code: i32 = game_object.pre_create_code.ok_or_else(|| format!(
                "Pre Create Code not set for Room Object with Instance ID {} at position ({}; {}) in room with index {}.",
                game_object.instance_id, game_object.x, game_object.y, room_index))?;
            builder.write_i32(pre_create_code);         // should be code reference {~~}
        }
    }

    Ok(())
}


fn build_room_tiles(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    general_info: &GMGeneralInfo,
    room_index: usize,
    tiles: &Vec<GMRoomTile>,
) -> Result<(), String> {
    builder.write_usize(tiles.len());

    for i in 0..tiles.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::room_tile(room_index, i))?;
    }

    for (i, tile) in tiles.iter().enumerate() {
        data_builder.push_pointer_resolve(builder, GMPointer::room_tile(room_index, i))?;
        builder.write_i32(tile.x);
        builder.write_i32(tile.y);
        if general_info.is_version_at_least(2, 0, 0, 0) {
            if let GMRoomTileTexture::Sprite(ref sprite) = tile.texture {
                data_builder.push_pointer_placeholder(builder, GMPointer::sprite(sprite.index))?;
            } else {
                return Err(format!(
                    "Invalid Room Tile Texture Mode (expected Sprite, got {:?}) for tile with Instance ID {} in room with index {}.",
                    tile.texture, tile.instance_id, room_index,
                ))
            };
        } else {
            if let GMRoomTileTexture::Background(ref background) = tile.texture {
                data_builder.push_pointer_placeholder(builder, GMPointer::background(background.index))?;
            } else {
                return Err(format!(
                    "Invalid Room Tile Texture Mode (expected Background, got {:?}) for tile with Instance ID {} in room with index {}.",
                    tile.texture, tile.instance_id, room_index,
                ))
            };
        }
        builder.write_u64(tile.source_x);
        builder.write_u64(tile.source_y);
        builder.write_u64(tile.width);
        builder.write_u64(tile.height);
        builder.write_i32(tile.tile_depth);
        builder.write_u64(tile.instance_id);
        builder.write_f32(tile.scale_x);
        builder.write_f32(tile.scale_y);
        builder.write_u64(tile.color);
    }

    Ok(())
}

