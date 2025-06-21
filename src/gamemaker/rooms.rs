use crate::gm_deserialize::{vec_with_capacity, DataReader, GMChunkElement, GMElement, GMRef};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::backgrounds::GMBackground;
use crate::gamemaker::code::GMCode;
use crate::gamemaker::fonts::GMFont;
use crate::gamemaker::game_objects::{GMGameObject};
use crate::gamemaker::particles::GMParticleSystem;
use crate::gamemaker::sequence::{GMAnimSpeedType, GMSequence};
use crate::gamemaker::sprites::GMSprite;


#[derive(Debug, Clone)]
pub struct GMRooms {
    pub rooms: Vec<GMRoom>,
    pub exists: bool,
}
impl GMChunkElement for GMRooms {
    fn empty() -> Self {
        Self { rooms: vec![], exists: false }
    }
}
impl GMElement for GMRooms {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let rooms: Vec<GMRoom> = reader.read_pointer_list()?;
        Ok(Self { rooms, exists: true })
    }
}


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
    pub layers: Vec<GMRoomLayer>,
    pub sequences: Vec<GMSequence>,
}
impl GMElement for GMRoom {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let caption: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let width: u32 = reader.read_u32()?;
        let height: u32 = reader.read_u32()?;
        let speed: u32 = reader.read_u32()?;
        let persistent: bool = reader.read_bool32()?;
        let background_color: u32 = reader.read_u32()? | 0xFF000000;   // make alpha 255 (background color doesn't have transparency)
        let draw_background_color: bool = reader.read_bool32()?;
        let creation_code: Option<GMRef<GMCode>> = reader.read_resource_by_id_option()?;
        let flags = GMRoomFlags::deserialize(reader)?;
        let backgrounds: Vec<GMRoomBackground> = reader.read_pointer_list()?;
        let views: Vec<GMRoomView> = reader.read_pointer_list()?;
        let game_objects: Vec<GMRoomGameObject> = reader.read_pointer_list()?;
        let tiles: Vec<GMRoomTile> = reader.read_pointer_list()?;
        let world: bool = reader.read_bool32()?;
        let top: u32 = reader.read_u32()?;
        let left: u32 = reader.read_u32()?;
        let right: u32 = reader.read_u32()?;
        let bottom: u32 = reader.read_u32()?;
        let gravity_x: f32 = reader.read_f32()?;
        let gravity_y: f32 = reader.read_f32()?;
        let meters_per_pixel: f32 = reader.read_f32()?;
        let mut layers: Vec<GMRoomLayer> = Vec::new();
        let mut sequences: Vec<GMSequence> = Vec::new();
        if reader.general_info.is_version_at_least((2, 0, 0, 0)) {
            layers = reader.read_pointer_list()?;
        }
        if reader.general_info.is_version_at_least((2, 3, 0, 0)) {
            sequences = reader.read_pointer_list()?;
        }

        Ok(GMRoom {
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
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomFlags {
    pub enable_views: bool,                 // views are enabled
    pub show_color: bool,                   // meaning uncertain
    pub dont_clear_display_buffer: bool,    // don't clear display buffer
    pub is_gms2: bool,                      // room was made in GameMaker: Studio 2
    pub is_gms2_3: bool,                    // room was made in GameMaker: Studio 2.3
}
impl GMElement for GMRoomFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let raw = reader.read_u32()?;
        Ok(GMRoomFlags {
            enable_views: 0 != raw & 1,
            show_color: 0 != raw & 2,
            dont_clear_display_buffer: 0 != raw & 4,
            is_gms2: 0 != raw & 131072,
            is_gms2_3: 0 != raw & 65536,
        })
    }
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
impl GMElement for GMRoomView {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let enabled: bool = reader.read_bool32()?;
        let view_x: i32 = reader.read_i32()?;
        let view_y: i32 = reader.read_i32()?;
        let view_width: i32 = reader.read_i32()?;
        let view_height: i32 = reader.read_i32()?;
        let port_x: i32 = reader.read_i32()?;
        let port_y: i32 = reader.read_i32()?;
        let port_width: i32 = reader.read_i32()?;
        let port_height: i32 = reader.read_i32()?;
        let border_x: u32 = reader.read_u32()?;
        let border_y: u32 = reader.read_u32()?;
        let speed_x: i32 = reader.read_i32()?;
        let speed_y: i32 = reader.read_i32()?;
        let object: Option<GMRef<GMGameObject>> = reader.read_resource_by_id_option()?;

        Ok(GMRoomView {
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
        })
    }
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
impl GMElement for GMRoomBackground {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let enabled: bool = reader.read_bool32()?;
        let foreground: bool = reader.read_bool32()?;
        let background_definition: Option<GMRef<GMBackground>> = reader.read_resource_by_id_option()?;
        let x: i32 = reader.read_i32()?;
        let y: i32 = reader.read_i32()?;
        let tile_x: i32 = reader.read_i32()?;    // idk if this should be an int instead of a bool
        let tile_y: i32 = reader.read_i32()?;    // ^
        let speed_x: i32 = reader.read_i32()?;
        let speed_y: i32 = reader.read_i32()?;
        let stretch: bool = reader.read_bool32()?;

        Ok(GMRoomBackground {
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
impl GMElement for GMRoomTile {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let x: i32 = reader.read_i32()?;
        let y: i32 = reader.read_i32()?;
        let texture: GMRoomTileTexture = if reader.general_info.is_version_at_least((2, 0, 0, 0)) {
            GMRoomTileTexture::Sprite(reader.read_resource_by_id_option()?)
        } else {
            GMRoomTileTexture::Background(reader.read_resource_by_id_option()?)
        };
        let source_x: u32 = reader.read_u32()?;
        let source_y: u32 = reader.read_u32()?;
        let width: u32 = reader.read_u32()?;
        let height: u32 = reader.read_u32()?;
        let tile_depth: i32 = reader.read_i32()?;
        let instance_id: u32 = reader.read_u32()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let color: u32 = reader.read_u32()?;

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
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMRoomTileTexture {
    Sprite(Option<GMRef<GMSprite>>),
    Background(Option<GMRef<GMBackground>>),
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
impl GMElement for GMRoomLayer {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let layer_name: GMRef<String> = reader.read_gm_string()?;
        let layer_id: u32 = reader.read_u32()?;
        let layer_type: u32 = reader.read_u32()?;
        let layer_type: GMRoomLayerType = layer_type.try_into().map_err(|_| format!(
            "Invalid Room Layer Type 0x{:04X} at position {} while parsing Room",
            layer_type, reader.cur_pos,
        ))?;
        let layer_depth: i32 = reader.read_i32()?;
        let x_offset: f32 = reader.read_f32()?;
        let y_offset: f32 = reader.read_f32()?;
        let horizontal_speed: f32 = reader.read_f32()?;
        let vertical_speed: f32 = reader.read_f32()?;
        let is_visible: bool = reader.read_bool32()?;

        let data_2022_1: Option<GMRoomLayer2022_1> = if reader.general_info.is_version_at_least((2022, 1, 0, 0)) {
            // TODO auto detect gm version; since gm2 the version in gen8 is stuck on 2.0
            Some(GMRoomLayer2022_1::deserialize(reader)?)
        } else {
            None
        };

        let data: GMRoomLayerData = match layer_type {
            GMRoomLayerType::Path | GMRoomLayerType::Path2 => GMRoomLayerData::None,
            GMRoomLayerType::Background => GMRoomLayerData::Background(GMRoomLayerDataBackground::deserialize(reader)?),
            GMRoomLayerType::Instances => GMRoomLayerData::Instances(GMRoomLayerDataInstances::deserialize(reader)?),
            GMRoomLayerType::Assets => GMRoomLayerData::Assets(GMRoomLayerDataAssets::deserialize(reader)?),
            GMRoomLayerType::Tiles => GMRoomLayerData::Tiles(GMRoomLayerDataTiles::deserialize(reader)?),
            GMRoomLayerType::Effect => GMRoomLayerData::Effect(GMRoomLayerDataEffect::deserialize(reader)?),
        };

        Ok(GMRoomLayer {
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
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayer2022_1 {
    pub effect_enabled: bool,
    pub effect_type: GMRef<String>, 
    pub effect_properties: Vec<GMRoomLayerEffectProperty>,
}
impl GMElement for GMRoomLayer2022_1 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let effect_enabled: bool = reader.read_bool32()?;
        let effect_type: GMRef<String> = reader.read_gm_string()?;
        let effect_properties: Vec<GMRoomLayerEffectProperty> = reader.read_simple_list()?;
        Ok(GMRoomLayer2022_1 { effect_enabled, effect_type, effect_properties })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerEffectProperty {
    pub kind: GMRoomLayerEffectPropertyType,
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}
impl GMElement for GMRoomLayerEffectProperty {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let kind: i32 = reader.read_i32()?;
        let kind: GMRoomLayerEffectPropertyType = kind.try_into().map_err(|_| format!("Invalid Room Layer Effect Property {kind} (0x{kind:08X})"))?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(GMRoomLayerEffectProperty { kind, name, value })
    }
}


#[derive(Debug, Clone, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMRoomLayerEffectPropertyType {
    Real = 0,
    Color = 1,
    Sampler = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMRoomLayerType {
    /// unused?
    Path = 0,
    Background = 1,
    Instances = 2,
    Assets = 3,
    Tiles = 4,
    Effect = 6,
    /// introduced in 2024.13
    Path2 = 7,
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
    pub instances: Vec<u32>,
    // pub instances: Vec<GMRoomGameObject>,        // TODO {~~}
}

impl GMElement for GMRoomLayerDataInstances {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let count: usize = reader.read_usize()?;
        let mut instances: Vec<u32> = vec_with_capacity(count)?;
        // {~~} check scuffed conditions for false positive idk

        for i in 0..count {
            let instance_id: u32 = reader.read_u32()?;
            // if instance_id == 0 {
            //     log::warn!("Skipping Zero Instance ID #{i} of Instance Layer of Room \"{room_name}\"");
            //     continue
            // }
            // let room_game_object: GMRoomGameObject = get_room_game_object_by_instance_id(room_game_objects, instance_id)
            //     .ok_or_else(|| format!("Nonexistent room game objects are not supported yet (has instance id {instance_id})"))?;
            instances.push(instance_id);
        }

        Ok(GMRoomLayerDataInstances { instances })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataTiles {
    pub background: GMRef<GMBackground>,
    /// Flattened 2D Array. Access using `tile_data[row * width * col]`.
    pub tile_data: Vec<u32>,
    pub width: usize,
    pub height: usize,
}
impl GMElement for GMRoomLayerDataTiles {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let background: GMRef<GMBackground> = reader.read_resource_by_id()?;
        let width: usize = reader.read_usize()?;
        let height: usize = reader.read_usize()?;
        if reader.general_info.is_version_at_least((2024, 2, 0, 0)) {
            return Err("Compressed tile data (GM >= 2024.2) is not supported yet. Report this error to GitHub".to_string())     // TODO
        }
        let mut tile_data: Vec<u32> = vec_with_capacity(width * height)?;
        for _y in 0..height {
            for _x in 0..width {
                tile_data.push(reader.read_u32()?);
            }
        }

        Ok(GMRoomLayerDataTiles {
            background,
            tile_data,
            width,
            height,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataBackground {
    pub visible: bool,
    pub foreground: bool,
    pub sprite: Option<GMRef<GMSprite>>,
    pub tiled_horizontally: bool,
    pub tiled_vertically: bool,
    pub stretch: bool,
    pub color: u32,
    pub first_frame: f32,
    pub animation_speed: f32,
    pub animation_speed_type: GMAnimSpeedType,
}
impl GMElement for GMRoomLayerDataBackground {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let visible: bool = reader.read_bool32()?;
        let foreground: bool = reader.read_bool32()?;
        let sprite: Option<GMRef<GMSprite>> = reader.read_resource_by_id_option()?;
        let tiled_horizontally: bool = reader.read_bool32()?;
        let tiled_vertically: bool = reader.read_bool32()?;
        let stretch: bool = reader.read_bool32()?;
        let color: u32 = reader.read_u32()?;
        let first_frame: f32 = reader.read_f32()?;
        let animation_speed: f32 = reader.read_f32()?;
        let animation_speed_type: u32 = reader.read_u32()?;
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
impl GMElement for GMRoomLayerDataAssets {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let legacy_tiles_pointer: usize = reader.read_usize()?;
        let sprites_pointer: usize = reader.read_usize()?;
        let mut sequences_pointer: Option<usize> = None;
        let mut nine_slices_pointer: Option<usize> = None;
        let mut particle_systems_pointer: Option<usize> = None;
        let mut text_items_pointer: Option<usize> = None;

        if reader.general_info.is_version_at_least((2, 3, 0, 0)) {
            sequences_pointer = Some(reader.read_usize()?);
            if !reader.general_info.is_version_at_least((2, 3, 2, 0)) {
                nine_slices_pointer = Some(reader.read_usize()?);
            }
        }
        if reader.general_info.is_version_at_least((2023, 2, 0, 0)) {   // {~~} non LTS
            particle_systems_pointer = Some(reader.read_usize()?);
        }
        if reader.general_info.is_version_at_least((2024, 6, 0, 0)) {
            text_items_pointer = Some(reader.read_usize()?);
        }

        reader.cur_pos = legacy_tiles_pointer;
        let legacy_tiles: Vec<GMRoomTile> = reader.read_pointer_list()?;

        reader.cur_pos = sprites_pointer;
        let sprites: Vec<GMSpriteInstance> = reader.read_pointer_list()?;

        let mut sequences: Vec<GMSequenceInstance> = Vec::new();
        let mut nine_slices: Vec<GMSpriteInstance> = Vec::new();
        let mut particle_systems: Vec<GMParticleSystemInstance> = Vec::new();
        let mut text_items: Vec<GMTextItemInstance> = Vec::new();

        if reader.general_info.is_version_at_least((2, 3, 0, 0)) {
            reader.cur_pos = sequences_pointer.unwrap();
            sequences = reader.read_pointer_list()?;

            if !reader.general_info.is_version_at_least((2, 3, 2, 0)) {
                reader.cur_pos = nine_slices_pointer.unwrap();
                nine_slices = reader.read_pointer_list()?;
            }
        }

        if reader.general_info.is_version_at_least((2023, 2, 0, 0)) {   // {~~} non LTS
            reader.cur_pos = particle_systems_pointer.unwrap();
            particle_systems = reader.read_pointer_list()?;
        }

        if reader.general_info.is_version_at_least((2024, 6, 0, 0)) {
            reader.cur_pos = text_items_pointer.unwrap();
            text_items = reader.read_pointer_list()?;
        }

        Ok(GMRoomLayerDataAssets {
            legacy_tiles,
            sprites,
            sequences,
            nine_slices,
            particle_systems,
            text_items,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataEffect {
    pub effect_type: GMRef<String>,
    pub properties: Vec<GMRoomLayerEffectProperty>,
}
impl GMElement for GMRoomLayerDataEffect {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        // {~~} dont serialize_old if >= 2022.1
        let effect_type: GMRef<String> = reader.read_gm_string()?;
        let properties: Vec<GMRoomLayerEffectProperty> = reader.read_simple_list()?;
        Ok(GMRoomLayerDataEffect { effect_type, properties })
    }
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
impl GMElement for GMSpriteInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let sprite: GMRef<GMSprite> = reader.read_resource_by_id()?;
        let x: i32 = reader.read_i32()?;
        let y: i32 = reader.read_i32()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let color: u32 = reader.read_u32()?;
        let animation_speed: f32 = reader.read_f32()?;
        let animation_speed_type: u32 = reader.read_u32()?;
        let animation_speed_type: GMAnimSpeedType = animation_speed_type.try_into()
            .map_err(|_| format!("Invalid Animation Speed Type {0} (0x{0:08X}) while parsing Room Assets Layer Sprite Instance", animation_speed_type))?;
        let frame_index: f32 = reader.read_f32()?;
        let rotation: f32 = reader.read_f32()?;

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
impl GMElement for GMSequenceInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let sequence: GMRef<GMSequence> = reader.read_resource_by_id()?;
        let x: i32 = reader.read_i32()?;
        let y: i32 = reader.read_i32()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let color: u32 = reader.read_u32()?;
        let animation_speed: f32 = reader.read_f32()?;
        let animation_speed_type: u32 = reader.read_u32()?;
        let animation_speed_type: GMAnimSpeedType = animation_speed_type.try_into()
            .map_err(|_| format!("Invalid Animation Speed Type {0} (0x{0:08X}) while parsing Room Assets Layer Sprite Instance", animation_speed_type))?;
        let frame_index: f32 = reader.read_f32()?;
        let rotation: f32 = reader.read_f32()?;

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
impl GMElement for GMParticleSystemInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let particle_system: GMRef<GMParticleSystem> = reader.read_resource_by_id()?;
        let x: i32 = reader.read_i32()?;
        let y: i32 = reader.read_i32()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let color: u32 = reader.read_u32()?;
        let rotation: f32 = reader.read_f32()?;

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
impl GMElement for GMTextItemInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let x: i32 = reader.read_i32()?;
        let y: i32 = reader.read_i32()?;
        let font: GMRef<GMFont> = reader.read_resource_by_id()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let rotation: f32 = reader.read_f32()?;
        let color: u32 = reader.read_u32()?;
        let origin_x: f32 = reader.read_f32()?;
        let origin_y: f32 = reader.read_f32()?;
        let text: GMRef<String> = reader.read_gm_string()?;
        let alignment: i32 = reader.read_i32()?;
        let character_spacing: f32 = reader.read_f32()?;
        let line_spacing: f32 = reader.read_f32()?;
        let frame_width: f32 = reader.read_f32()?;
        let frame_height: f32 = reader.read_f32()?;
        let wrap: bool = reader.read_bool32()?;

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
}
impl GMElement for GMRoomGameObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let x: i32 = reader.read_i32()?;
        let y: i32 = reader.read_i32()?;
        let object_definition: GMRef<GMGameObject> = reader.read_resource_by_id()?;
        let instance_id: u32 = reader.read_u32()?;
        let creation_code: Option<GMRef<GMCode>> = reader.read_resource_by_id_option()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let mut image_speed: Option<f32> = None;
        let mut image_index: Option<usize> = None;
        if reader.general_info.is_version_at_least((2, 2, 2, 302)) {
            image_speed = Some(reader.read_f32()?);
            image_index = Some(reader.read_usize()?);
        }
        let color: u32 = reader.read_u32()?;
        let rotation: f32 = reader.read_f32()?;

        // [From UndertaleModTool] "is that dependent on bytecode or something else?"
        let pre_create_code: Option<GMRef<GMCode>> = if reader.general_info.bytecode_version <= 15 {
            None
        } else {
            if reader.read_i32()? == -1 { None } else { creation_code }
        };

        Ok(GMRoomGameObject {
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
        })
    }
}

