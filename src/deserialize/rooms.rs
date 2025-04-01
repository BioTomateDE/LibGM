use num_enum::TryFromPrimitive;
use crate::deserialize::backgrounds::{UTBackgroundRef, UTBackgrounds};
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::game_objects::{UTGameObjectRef, UTGameObjects};
use crate::deserialize::general_info::UTGeneralInfo;
use crate::deserialize::sequence::{parse_sequence, UTSequence};
use crate::deserialize::strings::{UTStringRef, UTStrings};


#[derive(Debug, Clone)]
pub struct UTRoom {
    pub name: UTStringRef,
    pub caption: UTStringRef,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub background_color: u32,
    pub draw_background_color: bool,
    pub creation_code_id: u32,
    pub flags: UTRoomFlags,
    pub backgrounds: Vec<UTRoomBackground>,
    pub views: Vec<UTRoomView>,
    pub game_objects: Vec<UTRoomGameObject>,
    pub tiles: Vec<UTRoomTile>,
    pub world: bool,
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub meters_per_pixel: f32,
    pub layers: Option<Vec<UTRoomLayer>>,
    pub sequences: Option<Vec<UTSequence>>,
}
#[derive(Debug, Clone)]
pub struct UTRoomFlags {
    pub enable_views: bool,                 // views are enabled
    pub show_color: bool,                   // meaning uncertain
    pub dont_clear_display_buffer: bool,    // don't clear display buffer
    pub is_gms2: bool,                      // room was made in GameMaker: Studio 2
    pub is_gms2_3: bool,                    // room was made in GameMaker: Studio 2.3
}

#[derive(Debug, Clone)]
pub struct UTRoomView {
    pub enabled: bool,
    pub view_x: i32,
    pub view_y: i32,
    pub view_width: i32,
    pub view_height: i32,
    pub port_x: i32,
    pub port_y: i32,
    pub port_width: i32,
    pub port_height: i32,
    pub border_x: u32,
    pub border_y: u32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub object_id: i32,           // change to UTObject later
}

#[derive(Debug, Clone, Copy)]
pub struct UTRoomBackground {
    pub enabled: bool,
    pub foreground: bool,
    pub background_definition: Option<UTBackgroundRef>,
    pub x: i32,
    pub y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub stretch: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct UTRoomTile {
    pub x: i32,
    pub y: i32,
    pub texture: UTRoomTileTexture,
    pub source_x: u32,
    pub source_y: u32,
    pub width: u32,
    pub height: u32,
    pub tile_depth: i32,
    pub instance_id: u32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum UTRoomTileTexture {
    Stub(),
    // Sprite(UTSprite),
    // Background(UTBackground),
}

#[derive(Debug, Clone)]
pub struct UTRoomLayer {
    pub layer_name: UTStringRef,
    pub layer_id: u32,
    pub layer_type: UTRoomLayerType,
    pub layer_depth: i32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
    pub is_visible: bool,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum UTRoomLayerType {
    Path = 0,
    Background = 1,
    Instances = 2,
    Assets = 3,
    Tiles = 4,
    Effect = 6,
}

#[derive(Debug, Clone)]
pub struct UTRoomGameObject {
    pub x: i32,
    pub y: i32,
    pub object_definition: UTGameObjectRef,
    pub instance_id: u32,
    pub creation_code: i32,     // {!!} change type to code ref
    pub scale_x: f32,
    pub scale_y: f32,
    pub image_speed: Option<f32>,
    pub image_index: Option<usize>,
    pub color: u32,
    pub rotation: f32,
    pub pre_create_code: Option<i32>,       // {!!} change type to (option) code ref
}


#[allow(non_snake_case)]
pub fn parse_chunk_ROOM(
    chunk: &mut UTChunk,
    general_info: &UTGeneralInfo,
    ut_strings: &UTStrings,
    ut_backgrounds: &UTBackgrounds,
    ut_game_objects: &UTGameObjects,
) -> Result<Vec<UTRoom>, String> {
    chunk.file_index = 0;
    let room_count: usize = chunk.read_usize()?;
    let mut room_starting_positions: Vec<usize> = Vec::with_capacity(room_count);
    for _ in 0..room_count {
        let start_position: usize = chunk.read_usize()? - chunk.abs_pos;
        room_starting_positions.push(start_position);
    }

    let mut rooms: Vec<UTRoom> = Vec::with_capacity(room_count);
    for start_position in room_starting_positions {
        chunk.file_index = start_position;

        let name: UTStringRef = chunk.read_ut_string(ut_strings)?;
        let caption: UTStringRef = chunk.read_ut_string(ut_strings)?;
        let width: u32 = chunk.read_u32()?;
        let height: u32 = chunk.read_u32()?;
        let speed: u32 = chunk.read_u32()?;
        let persistent: bool = chunk.read_u32()? != 0;
        let background_color: u32 = chunk.read_u32()? | 0xFF000000;     // make alpha 255 (background color doesn't have transparency)
        let draw_background_color: bool = chunk.read_u32()? != 0;
        let creation_code_id: u32 = chunk.read_u32()?;      // (can be -1) reference to code; change type later
        let flags: UTRoomFlags = parse_room_flags(chunk.read_u32()?);
        let backgrounds: Vec<UTRoomBackground> = parse_room_backgrounds(chunk, ut_backgrounds)?;
        let views: Vec<UTRoomView> = parse_room_views(chunk)?;
        let game_objects: Vec<UTRoomGameObject> = parse_room_objects(chunk, name.clone(), &general_info, &ut_strings, &ut_game_objects)?;
        let tiles: Vec<UTRoomTile> = parse_room_tiles(chunk, general_info)?;
        let world: bool = chunk.read_u32()? != 0;
        let top: u32 = chunk.read_u32()?;
        let left: u32 = chunk.read_u32()?;
        let right: u32 = chunk.read_u32()?;
        let bottom: u32 = chunk.read_u32()?;
        let gravity_x: f32 = chunk.read_f32()?;
        let gravity_y: f32 = chunk.read_f32()?;
        let meters_per_pixel: f32 = chunk.read_f32()?;
        let mut layers: Option<Vec<UTRoomLayer>> = None;
        let mut sequences: Option<Vec<UTSequence>> = None;
        if general_info.is_version_at_least(2, 0, 0, 0) {
            layers = Some(parse_room_layers(chunk, ut_strings)?);
            if general_info.is_version_at_least(2, 3, 0, 0) {
                sequences = Some(parse_room_sequences(chunk, ut_strings)?);
            }
        }

        let room: UTRoom = UTRoom {
            name,
            caption,
            width,
            height,
            speed,
            persistent,
            background_color,
            draw_background_color,
            creation_code_id,
            flags,
            backgrounds,
            views,
            game_objects,
            tiles,
            world,
            top,
            left,
            right,
            bottom,
            gravity_x,
            gravity_y,
            meters_per_pixel,
            layers,
            sequences,
        };
        // room.print();
        rooms.push(room);
    }

    Ok(rooms)
}


fn parse_room_flags(raw: u32) -> UTRoomFlags {
    UTRoomFlags {
        enable_views: 0 != raw & 1,
        show_color: 0 != raw & 2,
        dont_clear_display_buffer: 0 != raw & 4,
        is_gms2: 0 != raw & 131072,
        is_gms2_3: 0 != raw & 65536,
    }
}

fn parse_room_views(chunk: &mut UTChunk) -> Result<Vec<UTRoomView>, String> {
    let view_pointers: Vec<usize> = chunk.read_pointer_list()?;
    let old_position: usize = chunk.file_index;
    let mut views: Vec<UTRoomView> = Vec::with_capacity(view_pointers.len());

    for pointer in view_pointers {
        chunk.file_index = pointer;

        let enabled: bool = chunk.read_u32()? != 0;
        let view_x: i32 = chunk.read_i32()?;
        let view_y: i32 = chunk.read_i32()?;
        let view_width: i32 = chunk.read_i32()?;
        let view_height: i32 = chunk.read_i32()?;
        let port_x: i32 = chunk.read_i32()?;
        let port_y: i32 = chunk.read_i32()?;
        let port_width: i32 = chunk.read_i32()?;
        let port_height: i32 = chunk.read_i32()?;
        let border_x: u32 = chunk.read_u32()?;
        let border_y: u32 = chunk.read_u32()?;
        let speed_x: i32 = chunk.read_i32()?;
        let speed_y: i32 = chunk.read_i32()?;
        let object_id: i32 = chunk.read_i32()?;           // change to UTObject later

        let view: UTRoomView = UTRoomView {
            enabled,
            view_x,
            view_y,
            view_width,
            view_height,
            port_x,
            port_y,
            port_width,
            port_height,
            border_x,
            border_y,
            speed_x,
            speed_y,
            object_id,
        };
        views.push(view);
    }

    chunk.file_index = old_position;
    Ok(views)
}

fn parse_room_objects(
    chunk: &mut UTChunk,
    room_name: UTStringRef,
    general_info: &UTGeneralInfo,
    strings: &UTStrings,
    ut_game_objects: &UTGameObjects,
) -> Result<Vec<UTRoomGameObject>, String> {
    let game_object_pointers: Vec<usize> = chunk.read_pointer_list()?;
    let old_position: usize = chunk.file_index;
    let mut room_game_objects: Vec<UTRoomGameObject> = Vec::with_capacity(game_object_pointers.len());

    for pointer in game_object_pointers {
        chunk.file_index = pointer;
        let x: i32 = chunk.read_i32()?;
        let y: i32 = chunk.read_i32()?;
        let object_definition: usize = chunk.read_usize()?;
        let object_definition: UTGameObjectRef = match ut_game_objects.get_game_object_by_index(object_definition) {
            Some(obj) => obj,
            None => return Err(format!(
                "Could not find Game Object with index {} for Room with name \"{}\" at position {} in chunk '{}'.",
                object_definition,
                room_name.resolve(strings).unwrap_or_else(|_| "<InvalidStringRef>"),
                pointer,
                chunk.name,
            )),
        };
        let instance_id: u32 = chunk.read_u32()?;
        let creation_code: i32 = chunk.read_i32()?;     // {!!} change type to code ref
        let scale_x: f32 = chunk.read_f32()?;
        let scale_y: f32 = chunk.read_f32()?;
        let mut image_speed: Option<f32> = None;
        let mut image_index: Option<usize> = None;
        if general_info.is_version_at_least(2, 2, 2, 302) {
            image_speed = Some(chunk.read_f32()?);
            image_index = Some(chunk.read_usize()?);
        }
        let color: u32 = chunk.read_u32()?;
        let rotation: f32 = chunk.read_f32()?;
        let mut pre_create_code: Option<i32> = None;        // {!!} change type to code ref
        if general_info.bytecode_version >= 16 {        // [From UndertaleModTool] "is that dependent on bytecode or something else?"
            pre_create_code = Some(chunk.read_i32()?);
        }

        let room_game_object = UTRoomGameObject {
            x,
            y,
            object_definition,
            instance_id,
            creation_code,
            scale_x,
            scale_y,
            image_speed,
            image_index,
            color,
            rotation,
            pre_create_code,
        };
        room_game_objects.push(room_game_object);
    }

    chunk.file_index = old_position;
    Ok(room_game_objects)
}

fn parse_room_backgrounds(chunk: &mut UTChunk, ut_backgrounds: &UTBackgrounds) -> Result<Vec<UTRoomBackground>, String> {
    let background_pointers: Vec<usize> = chunk.read_pointer_list()?;
    let old_position: usize = chunk.file_index;
    let mut room_backgrounds: Vec<UTRoomBackground> = Vec::with_capacity(background_pointers.len());

    for pointer in background_pointers {
        chunk.file_index = pointer;
        let enabled: bool = chunk.read_i32()? != 0;
        let foreground: bool = chunk.read_i32()? != 0;
        let background_definition: i32 = chunk.read_i32()?;
        let background_definition: Option<UTBackgroundRef> =
            if background_definition == -1 { None }
            else { ut_backgrounds.get_background_by_index(background_definition as usize) };
        let x: i32 = chunk.read_i32()?;
        let y: i32 = chunk.read_i32()?;
        let tile_x: i32 = chunk.read_i32()?;    // idk if this should be an int instead of a bool
        let tile_y: i32 = chunk.read_i32()?;    // ^
        let speed_x: i32 = chunk.read_i32()?;
        let speed_y: i32 = chunk.read_i32()?;
        let stretch: bool = chunk.read_i32()? != 0;

        let background: UTRoomBackground = UTRoomBackground {
            enabled,
            foreground,
            background_definition,
            x,
            y,
            tile_x,
            tile_y,
            speed_x,
            speed_y,
            stretch,
        };
        room_backgrounds.push(background);
    }

    chunk.file_index = old_position;
    Ok(room_backgrounds)
}

fn parse_room_tiles(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<Vec<UTRoomTile>, String> {  // TODO
    let tile_pointers: Vec<usize> = chunk.read_pointer_list()?;
    let old_position: usize = chunk.file_index;
    let mut tiles: Vec<UTRoomTile> = Vec::with_capacity(tile_pointers.len());

    for pointer in tile_pointers {
        chunk.file_index = pointer;

        let x: i32 = chunk.read_i32()?;
        let y: i32 = chunk.read_i32()?;
        let _sprite_mode: bool = general_info.is_version_at_least(2, 0, 0, 0);
        // if sprite_mode {}
        let _ = chunk.read_u32()?;  // sprite shit
        let texture: UTRoomTileTexture = UTRoomTileTexture::Stub();
        let source_x: u32 = chunk.read_u32()?;
        let source_y: u32 = chunk.read_u32()?;
        let width: u32 = chunk.read_u32()?;
        let height: u32 = chunk.read_u32()?;
        let tile_depth: i32 = chunk.read_i32()?;
        let instance_id: u32 = chunk.read_u32()?;
        let scale_x: f32 = chunk.read_f32()?;
        let scale_y: f32 = chunk.read_f32()?;
        let color: u32 = chunk.read_u32()?;

        let tile: UTRoomTile = UTRoomTile {
            x,
            y,
            texture,
            source_x,
            source_y,
            width,
            height,
            tile_depth,
            instance_id,
            scale_x,
            scale_y,
            color,
        };
        tiles.push(tile);
    }

    chunk.file_index = old_position;
    Ok(tiles)
}

fn parse_room_layers(chunk: &mut UTChunk, strings: &UTStrings) -> Result<Vec<UTRoomLayer>, String> {
    let layer_pointers: Vec<usize> = chunk.read_pointer_list()?;
    let old_position: usize = chunk.file_index;
    let mut layers: Vec<UTRoomLayer> = Vec::with_capacity(layer_pointers.len());

    for pointer in layer_pointers {
        chunk.file_index = pointer;

        let layer_name: UTStringRef = chunk.read_ut_string(strings)?;
        let layer_id: u32 = chunk.read_u32()?;
        let layer_type: u32 = chunk.read_u32()?;
        let layer_type: UTRoomLayerType = match layer_type.try_into() {
            Ok(layer_type) => layer_type,
            Err(_) => return Err(format!(
                "Invalid Room Layer Type 0x{:04X} while parsing room at position {} in chunk '{}'.",
                layer_type,
                chunk.file_index,
                chunk.name,
            )),
        };
        let layer_depth: i32 = chunk.read_i32()?;
        let x_offset: f32 = chunk.read_f32()?;
        let y_offset: f32 = chunk.read_f32()?;
        let horizontal_speed: f32 = chunk.read_f32()?;
        let vertical_speed: f32 = chunk.read_f32()?;
        let is_visible: bool = chunk.read_u32()? != 0;

        let layer: UTRoomLayer = UTRoomLayer {
            layer_name,
            layer_id,
            layer_type,
            layer_depth,
            x_offset,
            y_offset,
            horizontal_speed,
            vertical_speed,
            is_visible,
        };
        layers.push(layer);
    }

    chunk.file_index = old_position;
    Ok(layers)
}

fn parse_room_sequences(chunk: &mut UTChunk, strings: &UTStrings) -> Result<Vec<UTSequence>, String> {
    let sequence_count: usize = chunk.read_usize()?;
    let mut sequences: Vec<UTSequence> = Vec::with_capacity(sequence_count);
    for _ in 0..sequence_count {
        sequences.push(parse_sequence(chunk, strings)?);
    }
    Ok(sequences)
}

