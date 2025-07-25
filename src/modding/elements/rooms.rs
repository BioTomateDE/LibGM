use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::rooms::{GMRoomBackground, GMRoomFlags, GMRoomGameObject, GMRoomLayer, GMRoomLayerType, GMRoomTile, GMRoomTileTexture, GMRoomView};
use crate::modding::export::{convert_additions, edit_field, edit_field_convert, edit_field_convert_option, edit_field_option, flag_field, ModExporter, ModRef};
use crate::modding::elements::sequences::{AddSequence, EditSequence};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoom {
    pub name: ModRef,
    pub caption: Option<ModRef>,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub background_color: u32,
    pub draw_background_color: bool,
    pub creation_code: Option<ModRef>,
    pub flags: AddRoomFlags,
    pub backgrounds: Vec<AddRoomBackground>,
    pub views: Vec<AddRoomView>,
    pub game_objects: Vec<AddRoomGameObject>,
    pub tiles: Vec<AddRoomTile>,
    pub world: bool,
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub meters_per_pixel: f32,
    pub layers: Vec<AddRoomLayer>,
    pub sequences: Vec<AddSequence>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomView {
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
    pub object: Option<ModRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomBackground {
    pub enabled: bool,
    pub foreground: bool,
    pub background_definition: Option<ModRef>, // GMBackground
    pub x: i32,
    pub y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub stretch: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomTile {
    pub x: i32,
    pub y: i32,
    pub texture: ModRoomTileTexture,
    pub source_x: u32,
    pub source_y: u32,
    pub width: u32,
    pub height: u32,
    pub tile_depth: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomLayer {
    pub layer_name: ModRef, // String
    pub layer_id: u32,
    pub layer_type: ModRoomLayerType,
    pub layer_depth: i32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
    pub is_visible: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomGameObject {
    pub x: i32,
    pub y: i32,
    pub object_definition: ModRef, // GMGameObject
    pub creation_code: Option<ModRef>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub image_speed: Option<f32>,
    pub image_index: Option<usize>,
    pub color: u32,
    pub rotation: f32,
    pub pre_create_code: Option<ModRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomFlags {
    pub enable_views: bool,
    pub dont_clear_display_buffer: bool,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoom {
    pub name: Option<ModRef>,       // string ref
    pub caption: Option<ModRef>,    // string ref
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub speed: Option<u32>,
    pub persistent: Option<bool>,
    pub background_color: Option<u32>,
    pub draw_background_color: Option<bool>,
    pub creation_code: Option<Option<ModRef>>,  // code ref
    pub flags: EditRoomFlags,
    pub backgrounds: Vec<DataChange<AddRoomBackground, EditRoomBackground>>,
    pub views: Vec<DataChange<AddRoomView, EditRoomView>>,
    pub game_objects: Vec<DataChange<AddRoomGameObject, EditRoomGameObject>>,
    pub tiles: Vec<DataChange<AddRoomTile, EditRoomTile>>,
    pub world: Option<bool>,
    pub top: Option<u32>,
    pub left: Option<u32>,
    pub right: Option<u32>,
    pub bottom: Option<u32>,
    pub gravity_x: Option<f32>,
    pub gravity_y: Option<f32>,
    pub meters_per_pixel: Option<f32>,
    pub layers: Vec<DataChange<AddRoomLayer, EditRoomLayer>>,
    pub sequences: Vec<DataChange<AddSequence, EditSequence>>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomFlags {
    pub enable_views: Option<bool>,
    pub dont_clear_display_buffer: Option<bool>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomView {
    pub enabled: Option<bool>,
    pub view_x: Option<i32>,
    pub view_y: Option<i32>,
    pub view_width: Option<i32>,
    pub view_height: Option<i32>,
    pub port_x: Option<i32>,
    pub port_y: Option<i32>,
    pub port_width: Option<i32>,
    pub port_height: Option<i32>,
    pub border_x: Option<u32>,
    pub border_y: Option<u32>,
    pub speed_x: Option<i32>,
    pub speed_y: Option<i32>,
    pub object: Option<Option<ModRef>>,   // GMGameObject
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomBackground {
    pub enabled: Option<bool>,
    pub foreground: Option<bool>,
    pub background_definition: Option<Option<ModRef>>, // GMBackground
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub tile_x: Option<i32>,
    pub tile_y: Option<i32>,
    pub speed_x: Option<i32>,
    pub speed_y: Option<i32>,
    pub stretch: Option<bool>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomTile {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub texture: Option<ModRoomTileTexture>,
    pub source_x: Option<u32>,
    pub source_y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tile_depth: Option<i32>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub color: Option<u32>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomLayer {
    pub layer_name: Option<ModRef>, // String
    pub layer_id: Option<u32>,
    pub layer_type: Option<ModRoomLayerType>,
    pub layer_depth: Option<i32>,
    pub x_offset: Option<f32>,
    pub y_offset: Option<f32>,
    pub horizontal_speed: Option<f32>,
    pub vertical_speed: Option<f32>,
    pub is_visible: Option<bool>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomGameObject {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub object_definition: Option<ModRef>, // GMGameObject
    pub creation_code: Option<Option<ModRef>>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub image_speed: Option<f32>,
    pub image_index: Option<usize>,
    pub color: Option<u32>,
    pub rotation: Option<f32>,
    pub pre_create_code: Option<Option<ModRef>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModRoomTileTexture {
    Sprite(Option<ModRef>),     // GMSprite
    Background(Option<ModRef>), // GMBackground
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(u32)]
pub enum ModRoomLayerType {
    Background = 1,
    Instances = 2,
    Assets = 3,
    Tiles = 4,
    Effect = 6,
    Path2 = 7,
}


impl ModExporter<'_, '_> {
    pub fn export_rooms(&self) -> Result<Vec<DataChange<AddRoom, EditRoom>>, String> {
        export_changes_ordered_list(
            &self.original_data.rooms.rooms,
            &self.modified_data.rooms.rooms,
            |i| Ok(AddRoom {
                name: self.convert_string_ref(&i.name)?,
                caption: self.convert_string_ref_opt(&i.caption)?,
                width: i.width,
                height: i.height,
                speed: i.speed,
                persistent: i.persistent,
                background_color: i.background_color,
                draw_background_color: i.draw_background_color,
                creation_code: self.convert_code_ref_opt(&i.creation_code)?,
                flags: AddRoomFlags {
                    enable_views: i.flags.enable_views,
                    dont_clear_display_buffer: i.flags.dont_clear_display_buffer,
                },
                backgrounds: convert_additions(&i.backgrounds, |r| self.add_room_background(r))?,
                views: convert_additions(&i.views, |view| self.add_room_view(view))?,
                game_objects: convert_additions(&i.game_objects, |obj| self.add_room_game_object(obj))?,
                tiles: convert_additions(&i.tiles, |tile| self.add_room_tile(tile))?,
                world: i.world,
                top: i.top,
                left: i.left,
                right: i.right,
                bottom: i.bottom,
                gravity_x: i.gravity_x,
                gravity_y: i.gravity_y,
                meters_per_pixel: i.meters_per_pixel,
                layers: convert_additions(&i.layers, |layer| self.add_room_layer(layer))?,
                sequences: convert_additions(&i.sequences, |sequence| self.add_sequence(sequence))?,
            }),
            |o, m| Ok(EditRoom {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                caption: edit_field_convert_option(&o.caption, &m.caption, |r| self.convert_string_ref(r))?.flatten(),
                width: edit_field(&o.width, &m.width),
                height: edit_field(&o.height, &m.height),
                speed: edit_field(&o.speed, &m.speed),
                persistent: edit_field(&o.persistent, &m.persistent),
                background_color: edit_field(&o.background_color, &m.background_color),
                draw_background_color: edit_field(&o.draw_background_color, &m.draw_background_color),
                creation_code: edit_field_convert_option(&o.creation_code, &m.creation_code, |r| self.convert_code_ref(r))?,
                flags: edit_room_flags(&o.flags, &m.flags),
                backgrounds: export_changes_ordered_list(
                    &o.backgrounds,
                    &m.backgrounds,
                    |i| self.add_room_background(i),
                    |o, m| self.edit_room_background(o, m),
                )?,
                views: export_changes_ordered_list(
                    &o.views,
                    &m.views,
                    |i| self.add_room_view(i),
                    |o, m| self.edit_room_view(o, m),
                )?,
                game_objects: export_changes_ordered_list(
                    &o.game_objects,
                    &m.game_objects,
                    |i| self.add_room_game_object(i),
                    |o, m| self.edit_room_game_object(o, m),
                )?,
                tiles: export_changes_ordered_list(
                    &o.tiles,
                    &m.tiles,
                    |i| self.add_room_tile(i),
                    |o, m| self.edit_room_tile(o, m),
                )?,
                world: edit_field(&o.world, &m.world),
                top: edit_field(&o.top, &m.top),
                left: edit_field(&o.left, &m.left),
                right: edit_field(&o.right, &m.right),
                bottom: edit_field(&o.bottom, &m.bottom),
                gravity_x: edit_field(&o.gravity_x, &m.gravity_x),
                gravity_y: edit_field(&o.gravity_y, &m.gravity_y),
                meters_per_pixel: edit_field(&o.meters_per_pixel, &m.meters_per_pixel),
                layers: export_changes_ordered_list(
                    &o.layers,
                    &m.layers,
                    |i| self.add_room_layer(i),
                    |o, m| self.edit_room_layer(o, m),
                )?,
                sequences: export_changes_ordered_list(
                    &o.sequences,
                    &m.sequences,
                    |i| self.add_sequence(i),
                    |o, m| self.edit_sequence(o, m),
                )?,
            }),
        )
    }

    fn add_room_background(&self, i: &GMRoomBackground) -> Result<AddRoomBackground, String> {
        Ok(AddRoomBackground {
            enabled: i.enabled,
            foreground: i.foreground,
            background_definition: self.convert_background_ref_opt(&i.background_definition)?,
            x: i.x,
            y: i.y,
            tile_x: i.tile_x,
            tile_y: i.tile_y,
            speed_x: i.speed_x,
            speed_y: i.speed_y,
            stretch: i.stretch,
        })
    }
    
    fn edit_room_background(&self, o: &GMRoomBackground, m: &GMRoomBackground) -> Result<EditRoomBackground, String> {
        Ok(EditRoomBackground {
            enabled: edit_field(&o.enabled, &m.enabled),
            foreground: edit_field(&o.foreground, &m.foreground),
            background_definition: edit_field_convert_option(
                &o.background_definition,
                &m.background_definition,
                |r| self.convert_background_ref(r)
            )?,
            x: edit_field(&o.x, &m.x),
            y: edit_field(&o.y, &m.y),
            tile_x: edit_field(&o.tile_x, &m.tile_x),
            tile_y: edit_field(&o.tile_y, &m.tile_y),
            speed_x: edit_field(&o.speed_x, &m.speed_x),
            speed_y: edit_field(&o.speed_y, &m.speed_y),
            stretch: edit_field(&o.stretch, &m.stretch),
        })
    }
    
    fn add_room_view(&self, i: &GMRoomView) -> Result<AddRoomView, String> {
        Ok(AddRoomView {
            enabled: i.enabled,
            view_x: i.view_x,
            view_y: i.view_y,
            view_width: i.view_width,
            view_height: i.view_height,
            port_x: i.port_x,
            port_y: i.port_y,
            port_width: i.port_width,
            port_height: i.port_height,
            border_x: i.border_x,
            border_y: i.border_y,
            speed_x: i.speed_x,
            speed_y: i.speed_y,
            object: self.convert_game_object_ref_opt(&i.object)?,
        })
    }
    
    fn edit_room_view(&self, o: &GMRoomView, m: &GMRoomView) -> Result<EditRoomView, String> {
        Ok(EditRoomView {
            enabled: edit_field(&o.enabled, &m.enabled),
            view_x: edit_field(&o.view_x, &m.view_x),
            view_y: edit_field(&o.view_y, &m.view_y),
            view_width: edit_field(&o.view_width, &m.view_width),
            view_height: edit_field(&o.view_height, &m.view_height),
            port_x: edit_field(&o.port_x, &m.port_x),
            port_y: edit_field(&o.port_y, &m.port_y),
            port_width: edit_field(&o.port_width, &m.port_width),
            port_height: edit_field(&o.port_height, &m.port_height),
            border_x: edit_field(&o.border_x, &m.border_x),
            border_y: edit_field(&o.border_y, &m.border_y),
            speed_x: edit_field(&o.speed_x, &m.speed_x),
            speed_y: edit_field(&o.speed_y, &m.speed_y),
            object: edit_field_convert_option(&o.object, &m.object, |r| self.convert_game_object_ref(r))?,
        })
    }
    
    fn convert_room_tile_texture(&self, room_tile_texture: &GMRoomTileTexture) -> Result<ModRoomTileTexture, String> {
        match room_tile_texture {
            GMRoomTileTexture::Sprite(sprite) => {
                self.convert_sprite_ref_opt(sprite).map(ModRoomTileTexture::Sprite)
            },
            GMRoomTileTexture::Background(background) => {
                self.convert_background_ref_opt(background).map(ModRoomTileTexture::Background)
            },
        }
    }
    
    fn add_room_tile(&self, i: &GMRoomTile) -> Result<AddRoomTile, String> {
        Ok(AddRoomTile {
            x: i.x,
            y: i.y,
            texture: self.convert_room_tile_texture(&i.texture)?,
            source_x: i.source_x,
            source_y: i.source_y,
            width: i.width,
            height: i.height,
            tile_depth: i.tile_depth,
            scale_x: i.scale_x,
            scale_y: i.scale_y,
            color: i.color,
        })
    }
    
    fn edit_room_tile(&self, o: &GMRoomTile, m: &GMRoomTile) -> Result<EditRoomTile, String> {
        Ok(EditRoomTile {
            x: edit_field(&o.x, &m.x),
            y: edit_field(&o.y, &m.y),
            texture: edit_field(&self.convert_room_tile_texture(&o.texture)?, &self.convert_room_tile_texture(&m.texture)?),
            source_x: edit_field(&o.source_x, &m.source_x),
            source_y: edit_field(&o.source_y, &m.source_y),
            width: edit_field(&o.width, &m.width),
            height: edit_field(&o.height, &m.height),
            tile_depth: edit_field(&o.tile_depth, &m.tile_depth),
            scale_x: edit_field(&o.scale_x, &m.scale_x),
            scale_y: edit_field(&o.scale_y, &m.scale_y),
            color: edit_field(&o.color, &m.color),
        })
    }
    
    fn add_room_layer(&self, i: &GMRoomLayer) -> Result<AddRoomLayer, String> {
        Ok(AddRoomLayer {
            layer_name: self.convert_string_ref(&i.layer_name)?,
            layer_id: i.layer_id,
            layer_type: convert_layer_type(&i.layer_type)?,
            layer_depth: i.layer_depth,
            x_offset: i.x_offset,
            y_offset: i.y_offset,
            horizontal_speed: i.horizontal_speed,
            vertical_speed: i.vertical_speed,
            is_visible: i.is_visible,
        })
    }
    
    fn edit_room_layer(&self, o: &GMRoomLayer, m: &GMRoomLayer) -> Result<EditRoomLayer, String> {
        Ok(EditRoomLayer {
            layer_name: edit_field_convert(&o.layer_name, &m.layer_name, |r| self.convert_string_ref(r))?,
            layer_id: edit_field(&o.layer_id, &m.layer_id),
            layer_type: edit_field(&convert_layer_type(&o.layer_type)?, &convert_layer_type(&m.layer_type)?),
            layer_depth: edit_field(&o.layer_depth, &m.layer_depth),
            x_offset: edit_field(&o.x_offset, &m.x_offset),
            y_offset: edit_field(&o.y_offset, &m.y_offset),
            horizontal_speed: edit_field(&o.horizontal_speed, &m.horizontal_speed),
            vertical_speed: edit_field(&o.vertical_speed, &m.vertical_speed),
            is_visible: edit_field(&o.is_visible, &m.is_visible),
        })
    }
    
    fn add_room_game_object(&self, i: &GMRoomGameObject) -> Result<AddRoomGameObject, String> {
        Ok(AddRoomGameObject {
            x: i.x,
            y: i.y,
            object_definition: self.convert_game_object_ref(&i.object_definition)?,
            creation_code: self.convert_code_ref_opt(&i.creation_code)?,
            scale_x: i.scale_x,
            scale_y: i.scale_y,
            image_speed: i.image_speed,
            image_index: i.image_index,
            color: i.color,
            rotation: i.rotation,
            pre_create_code: self.convert_code_ref_opt(&i.pre_create_code)?,
        })
    }
    
    fn edit_room_game_object(&self, o: &GMRoomGameObject, m: &GMRoomGameObject) -> Result<EditRoomGameObject, String> {
        Ok(EditRoomGameObject {
            x: edit_field(&o.x, &m.x),
            y: edit_field(&o.y, &m.y),
            object_definition: edit_field_convert(&o.object_definition, &m.object_definition, |r| self.convert_game_object_ref(r))?,
            creation_code: edit_field_convert_option(&o.creation_code, &m.creation_code, |r| self.convert_code_ref(r))?,
            scale_x: edit_field(&o.scale_x, &m.scale_x),
            scale_y: edit_field(&o.scale_y, &m.scale_y),
            image_speed: edit_field_option(&o.image_speed, &m.image_speed).flatten(),
            image_index: edit_field_option(&o.image_index, &m.image_index).flatten(),
            color: edit_field(&o.color, &m.color),
            rotation: edit_field(&o.rotation, &m.rotation),
            pre_create_code: edit_field_convert_option(&o.pre_create_code, &m.pre_create_code, |r| self.convert_code_ref(r))?,
        })
    }
}

fn add_room_flags(i: &GMRoomFlags) -> AddRoomFlags {
    AddRoomFlags {
        enable_views: i.enable_views,
        dont_clear_display_buffer: i.dont_clear_display_buffer,
    }
}

fn edit_room_flags(o: &GMRoomFlags, m: &GMRoomFlags) -> EditRoomFlags {
    EditRoomFlags {
        enable_views: flag_field(o.enable_views, m.enable_views),
        dont_clear_display_buffer: flag_field(o.dont_clear_display_buffer, m.dont_clear_display_buffer),
    }
}

fn convert_layer_type(i: &GMRoomLayerType) -> Result<ModRoomLayerType, String> {
    Ok(match i {
        GMRoomLayerType::Path => return Err("Room Layer Type 'Path' is not supported since it seemed to be unused (report this error)".to_string()),
        GMRoomLayerType::Background => ModRoomLayerType::Background,
        GMRoomLayerType::Instances => ModRoomLayerType::Instances,
        GMRoomLayerType::Assets => ModRoomLayerType::Assets,
        GMRoomLayerType::Tiles => ModRoomLayerType::Tiles,
        GMRoomLayerType::Effect => ModRoomLayerType::Effect,
        GMRoomLayerType::Path2 => ModRoomLayerType::Path2,
    })
}

