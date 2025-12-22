use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::room::layer::Type, gm_version::GMVersionReq,
    },
    prelude::*,
    util::init::num_enum_from,
};

pub fn check_2022_1(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2022, 1).into()));
    // Iterate over all rooms until a length check is performed

    let room_count = reader.read_u32()?;
    for room_index in 0..room_count {
        // Advance to room data we're interested in (and grab pointer for next room)
        reader.set_rel_cur_pos(4 * room_index + 4)?;
        let room_pointer = reader.read_u32()?;
        reader.cur_pos = room_pointer + 22 * 4;

        // Get the pointer for this room's layer list, as well as pointer to sequence list
        let layer_list_pointer = reader.read_u32()?;
        let sequence_pointer = reader.read_u32()?;
        reader.cur_pos = layer_list_pointer;
        let layer_count = reader.read_i32()?;
        if layer_count < 1 {
            continue; // No layers to detect; go to next room
        }

        // Get pointer into the individual layer data (plus 8 bytes) for the first layer in the room
        let jump_pointer = reader.read_u32()? + 8;

        // Find the offset for the end of this layer
        let next_pointer = if layer_count == 1 {
            sequence_pointer
        } else {
            reader.read_u32()? // Pointer to next element in the layer list
        };

        // Actually perform the length checks, depending on layer data
        reader.cur_pos = jump_pointer;
        let layer_type = reader.read_i32()?;
        let Ok(layer_type) = Type::try_from(layer_type) else {
            continue;
        };

        match layer_type {
            Type::Path | Type::Path2 => continue,
            Type::Background => {
                if next_pointer - reader.cur_pos > 16 * 4 {
                    return target_ver;
                }
            },
            Type::Instances => {
                reader.cur_pos += 6 * 4;
                let instance_count = reader.read_u32()?;
                if next_pointer - reader.cur_pos != instance_count * 4 {
                    return target_ver;
                }
            },
            Type::Assets => {
                reader.cur_pos += 6 * 4;
                let tile_pointer = reader.read_u32()?;
                if tile_pointer != reader.cur_pos + 8 && tile_pointer != reader.cur_pos + 12 {
                    return target_ver;
                }
            },
            Type::Tiles => {
                reader.cur_pos += 6 * 4;
                let tile_map_width = reader.read_u32()?;
                let tile_map_height = reader.read_u32()?;
                if next_pointer - reader.cur_pos != (tile_map_width * tile_map_height * 4) {
                    return target_ver;
                }
            },
            Type::Effect => {
                reader.cur_pos += 7 * 4;
                let property_count = reader.read_u32()?;
                if next_pointer - reader.cur_pos != (property_count * 3 * 4) {
                    return target_ver;
                }
            },
        }
        return Ok(None); // Check complete, found and tested a layer (but didn't detect 2022.1)
    }

    Ok(None)
}

pub fn check_2_2_2_302(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    // Check the size of the first GameObject in a room
    let room_count = reader.read_u32()?;

    for room_index in 0..room_count {
        // Advance to room data we're interested in (and grab pointer for next room)
        reader.set_rel_cur_pos(4 * room_index + 4)?;
        let room_pointer = reader.read_u32()?;
        reader.cur_pos = room_pointer + 12 * 4;

        // Get the pointer for this room's object list, as well as pointer to tile list
        let object_list_pointer = reader.read_u32()?;
        let tile_list_pointer = reader.read_u32()?;
        reader.cur_pos = object_list_pointer;
        let object_count = reader.read_u32()?;
        if object_count < 1 {
            continue; // No objects => nothing to detect; go to next room
        }

        let pointer1 = reader.read_u32()?;
        let pointer2 = if object_count == 1 {
            tile_list_pointer // Tile list starts right after, so it works as an alternate
        } else {
            reader.read_u32()?
        };
        if pointer2 - pointer1 == 48 {
            return Ok(Some((2, 2, 2, 302).into()));
        }
    }

    Ok(None)
}

pub fn check_2024_2_and_2024_4(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    // Check for tile compression
    let room_count = reader.read_u32()?;
    let mut any_layers_misaligned: bool = false;

    for room_index in 0..room_count {
        // Advance to room data we're interested in (and grab pointer for next room)
        reader.set_rel_cur_pos(4 * room_index + 4)?;
        let room_pointer = reader.read_u32()?;
        reader.cur_pos = room_pointer + 22 * 4;

        // Get the pointer for this room's layer list, as well as pointer to sequence list
        let layer_list_ptr = reader.read_u32()?;
        let sequence_ptr = reader.read_u32()?;
        reader.cur_pos = layer_list_ptr;
        let layer_count = reader.read_u32()?;
        if layer_count < 1 {
            continue; // No layers to detect; go to next room
        }

        let mut check_next_layer_offset: bool = false;
        for layer_index in 0..layer_count {
            let layer_ptr = layer_list_ptr + 4 * layer_index;
            if check_next_layer_offset && layer_ptr % 4 != 0 {
                any_layers_misaligned = true;
            }

            reader.cur_pos = layer_ptr + 4;
            // Get pointer into the individual layer data
            let layer_data_ptr = reader.read_u32()?;

            // Find the offset for the end of this layer
            let next_pointer = if layer_index == layer_count - 1 {
                sequence_ptr
            } else {
                reader.read_u32()? // Pointer to next element in the layer list
            };

            // Actually perform the length checks
            reader.cur_pos = layer_data_ptr + 8;
            let layer_type: Type = num_enum_from(reader.read_i32()?)?;
            if layer_type != Type::Tiles {
                check_next_layer_offset = false;
                continue;
            }
            check_next_layer_offset = true;
            reader.cur_pos += 32;
            let effect_count = reader.read_u32()?;
            reader.cur_pos += 12 * effect_count + 4;

            let tile_map_width = reader.read_u32()?;
            let tile_map_height = reader.read_u32()?;
            if next_pointer - reader.cur_pos != (tile_map_width * tile_map_height * 4) {
                return if any_layers_misaligned {
                    Ok(Some((2024, 2).into()))
                } else {
                    Ok(Some((2024, 4).into()))
                };
            }
        }
    }

    Ok(None)
}
