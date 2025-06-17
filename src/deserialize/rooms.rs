use crate::deserialize::chunk_reading::GMRef;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use crate::deserialize::backgrounds::{GMBackground, GMBackgroundGMS2Data};
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::code::GMCode;
use crate::deserialize::fonts::GMFont;
use crate::deserialize::game_objects::{GMGameObject};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::particles::{GMParticleSystem, GMParticleSystems};
use crate::deserialize::sequence::{parse_sequence, GMAnimSpeedType, GMSequence};
use crate::deserialize::sprites::GMSprite;
use crate::deserialize::strings::GMStrings;


#[derive(Debug, Clone, PartialEq)]
pub struct GMRoom {
    pub name: GMRef<String>,
    pub caption: Option<GMRef<String>>,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub background_color: u32,
    pub draw_background_color: bool,
    pub creation_code: Option<GMRef<GMCode>>,
    pub flags: GMRoomFlags,
    pub backgrounds: Vec<GMRoomBackground>,
    pub views: Vec<GMRoomView>,
    pub game_objects: Vec<GMRoomGameObject>,
    pub tiles: Vec<GMRoomTile>,
    pub world: bool,
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub meters_per_pixel: f32,
    pub layers: Option<Vec<GMRoomLayer>>,
    pub sequences: Option<Vec<GMSequence>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMRoomFlags {
    pub enable_views: bool,                 // views are enabled
    pub show_color: bool,                   // meaning uncertain
    pub dont_clear_display_buffer: bool,    // don't clear display buffer
    pub is_gms2: bool,                      // room was made in GameMaker: Studio 2
    pub is_gms2_3: bool,                    // room was made in GameMaker: Studio 2.3
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomView {
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
    pub object: Option<GMRef<GMGameObject>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomBackground {
    pub enabled: bool,
    pub foreground: bool,
    pub background_definition: Option<GMRef<GMBackground>>,
    pub x: i32,
    pub y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub stretch: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomTile {
    pub x: i32,
    pub y: i32,
    pub texture: GMRoomTileTexture,
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

#[derive(Debug, Clone, PartialEq)]
pub enum GMRoomTileTexture {
    Sprite(GMRef<GMSprite>),
    Background(GMRef<GMBackground>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayer {
    pub layer_name: GMRef<String>,
    pub layer_id: u32,
    pub layer_type: GMRoomLayerType,
    pub layer_depth: i32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
    pub is_visible: bool,
    pub data_2022_1: Option<GMRoomLayer2022_1>,
    pub data: GMRoomLayerData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayer2022_1 {
    pub effect_enabled: bool,
    pub effect_type: GMRef<String>, 
    pub effect_properties: Vec<GMRoomLayerEffectProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerEffectProperty {
    pub kind: GMRoomLayerEffectPropertyType,
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}

#[derive(Debug, Clone, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMRoomLayerEffectPropertyType {
    Real = 0,
    Color = 1,
    Sampler = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum GMRoomLayerType {
    Path = 0,
    Background = 1,
    Instances = 2,
    Assets = 3,
    Tiles = 4,
    Effect = 6,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMRoomLayerData {
    None,
    Instances(GMRoomLayerDataInstances),
    Tiles(GMRoomLayerDataTiles),
    Background(GMRoomLayerDataBackground),
    Assets(GMRoomLayerDataAssets),
    Effect(GMRoomLayerDataEffect),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataInstances {
    pub instances: Vec<GMRoomGameObject>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataTiles {
    pub background: GMRef<GMBackground>,
    /// Flattened 2D Array. Access using `tile_data[row * width * col]`.
    pub tile_data: Vec<u32>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataBackground {
    pub visible: bool,
    pub foreground: bool,
    pub sprite: GMRef<GMSprite>,
    pub tiled_horizontally: bool,
    pub tiled_vertically: bool,
    pub stretch: bool,
    pub color: u32,
    pub first_frame: f32,
    pub animation_speed: f32,
    pub animation_speed_type: GMAnimSpeedType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataAssets {
    pub legacy_tiles: Vec<GMRoomTile>,
    pub sprites: Vec<GMSpriteInstance>,
    pub sequences: Vec<GMSequenceInstance>,
    pub nine_slices: Vec<GMSpriteInstance>,
    pub particle_systems: Vec<GMParticleSystemInstance>,
    pub text_items: Vec<GMTextItemInstance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataEffect {
    pub effect_type: GMRef<String>,
    pub properties: Vec<GMRoomLayerEffectProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteInstance {
    pub name: GMRef<String>,
    pub sprite: GMRef<GMSprite>,
    pub x: i32,
    pub y: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
    pub animation_speed: f32,
    pub animation_speed_type: GMAnimSpeedType,
    pub frame_index: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSequenceInstance {
    pub name: GMRef<String>,
    pub sequence: GMRef<GMSequence>,
    pub x: i32,
    pub y: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
    pub animation_speed: f32,
    pub animation_speed_type: GMAnimSpeedType,
    pub frame_index: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleSystemInstance {
    pub name: GMRef<String>,
    pub particle_system: GMRef<GMParticleSystem>,
    pub x: i32,
    pub y: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMTextItemInstance {
    pub name: GMRef<String>,
    pub x: i32,
    pub y: i32,
    pub font: GMRef<GMFont>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub color: u32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub text: GMRef<String>,
    pub alignment: i32,
    pub character_spacing: f32,
    pub line_spacing: f32,
    pub frame_width: f32,
    pub frame_height: f32,
    pub wrap: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomGameObject {
    pub x: i32,
    pub y: i32,
    pub object_definition: GMRef<GMGameObject>,
    pub instance_id: u32,
    pub creation_code: Option<GMRef<GMCode>>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub image_speed: Option<f32>,
    pub image_index: Option<usize>,
    pub color: u32,
    pub rotation: f32,
    pub pre_create_code: Option<GMRef<GMCode>>,
    // pub exists: bool,
}


#[derive(Debug, Clone)]
pub struct GMRooms {
    pub rooms_by_index: Vec<GMRoom>,
}


pub fn parse_chunk_room(chunk: &mut GMChunk, general_info: &GMGeneralInfo, gm_strings: &GMStrings) -> Result<GMRooms, String> {
    chunk.cur_pos = 0;
    let room_count: usize = chunk.read_usize_count()?;
    let mut room_starting_positions: Vec<usize> = Vec::with_capacity(room_count);
    for _ in 0..room_count {
        let start_position: usize = chunk.read_usize_pos()? - chunk.abs_pos;
        room_starting_positions.push(start_position);
    }

    let mut rooms_by_index: Vec<GMRoom> = Vec::with_capacity(room_count);
    for start_position in room_starting_positions {
        chunk.cur_pos = start_position;

        let name: GMRef<String> = chunk.read_gm_string(gm_strings)?;
        let caption: Option<GMRef<String>> = chunk.read_gm_string_optional(gm_strings)?;
        let width: u32 = chunk.read_u32()?;
        let height: u32 = chunk.read_u32()?;
        let speed: u32 = chunk.read_u32()?;
        let persistent: bool = chunk.read_bool32()?;
        let background_color: u32 = chunk.read_u32()? | 0xFF000000;     // make alpha 255 (background color doesn't have transparency)
        let draw_background_color: bool = chunk.read_bool32()?;
        let creation_code_id: i32 = chunk.read_i32()?;
        let creation_code: Option<GMRef<GMCode>> = if creation_code_id == -1 { None } else { Some(GMRef::new(creation_code_id as usize)) };
        let flags: GMRoomFlags = parse_room_flags(chunk.read_u32()?);
        let backgrounds: Vec<GMRoomBackground> = parse_room_backgrounds(chunk)?;
        let views: Vec<GMRoomView> = parse_room_views(chunk)?;
        let game_objects: Vec<GMRoomGameObject> = parse_room_objects(chunk, &general_info)?;
        let tiles: Vec<GMRoomTile> = parse_room_tiles(chunk, general_info)?;
        let world: bool = chunk.read_bool32()?;
        let top: u32 = chunk.read_u32()?;
        let left: u32 = chunk.read_u32()?;
        let right: u32 = chunk.read_u32()?;
        let bottom: u32 = chunk.read_u32()?;
        let gravity_x: f32 = chunk.read_f32()?;
        let gravity_y: f32 = chunk.read_f32()?;
        let meters_per_pixel: f32 = chunk.read_f32()?;
        let mut layers: Option<Vec<GMRoomLayer>> = None;
        let mut sequences: Option<Vec<GMSequence>> = None;
        if general_info.is_version_at_least(2, 0, 0, 0) {
            layers = Some(parse_room_layers(chunk, general_info, gm_strings, &game_objects)?);
            if general_info.is_version_at_least(2, 3, 0, 0) {
                sequences = Some(parse_room_sequences(chunk, general_info, gm_strings)?);
            }
        }

        let room: GMRoom = GMRoom {
            name,
            caption,
            width,
            height,
            speed,
            persistent,
            background_color,
            draw_background_color,
            creation_code,
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
        rooms_by_index.push(room);
    }

    Ok(GMRooms{ rooms_by_index })
}


fn parse_room_flags(raw: u32) -> GMRoomFlags {
    GMRoomFlags {
        enable_views: 0 != raw & 1,
        show_color: 0 != raw & 2,
        dont_clear_display_buffer: 0 != raw & 4,
        is_gms2: 0 != raw & 131072,
        is_gms2_3: 0 != raw & 65536,
    }
}

fn parse_room_views(chunk: &mut GMChunk) -> Result<Vec<GMRoomView>, String> {
    let view_pointers: Vec<usize> = chunk.read_pointer_to_pointer_list()?;
    let old_position: usize = chunk.cur_pos;
    let mut views: Vec<GMRoomView> = Vec::with_capacity(view_pointers.len());

    for pointer in view_pointers {
        chunk.cur_pos = pointer;

        let enabled: bool = chunk.read_bool32()?;
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
        let object_id: i32 = chunk.read_i32()?;
        let object: Option<GMRef<GMGameObject>> = if object_id == -1 { None } else { Some(GMRef::new(object_id as usize)) };

        let view: GMRoomView = GMRoomView {
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
            object,
        };
        views.push(view);
    }

    chunk.cur_pos = old_position;
    Ok(views)
}

fn parse_room_objects(
    chunk: &mut GMChunk,
    general_info: &GMGeneralInfo,
) -> Result<Vec<GMRoomGameObject>, String> {
    let game_object_pointers: Vec<usize> = chunk.read_pointer_to_pointer_list()?;
    let old_position: usize = chunk.cur_pos;
    let mut room_game_objects: Vec<GMRoomGameObject> = Vec::with_capacity(game_object_pointers.len());

    for pointer in game_object_pointers {
        chunk.cur_pos = pointer;
        let x: i32 = chunk.read_i32()?;
        let y: i32 = chunk.read_i32()?;
        let object_definition: usize = chunk.read_usize_count()?;
        let object_definition: GMRef<GMGameObject> = GMRef::new(object_definition);
        let instance_id: u32 = chunk.read_u32()?;
        let creation_code_id: i32 = chunk.read_i32()?;
        let creation_code: Option<GMRef<GMCode>> = if creation_code_id == -1 { None } else { Some(GMRef::new(creation_code_id as usize)) };
        let scale_x: f32 = chunk.read_f32()?;
        let scale_y: f32 = chunk.read_f32()?;
        let mut image_speed: Option<f32> = None;
        let mut image_index: Option<usize> = None;
        if general_info.is_version_at_least(2, 2, 2, 302) {
            image_speed = Some(chunk.read_f32()?);
            image_index = Some(chunk.read_usize_pos()?);
        }
        let color: u32 = chunk.read_u32()?;
        let rotation: f32 = chunk.read_f32()?;

        // [From UndertaleModTool] "is that dependent on bytecode or something else?"
        let pre_create_code: Option<GMRef<GMCode>> = if general_info.bytecode_version <= 15 { None } else {
            if chunk.read_i32()? == -1 { None } else { Some(GMRef::new(creation_code_id as usize)) }
        };

        let room_game_object = GMRoomGameObject {
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

    chunk.cur_pos = old_position;
    Ok(room_game_objects)
}

fn parse_room_backgrounds(chunk: &mut GMChunk) -> Result<Vec<GMRoomBackground>, String> {
    let background_pointers: Vec<usize> = chunk.read_pointer_to_pointer_list()?;
    let old_position: usize = chunk.cur_pos;
    let mut room_backgrounds: Vec<GMRoomBackground> = Vec::with_capacity(background_pointers.len());

    for pointer in background_pointers {
        chunk.cur_pos = pointer;
        let enabled: bool = chunk.read_bool32()?;
        let foreground: bool = chunk.read_bool32()?;
        let background_definition: i32 = chunk.read_i32()?;
        let background_definition: Option<GMRef<GMBackground>> =
            if background_definition == -1 { None }
            else { Some(GMRef::new(background_definition as usize)) };
        let x: i32 = chunk.read_i32()?;
        let y: i32 = chunk.read_i32()?;
        let tile_x: i32 = chunk.read_i32()?;    // idk if this should be an int instead of a bool
        let tile_y: i32 = chunk.read_i32()?;    // ^
        let speed_x: i32 = chunk.read_i32()?;
        let speed_y: i32 = chunk.read_i32()?;
        let stretch: bool = chunk.read_bool32()?;

        let background: GMRoomBackground = GMRoomBackground {
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

    chunk.cur_pos = old_position;
    Ok(room_backgrounds)
}

fn parse_room_tiles(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<Vec<GMRoomTile>, String> {
    let tile_pointers: Vec<usize> = chunk.read_pointer_to_pointer_list()?;
    let old_position: usize = chunk.cur_pos;
    let mut tiles: Vec<GMRoomTile> = Vec::with_capacity(tile_pointers.len());

    for pointer in tile_pointers {
        chunk.cur_pos = pointer;
        let tile: GMRoomTile = parse_room_tile(chunk, general_info)?;
        tiles.push(tile);
    }

    chunk.cur_pos = old_position;
    Ok(tiles)
}

fn parse_room_tile(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMRoomTile, String> {
    let x: i32 = chunk.read_i32()?;
    let y: i32 = chunk.read_i32()?;
    let texture_index: usize = chunk.read_usize_count()?;
    let texture: GMRoomTileTexture = if general_info.is_version_at_least(2, 0, 0, 0) {
        GMRoomTileTexture::Sprite(GMRef::new(texture_index))
    } else {
        GMRoomTileTexture::Background(GMRef::new(texture_index))
    };
    let source_x: u32 = chunk.read_u32()?;
    let source_y: u32 = chunk.read_u32()?;
    let width: u32 = chunk.read_u32()?;
    let height: u32 = chunk.read_u32()?;
    let tile_depth: i32 = chunk.read_i32()?;
    let instance_id: u32 = chunk.read_u32()?;
    let scale_x: f32 = chunk.read_f32()?;
    let scale_y: f32 = chunk.read_f32()?;
    let color: u32 = chunk.read_u32()?;

    Ok(GMRoomTile {
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
    })
}

fn parse_room_layers(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings, room_game_objects: &Vec<GMRoomGameObject>) -> Result<Vec<GMRoomLayer>, String> {
    let layer_pointers: Vec<usize> = chunk.read_pointer_to_pointer_list()?;
    let old_position: usize = chunk.cur_pos;
    let mut layers: Vec<GMRoomLayer> = Vec::with_capacity(layer_pointers.len());

    for pointer in layer_pointers {
        chunk.cur_pos = pointer;

        let layer_name: GMRef<String> = chunk.read_gm_string(strings)?;
        let layer_id: u32 = chunk.read_u32()?;
        let layer_type: u32 = chunk.read_u32()?;
        let layer_type: GMRoomLayerType = layer_type.try_into().map_err(|_| format!(
            "Invalid Room Layer Type 0x{:04X} while parsing room at position {} in chunk '{}'",
            layer_type, chunk.cur_pos, chunk.name,
        ))?;
        let layer_depth: i32 = chunk.read_i32()?;
        let x_offset: f32 = chunk.read_f32()?;
        let y_offset: f32 = chunk.read_f32()?;
        let horizontal_speed: f32 = chunk.read_f32()?;
        let vertical_speed: f32 = chunk.read_f32()?;
        let is_visible: bool = chunk.read_bool32()?;
        
        let data_2022_1: Option<GMRoomLayer2022_1> = if general_info.is_version_at_least(2022, 1, 0, 0) {
            let effect_enabled = chunk.read_bool32()?;
            let effect_type = chunk.read_gm_string(strings)?;
            let effect_properties_len = chunk.read_usize_count()?;
            let mut effect_properties: Vec<GMRoomLayerEffectProperty> = Vec::with_capacity(effect_properties_len);
            for _ in 0..effect_properties_len {
                let kind: i32 = chunk.read_i32()?;
                let kind: GMRoomLayerEffectPropertyType = kind.try_into()
                    .map_err(|_| format!("Invalid Room Layer Effect Property {kind} (0x{kind:08X})"))?;
                let name: GMRef<String> = chunk.read_gm_string(strings)?;
                let value: GMRef<String> = chunk.read_gm_string(strings)?;
                effect_properties.push(GMRoomLayerEffectProperty { kind, name, value });
            }
            Some(GMRoomLayer2022_1 {
                effect_enabled,
                effect_type,
                effect_properties,
            })
        } else {
            None
        };

        let data: GMRoomLayerData = match layer_type {
            GMRoomLayerType::Path => GMRoomLayerData::None,
            GMRoomLayerType::Background => GMRoomLayerData::Background(parse_layer_background(chunk, general_info)?),
            GMRoomLayerType::Instances => GMRoomLayerData::Instances(parse_layer_instances(chunk, general_info, room_game_objects)?),
            GMRoomLayerType::Assets => GMRoomLayerData::Assets(parse_layer_assets(chunk, general_info, strings)?),
            GMRoomLayerType::Tiles => GMRoomLayerData::Tiles(parse_layer_tiles(chunk, general_info)?),
            GMRoomLayerType::Effect => GMRoomLayerData::Effect(parse_layer_effect(chunk, general_info, strings)?),
        };
        
        let layer: GMRoomLayer = GMRoomLayer {
            layer_name,
            layer_id,
            layer_type,
            layer_depth,
            x_offset,
            y_offset,
            horizontal_speed,
            vertical_speed,
            is_visible,
            data_2022_1,
            data,
        };
        layers.push(layer);
    }

    chunk.cur_pos = old_position;
    Ok(layers)
}

fn parse_room_sequences(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<Vec<GMSequence>, String> {
    let sequence_count: usize = chunk.read_usize_count()?;
    let mut sequences: Vec<GMSequence> = Vec::with_capacity(sequence_count);
    for _ in 0..sequence_count {
        sequences.push(parse_sequence(chunk, general_info, strings)?);
    }
    Ok(sequences)
}

fn get_room_game_object_by_instance_id(room_game_objects: &Vec<GMRoomGameObject>, instance_id: u32) -> Option<GMRoomGameObject> {
    for obj in room_game_objects {
        if obj.instance_id == instance_id {
            return Some(obj.clone())
        }
    }
    None
}

fn parse_layer_instances(chunk: &mut GMChunk, general_info: &GMGeneralInfo, room_game_objects: &Vec<GMRoomGameObject>) -> Result<GMRoomLayerDataInstances, String> {
    let count: usize = chunk.read_usize_count()?;
    let mut instances: Vec<GMRoomGameObject> = Vec::with_capacity(count);
    // {~~} check scuffed conditions for false positive idk

    for _ in 0..count {
        let instance_id: u32 = chunk.read_u32()?;
        let room_game_object: GMRoomGameObject = get_room_game_object_by_instance_id(room_game_objects, instance_id)
            .ok_or_else(|| format!("Nonexistent room game objects are not supported yet (has instance id {instance_id}"))?;
        instances.push(room_game_object);
    }

    Ok(GMRoomLayerDataInstances {
        instances,
    })
}

fn parse_layer_tiles(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMRoomLayerDataTiles, String> {
    let background: GMRef<GMBackground> = GMRef::new(chunk.read_usize_count()?);
    let width: usize = chunk.read_usize_count()?;
    let height: usize = chunk.read_usize_count()?;
    if general_info.is_version_at_least(2024, 2, 0, 0) {
        return Err("Compressed tile data (GM >= 2024.2) is not supported yet. Report this error to GitHub".to_string())     // TODO
    }
    let mut tile_data: Vec<u32> = Vec::with_capacity(width * height);
    for _y in 0..height {
        for _x in 0..width {
            tile_data.push(chunk.read_u32()?);
        }
    }

    Ok(GMRoomLayerDataTiles {
        background,
        tile_data,
        width,
        height,
    })
}

fn parse_layer_background(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMRoomLayerDataBackground, String> {
    let visible: bool = chunk.read_bool32()?;
    let foreground: bool = chunk.read_bool32()?;
    let sprite: GMRef<GMSprite> = GMRef::new(chunk.read_usize_count()?);
    let tiled_horizontally: bool = chunk.read_bool32()?;
    let tiled_vertically: bool = chunk.read_bool32()?;
    let stretch: bool = chunk.read_bool32()?;
    let color: u32 = chunk.read_u32()?;
    let first_frame: f32 = chunk.read_f32()?;
    let animation_speed: f32 = chunk.read_f32()?;
    let animation_speed_type: u32 = chunk.read_u32()?;
    let animation_speed_type: GMAnimSpeedType = animation_speed_type.try_into()
        .map_err(|_| format!("Invalid Animation Speed Type {0} (0x{0:08X}) while parsing Room Background Layer", animation_speed_type))?;

    Ok(GMRoomLayerDataBackground {
        visible,
        foreground,
        sprite,
        tiled_horizontally,
        tiled_vertically,
        stretch,
        color,
        first_frame,
        animation_speed,
        animation_speed_type,
    })
}

fn parse_layer_assets(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMRoomLayerDataAssets, String> {
    let legacy_tiles_pointer: usize = chunk.read_usize_pos()?;
    let sprites_pointer: usize = chunk.read_usize_pos()?;
    let mut sequences_pointer: Option<usize> = None;
    let mut nine_slices_pointer: Option<usize> = None;
    let mut particle_systems_pointer: Option<usize> = None;
    let mut text_items_pointer: Option<usize> = None;
    
    if general_info.is_version_at_least(2, 3, 0, 0) {
        sequences_pointer = Some(chunk.read_usize_pos()?);
    }
    if !general_info.is_version_at_least(2, 3, 2, 0) {
        nine_slices_pointer = Some(chunk.read_usize_pos()?);
    }
    if general_info.is_version_at_least(2023, 2, 0, 0) {   // {~~} non LTS
        particle_systems_pointer = Some(chunk.read_usize_pos()?);
    }
    if general_info.is_version_at_least(2024, 6, 0, 0) {
        text_items_pointer = Some(chunk.read_usize_pos()?);
    }

    let legacy_tile_count: usize = chunk.read_usize_count()?;
    let mut legacy_tiles: Vec<GMRoomTile> = Vec::with_capacity(legacy_tile_count);
    for _ in 0..legacy_tile_count {
        legacy_tiles.push(parse_room_tile(chunk, general_info)?);
    }

    let sprite_count: usize = chunk.read_usize_count()?;
    let mut sprites: Vec<GMSpriteInstance> = Vec::with_capacity(sprite_count);
    for _ in 0..sprite_count {
        sprites.push(parse_sprite_instance(chunk, strings)?);
    }
    
    let mut sequences: Vec<GMSequenceInstance> = Vec::new();
    let mut nine_slices: Vec<GMSpriteInstance> = Vec::new();
    let mut particle_systems: Vec<GMParticleSystemInstance> = Vec::new();
    let mut text_items: Vec<GMTextItemInstance> = Vec::new();

    if general_info.is_version_at_least(2, 3, 0, 0) {
        let sequence_count: usize = chunk.read_usize_count()?;
        sequences.reserve(sequence_count);
        for _ in 0..sequence_count {
            sequences.push(parse_sequence_instance(chunk, strings)?);
        }
    }
    
    if !general_info.is_version_at_least(2, 3, 2, 0) {
        let nine_slice_count: usize = chunk.read_usize_count()?;
        nine_slices.reserve(nine_slice_count);
        for _ in 0..nine_slice_count {
            nine_slices.push(parse_sprite_instance(chunk, strings)?);
        }
    }
    
    if general_info.is_version_at_least(2023, 2, 0, 0) {   // {~~} non LTS
        let particle_system_count: usize = chunk.read_usize_count()?;
        particle_systems.reserve(particle_system_count);
        for _ in 0..particle_system_count {
            particle_systems.push(parse_particle_system_instance(chunk, strings)?);
        }
    }
    
    if general_info.is_version_at_least(2024, 6, 0, 0) {
        let text_item_count: usize = chunk.read_usize_count()?;
        text_items.reserve(text_item_count);
        for _ in 0..text_item_count {
            text_items.push(parse_text_item_instance(chunk, strings)?);
        }
    }
    
    // TODO pointers are very scuffed; issue will probably arise
    
    Ok(GMRoomLayerDataAssets {
        legacy_tiles,
        sprites,
        sequences,
        nine_slices,
        particle_systems,
        text_items,
    })
}

fn parse_sprite_instance(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMSpriteInstance, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let sprite: GMRef<GMSprite> = GMRef::new(chunk.read_usize_count()?);
    let x: i32 = chunk.read_i32()?;
    let y: i32 = chunk.read_i32()?;
    let scale_x: f32 = chunk.read_f32()?;
    let scale_y: f32 = chunk.read_f32()?;
    let color: u32 = chunk.read_u32()?;
    let animation_speed: f32 = chunk.read_f32()?;
    let animation_speed_type: u32 = chunk.read_u32()?;
    let animation_speed_type: GMAnimSpeedType = animation_speed_type.try_into()
        .map_err(|_| format!("Invalid Animation Speed Type {0} (0x{0:08X}) while parsing Room Assets Layer Sprite Instance", animation_speed_type))?;
    let frame_index: f32 = chunk.read_f32()?;
    let rotation: f32 = chunk.read_f32()?;
    
    Ok(GMSpriteInstance {
        name,
        sprite,
        x,
        y,
        scale_x,
        scale_y,
        color,
        animation_speed,
        animation_speed_type,
        frame_index,
        rotation,
    })
}


fn parse_sequence_instance(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMSequenceInstance, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let sequence: GMRef<GMSequence> = GMRef::new(chunk.read_usize_count()?);
    let x: i32 = chunk.read_i32()?;
    let y: i32 = chunk.read_i32()?;
    let scale_x: f32 = chunk.read_f32()?;
    let scale_y: f32 = chunk.read_f32()?;
    let color: u32 = chunk.read_u32()?;
    let animation_speed: f32 = chunk.read_f32()?;
    let animation_speed_type: u32 = chunk.read_u32()?;
    let animation_speed_type: GMAnimSpeedType = animation_speed_type.try_into()
        .map_err(|_| format!("Invalid Animation Speed Type {0} (0x{0:08X}) while parsing Room Assets Layer Sprite Instance", animation_speed_type))?;
    let frame_index: f32 = chunk.read_f32()?;
    let rotation: f32 = chunk.read_f32()?;

    Ok(GMSequenceInstance {
        name,
        sequence,
        x,
        y,
        scale_x,
        scale_y,
        color,
        animation_speed,
        animation_speed_type,
        frame_index,
        rotation,
    })
}

fn parse_particle_system_instance(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMParticleSystemInstance, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let particle_system: GMRef<GMParticleSystem> = GMRef::new(chunk.read_usize_count()?);
    let x: i32 = chunk.read_i32()?;
    let y: i32 = chunk.read_i32()?;
    let scale_x: f32 = chunk.read_f32()?;
    let scale_y: f32 = chunk.read_f32()?;
    let color: u32 = chunk.read_u32()?;
    let rotation: f32 = chunk.read_f32()?;

    Ok(GMParticleSystemInstance {
        name,
        particle_system,
        x,
        y,
        scale_x,
        scale_y,
        color,
        rotation,
    })
}

fn parse_text_item_instance(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMTextItemInstance, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let x: i32 = chunk.read_i32()?;
    let y: i32 = chunk.read_i32()?;
    let font: GMRef<GMFont> = GMRef::new(chunk.read_usize_count()?);
    let scale_x: f32 = chunk.read_f32()?;
    let scale_y: f32 = chunk.read_f32()?;
    let rotation: f32 = chunk.read_f32()?;
    let color: u32 = chunk.read_u32()?;
    let origin_x: f32 = chunk.read_f32()?;
    let origin_y: f32 = chunk.read_f32()?;
    let text: GMRef<String> = chunk.read_gm_string(strings)?;
    let alignment: i32 = chunk.read_i32()?;
    let character_spacing: f32 = chunk.read_f32()?;
    let line_spacing: f32 = chunk.read_f32()?;
    let frame_width: f32 = chunk.read_f32()?;
    let frame_height: f32 = chunk.read_f32()?;
    let wrap: bool = chunk.read_bool32()?;

    Ok(GMTextItemInstance {
        name,
        x,
        y,
        font,
        scale_x,
        scale_y,
        rotation,
        color,
        origin_x,
        origin_y,
        text,
        alignment,
        character_spacing,
        line_spacing,
        frame_width,
        frame_height,
        wrap,
    })
}

fn parse_layer_effect(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMRoomLayerDataEffect, String> {
    // {~~} dont serialize if >= 2022.1
    let effect_type: GMRef<String> = chunk.read_gm_string(strings)?;
    
    let properties_count: usize = chunk.read_usize_count()?;
    let mut properties: Vec<GMRoomLayerEffectProperty> = Vec::with_capacity(properties_count);
    
    for _ in 0..properties_count {
        let kind: i32 = chunk.read_i32()?;
        let kind: GMRoomLayerEffectPropertyType = kind.try_into()
            .map_err(|_| format!("Invalid Property Type {0} (0x{0:08X}) while parsing Room Effect Layers", kind))?;
        
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let value: GMRef<String> = chunk.read_gm_string(strings)?;
        properties.push(GMRoomLayerEffectProperty {
            kind,
            name,
            value,
        });
    }
    
    Ok(GMRoomLayerDataEffect {
        effect_type,
        properties,
    })
}

