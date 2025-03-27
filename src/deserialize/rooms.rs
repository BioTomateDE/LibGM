use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::general_info::{parse_chunk_GEN8, UTGeneralInfo};
use crate::deserialize::sequence::{parse_sequence, UTSequence};
use crate::deserialize::strings::UTStrings;


#[derive(Debug, Clone)]
pub struct UTRoom {
    pub name: String,
    pub caption: String,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub background_color: u32,
    pub draw_background_color: bool,
    pub creation_code_id: u32,
    pub flags: UTRoomFlags,
    pub backgrounds: Vec<UTRoomBackground>,
    pub views: Vec<UTRoomView>,                  // change type
    pub game_objects: Vec<usize>,           // change type
    pub tiles: Vec<usize>,                  // change type
    pub world: bool,
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub meters_per_pixel: f32,
    pub layers: Option<Vec<usize>>,         // change type
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
    pub object_id: usize,           // change to UTObject later
}

#[derive(Debug, Clone, Copy)]
pub struct UTRoomBackground {
    pub enabled: bool,
    pub foreground: bool,
    pub background_definition: u32,     // pointer to background; change type later
    pub x: i32,
    pub y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub stretch: bool,
}


pub fn parse_chunk_ROOM(mut chunk: UTChunk, general_info: &UTGeneralInfo, strings: &UTStrings) -> Result<Vec<UTRoom>, String> {
    let room_count: usize = chunk.read_usize()?;
    let mut room_starting_positions: Vec<usize> = Vec::with_capacity(room_count);
    for _ in 0..room_count {
        let start_position: usize = chunk.read_usize()? - chunk.abs_pos;
        room_starting_positions.push(start_position);
    }

    let mut rooms: Vec<UTRoom> = Vec::with_capacity(room_count);
    for start_position in room_starting_positions {
        chunk.file_index = start_position;

        let name: String = chunk.read_ut_string(strings)?;
        let caption: String = chunk.read_ut_string(strings)?;
        let width: u32 = chunk.read_u32()?;
        let height: u32 = chunk.read_u32()?;
        let speed: u32 = chunk.read_u32()?;
        let persistent: bool = chunk.read_u32()? != 0;
        let background_color: u32 = chunk.read_u32()? | 0xFF000000;     // make alpha 255 (background color doesn't have transparency)
        let draw_background_color: bool = chunk.read_u32()? != 0;
        let creation_code_id: u32 = chunk.read_u32()?;      // (can be -1) reference to code; change type later
        let flags: UTRoomFlags = parse_room_flags(chunk.read_u32()?);
        let backgrounds: Vec<UTRoomBackground> = parse_room_backgrounds(&mut chunk)?;        // change type later
        let views: Vec<UTRoomView> = parse_room_views(&mut chunk)?;
        let game_objects: Vec<usize> = parse_room_objects(&mut chunk)?;     // change to UTObject
        let tiles: Vec<usize> = parse_room_tiles(&mut chunk)?;     // change to UTTile
        let world: bool = chunk.read_u32()? != 0;
        let top: u32 = chunk.read_u32()?;
        let left: u32 = chunk.read_u32()?;
        let right: u32 = chunk.read_u32()?;
        let bottom: u32 = chunk.read_u32()?;
        let gravity_x: f32 = chunk.read_f32()?;
        let gravity_y: f32 = chunk.read_f32()?;
        let meters_per_pixel: f32 = chunk.read_f32()?;
        let mut layers: Option<Vec<usize>> = None;      // change to UTRoomLayer
        let mut sequences: Option<Vec<UTSequence>> = None;      // change to UTSequence
        if general_info.is_version_at_least(2, 0, 0, 0) {
            layers = Some(parse_room_layers(&mut chunk)?);
            if general_info.is_version_at_least(2, 3, 0, 0) {
                sequences = Some(parse_room_sequences(&mut chunk, strings)?);
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
        room.print();
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

fn parse_room_views(chunk: &mut UTChunk) -> Result<Vec<UTRoomView>, String> {  // TODO
    // pointer list bhudsfgbdgs

    // Enabled = reader.ReadBoolean();
    // ViewX = reader.ReadInt32();
    // ViewY = reader.ReadInt32();
    // ViewWidth = reader.ReadInt32();
    // ViewHeight = reader.ReadInt32();
    // PortX = reader.ReadInt32();
    // PortY = reader.ReadInt32();
    // PortWidth = reader.ReadInt32();
    // PortHeight = reader.ReadInt32();
    // BorderX = reader.ReadUInt32();
    // BorderY = reader.ReadUInt32();
    // SpeedX = reader.ReadInt32();
    // SpeedY = reader.ReadInt32();
    // _objectId = reader.ReadUndertaleObject<UndertaleResourceById<UndertaleGameObject, UndertaleChunkOBJT>>();

    Ok(vec![])
}


fn parse_room_backgrounds(chunk: &mut UTChunk) -> Result<Vec<UTRoomBackground>, String> {
    let pointer_position: usize = chunk.read_usize()? - chunk.abs_pos;
    let old_position: usize = chunk.file_index;
    chunk.file_index = pointer_position;

    let background_count: usize = chunk.read_usize()?;
    let mut backgrounds: Vec<UTRoomBackground> = Vec::with_capacity(background_count);
    for _ in 0..background_count {
        let background_pointer = chunk.read_usize()? - chunk.abs_pos;
        let old_position2: usize = chunk.file_index;
        chunk.file_index = background_pointer;
        let background: UTRoomBackground = parse_room_background(chunk)?;
        chunk.file_index = old_position2;
        backgrounds.push(background);
    }

    chunk.file_index = old_position;
    Ok(backgrounds)
}


fn parse_room_background(chunk: &mut UTChunk) -> Result<UTRoomBackground, String> {
    let enabled: bool = chunk.read_i32()? != 0;
    let foreground: bool = chunk.read_i32()? != 0;
    let background_definition: u32 = chunk.read_u32()?;     // change to UTBackground later
    let x: i32 = chunk.read_i32()?;
    let y: i32 = chunk.read_i32()?;
    let tile_x: i32 = chunk.read_i32()?;    // idk if this should be an int instead of a bool
    let tile_y: i32 = chunk.read_i32()?;    // ^
    let speed_x: i32 = chunk.read_i32()?;
    let speed_y: i32 = chunk.read_i32()?;
    let stretch: bool = chunk.read_i32()? != 0;

    Ok(UTRoomBackground {
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
    })
}

fn parse_room_objects(chunk: &mut UTChunk) -> Result<Vec<usize>, String> {  // TODO
    // pointer list bhudsfgbdgs

    Ok(vec![])
}


fn parse_room_tiles(chunk: &mut UTChunk) -> Result<Vec<usize>, String> {  // TODO
    // pointer list bhudsfgbdgs

    Ok(vec![])
}

fn parse_room_layers(chunk: &mut UTChunk) -> Result<Vec<usize>, String> {  // TODO
    // pointer list bhudsfgbdgs

    Ok(vec![])
}

fn parse_room_sequences(chunk: &mut UTChunk, strings: &UTStrings) -> Result<Vec<UTSequence>, String> {
    let sequence_count: usize = chunk.read_usize()?;
    let mut sequences: Vec<UTSequence> = Vec::with_capacity(sequence_count);
    for _ in 0..sequence_count {
        sequences.push(parse_sequence(chunk, strings)?);
    }
    Ok(sequences)
}

