use crate::deserialize::all::GMData;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::rooms::{GMRoom, GMRoomBackground, GMRoomFlags, GMRoomGameObject, GMRoomLayer, GMRoomTile, GMRoomTileTexture, GMRoomView};
use crate::deserialize::sequence::GMSequence;
use crate::deserialize::strings::GMStrings;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};
use crate::serialize::sequence::build_sequence;

pub fn build_chunk_room(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "ROOM");

    let room_count: usize = gm_data.rooms.rooms_by_index.len();
    builder.write_usize(room_count);

    for i in 0..room_count {
        data_builder.write_pointer_placeholder(&mut builder, GMPointer::Room(i))?;
    }

    for (i, room) in gm_data.rooms.rooms_by_index.iter().enumerate() {
        data_builder.resolve_pointer(&mut builder, GMPointer::Room(i))?;
        build_room(data_builder, &mut builder, &gm_data.general_info, &gm_data.strings, i, room)
            .map_err(|e| format!("{e} for Room #{i} with name \"{}\" while building Rooms", room.name.display(&gm_data.strings)))?;
    }

    builder.finish(data_builder)?;
    Ok(())
}


fn build_room(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, strings: &GMStrings, room_index: usize, room: &GMRoom) -> Result<(), String> {
    builder.write_gm_string(data_builder, &room.name)?;
    builder.write_gm_string(data_builder, &room.caption)?;
    builder.write_u32(room.width);
    builder.write_u32(room.height);
    builder.write_u32(room.speed);
    builder.write_bool32(room.persistent);
    builder.write_u32(room.background_color ^ 0xFF000000);    // remove alpha (background color doesn't have alpha)
    builder.write_bool32(room.draw_background_color);
    if let Some(ref creation_code) = room.creation_code {
        data_builder.write_pointer_placeholder(builder, GMPointer::Code(creation_code.index))?;
    } else {
        builder.write_i32(-1);
    }
    builder.write_u32(build_room_flags(&room.flags));
    build_room_backgrounds(data_builder, builder, room_index, &room.backgrounds)?;
    build_room_views(data_builder, builder, room_index, &room.views)?;
    build_room_objects(data_builder, builder, &general_info, room_index, &room.game_objects)?;
    build_room_tiles(data_builder, builder, &general_info, room_index, &room.tiles)?;
    builder.write_bool32(room.world);
    builder.write_u32(room.top);
    builder.write_u32(room.left);
    builder.write_u32(room.right);
    builder.write_u32(room.bottom);
    builder.write_f32(room.gravity_x);
    builder.write_f32(room.gravity_y);
    builder.write_f32(room.meters_per_pixel);
    if general_info.is_version_at_least(2, 0, 0, 0) {
        build_room_layers(data_builder, builder, room_index, room.layers.as_ref().ok_or("Layers not set")?)?;
        if general_info.is_version_at_least(2, 3, 0, 0) {
            build_room_sequences(data_builder, builder, general_info, strings, room.sequences.as_ref().ok_or("Sequences not set")?)?;
        }
    }
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
    data_builder.write_pointer_placeholder(builder, GMPointer::RoomBackgroundPointerList(room_index))?;
    data_builder.resolve_pointer(builder, GMPointer::RoomBackgroundPointerList(room_index))?;
    builder.write_usize(backgrounds.len());

    for i in 0..backgrounds.len() {
        data_builder.write_pointer_placeholder(builder, GMPointer::RoomBackground(room_index, i))?;
    }

    for (i, background) in backgrounds.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::RoomBackground(room_index, i))?;
        builder.write_bool32(background.enabled);
        builder.write_bool32(background.foreground);
        if let Some(ref background) = background.background_definition {
            data_builder.write_pointer_placeholder(builder, GMPointer::Background(background.index))?;
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
    data_builder.write_pointer_placeholder(builder, GMPointer::RoomViewPointerList(room_index))?;
    data_builder.resolve_pointer(builder, GMPointer::RoomViewPointerList(room_index))?;
    builder.write_usize(views.len());

    for i in 0..views.len() {
        data_builder.write_pointer_placeholder(builder, GMPointer::RoomView(room_index, i))?;
    }

    for (i, view) in views.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::RoomView(room_index, i))?;
        builder.write_bool32(view.enabled);
        builder.write_i32(view.view_x);
        builder.write_i32(view.view_y);
        builder.write_i32(view.view_width);
        builder.write_i32(view.view_height);
        builder.write_i32(view.port_x);
        builder.write_i32(view.port_y);
        builder.write_i32(view.port_width);
        builder.write_i32(view.port_height);
        builder.write_u32(view.border_x);
        builder.write_u32(view.border_y);
        builder.write_i32(view.speed_x);
        builder.write_i32(view.speed_y);
        if let Some(ref obj) = view.object {
            data_builder.write_pointer_placeholder(builder, GMPointer::GameObject(obj.index))?;
        } else {
            builder.write_i32(-1);
        }
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
    data_builder.write_pointer_placeholder(builder, GMPointer::RoomGameObjectPointerList(room_index))?;
    data_builder.resolve_pointer(builder, GMPointer::RoomGameObjectPointerList(room_index))?;
    builder.write_usize(views.len());

    for i in 0..views.len() {
        data_builder.write_pointer_placeholder(builder, GMPointer::RoomGameObject(room_index, i))?;
    }

    for (i, game_object) in views.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::RoomGameObject(room_index, i))?;
        builder.write_i32(game_object.x);
        builder.write_i32(game_object.y);
        data_builder.write_pointer_placeholder(builder, GMPointer::GameObject(game_object.object_definition.index))?;
        builder.write_u32(game_object.instance_id);
        if let Some(ref creation_code) = game_object.creation_code {
            data_builder.write_pointer_placeholder(builder, GMPointer::Code(creation_code.index))?;
        } else {
            builder.write_i32(-1);
        }
        builder.write_f32(game_object.scale_x);
        builder.write_f32(game_object.scale_y);

        if general_info.is_version_at_least(2, 2, 2, 302) {
            let image_speed: f32 = game_object.image_speed.ok_or_else(|| format!(
                "Image Speed not set for Room Object with Instance ID {} at position ({}; {}) in room with index {}",
                game_object.instance_id, game_object.x, game_object.y, room_index))?;
            let image_index: usize = game_object.image_index.ok_or_else(|| format!(
                "Image Index not set for Room Object with Instance ID {} at position ({}; {}) in room with index {}",
                game_object.instance_id, game_object.x, game_object.y, room_index))?;
            builder.write_f32(image_speed);
            builder.write_usize(image_index);
        }

        builder.write_u32(game_object.color);
        builder.write_f32(game_object.rotation);

        if general_info.bytecode_version >= 16 {
            if let Some(ref pre_creation_code) = game_object.pre_create_code {
                data_builder.write_pointer_placeholder(builder, GMPointer::Code(pre_creation_code.index))?;
            } else {
                builder.write_i32(-1);
            }
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
    data_builder.write_pointer_placeholder(builder, GMPointer::RoomTilePointerList(room_index))?;
    data_builder.resolve_pointer(builder, GMPointer::RoomTilePointerList(room_index))?;
    builder.write_usize(tiles.len());

    for i in 0..tiles.len() {
        data_builder.write_pointer_placeholder(builder, GMPointer::RoomTile(room_index, i))?;
    }

    for (i, tile) in tiles.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::RoomTile(room_index, i))?;
        builder.write_i32(tile.x);
        builder.write_i32(tile.y);
        if general_info.is_version_at_least(2, 0, 0, 0) {
            if let GMRoomTileTexture::Sprite(ref sprite) = tile.texture {
                data_builder.write_pointer_placeholder(builder, GMPointer::Sprite(sprite.index))?;
            } else {
                return Err(format!(
                    "Invalid Room Tile Texture Mode (expected Sprite, got {:?}) for tile with Instance ID {} in room with index {}",
                    tile.texture, tile.instance_id, room_index,
                ))
            };
        } else {
            if let GMRoomTileTexture::Background(ref background) = tile.texture {
                data_builder.write_pointer_placeholder(builder, GMPointer::Background(background.index))?;
            } else {
                return Err(format!(
                    "Invalid Room Tile Texture Mode (expected Background, got {:?}) for tile with Instance ID {} in room with index {}",
                    tile.texture, tile.instance_id, room_index,
                ))
            };
        }
        builder.write_u32(tile.source_x);
        builder.write_u32(tile.source_y);
        builder.write_u32(tile.width);
        builder.write_u32(tile.height);
        builder.write_i32(tile.tile_depth);
        builder.write_u32(tile.instance_id);
        builder.write_f32(tile.scale_x);
        builder.write_f32(tile.scale_y);
        builder.write_u32(tile.color);
    }

    Ok(())
}


fn build_room_layers(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, room_index: usize, layers: &Vec<GMRoomLayer>) -> Result<(), String> {
    data_builder.write_pointer_placeholder(builder, GMPointer::RoomLayerPointerList(room_index))?;
    data_builder.resolve_pointer(builder, GMPointer::RoomLayerPointerList(room_index))?;
    builder.write_usize(layers.len());

    for i in 0..layers.len() {
        data_builder.write_pointer_placeholder(builder, GMPointer::RoomLayer(room_index, i))?;
    }

    for (i, layer) in layers.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::RoomLayer(room_index, i))?;
        builder.write_gm_string(data_builder, &layer.layer_name)?;
        builder.write_u32(layer.layer_id);
        builder.write_u32(layer.layer_type.into());
        builder.write_i32(layer.layer_depth);
        builder.write_f32(layer.x_offset);
        builder.write_f32(layer.y_offset);
        builder.write_f32(layer.horizontal_speed);
        builder.write_f32(layer.vertical_speed);
        builder.write_bool32(layer.is_visible);
    }

    Ok(())
}


fn build_room_sequences(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, strings: &GMStrings, sequences: &Vec<GMSequence>) -> Result<(), String> {
    builder.write_usize(sequences.len());
    for sequence in sequences {
        build_sequence(data_builder, builder, general_info, strings, sequence)?;
    }
    Ok(())
}

