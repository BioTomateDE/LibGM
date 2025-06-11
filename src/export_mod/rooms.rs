use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::deserialize::rooms::{GMRoom, GMRoomBackground, GMRoomFlags, GMRoomGameObject, GMRoomLayer, GMRoomLayerType, GMRoomTile, GMRoomTileTexture, GMRoomView};
use crate::deserialize::sequence::GMSequence;
use crate::export_mod::export::{edit_field, edit_field_option, flag_field, GModData, ModUnorderedRef};
use crate::export_mod::sequences::{AddSequence, EditSequence};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList, GModUnorderedListChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoom {
    pub name: ModUnorderedRef,
    pub caption: ModUnorderedRef,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub background_color: u32,
    pub draw_background_color: bool,
    pub creation_code: Option<ModUnorderedRef>,
    pub flags: GMRoomFlags,
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
    pub layers: Option<Vec<AddRoomLayer>>,
    pub sequences: Option<Vec<AddSequence>>,
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
    pub object: Option<ModUnorderedRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomBackground {
    pub enabled: bool,
    pub foreground: bool,
    pub background_definition: Option<ModUnorderedRef>, // GMBackground
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
    pub instance_id: u32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRoomLayer {
    pub layer_name: ModUnorderedRef, // String
    pub layer_id: u32,
    pub layer_type: GMRoomLayerType,
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
    pub object_definition: ModUnorderedRef, // GMGameObject
    pub instance_id: u32,
    pub creation_code: Option<ModUnorderedRef>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub image_speed: Option<f32>,
    pub image_index: Option<usize>,
    pub color: u32,
    pub rotation: f32,
    pub pre_create_code: Option<ModUnorderedRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoom {
    pub name: Option<ModUnorderedRef>,
    pub caption: Option<ModUnorderedRef>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub speed: Option<u32>,
    pub persistent: Option<bool>,
    pub background_color: Option<u32>,
    pub draw_background_color: Option<bool>,
    pub creation_code: Option<ModUnorderedRef>,
    pub flags: Option<EditRoomFlags>,
    pub backgrounds: Option<EditUnorderedList<AddRoomBackground, EditRoomBackground>>,
    pub views: Option<EditUnorderedList<AddRoomView, EditRoomView>>,
    pub game_objects: Option<EditUnorderedList<AddRoomGameObject, EditRoomGameObject>>,
    pub tiles: Option<EditUnorderedList<AddRoomTile, EditRoomTile>>,    // TODO no more Option 
    pub world: Option<bool>,
    pub top: Option<u32>,
    pub left: Option<u32>,
    pub right: Option<u32>,
    pub bottom: Option<u32>,
    pub gravity_x: Option<f32>,
    pub gravity_y: Option<f32>,
    pub meters_per_pixel: Option<f32>,
    pub layers: Option<EditUnorderedList<AddRoomLayer, EditRoomLayer>>,
    pub sequences: Option<EditUnorderedList<AddSequence, EditSequence>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomFlags {
    pub enable_views: Option<bool>,
    pub show_color: Option<bool>,
    pub dont_clear_display_buffer: Option<bool>,
    pub is_gms2: Option<bool>,
    pub is_gms2_3: Option<bool>,
}
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
    pub object: Option<ModUnorderedRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomBackground {
    pub enabled: Option<bool>,
    pub foreground: Option<bool>,
    pub background_definition: Option<ModUnorderedRef>, // GMBackground
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub tile_x: Option<i32>,
    pub tile_y: Option<i32>,
    pub speed_x: Option<i32>,
    pub speed_y: Option<i32>,
    pub stretch: Option<bool>,
}
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
    pub instance_id: Option<u32>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub color: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomLayer {
    pub layer_name: Option<ModUnorderedRef>, // String
    pub layer_id: Option<u32>,
    pub layer_type: Option<GMRoomLayerType>,
    pub layer_depth: Option<i32>,
    pub x_offset: Option<f32>,
    pub y_offset: Option<f32>,
    pub horizontal_speed: Option<f32>,
    pub vertical_speed: Option<f32>,
    pub is_visible: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRoomGameObject {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub object_definition: Option<ModUnorderedRef>, // GMGameObject
    pub instance_id: Option<u32>,
    pub creation_code: Option<ModUnorderedRef>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub image_speed: Option<f32>,
    pub image_index: Option<usize>,
    pub color: Option<u32>,
    pub rotation: Option<f32>,
    pub pre_create_code: Option<ModUnorderedRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModRoomTileTexture {
    Sprite(ModUnorderedRef),     // GMSprite
    Background(ModUnorderedRef), // GMBackground
}


impl GModData<'_, '_> {
    pub fn convert_room_additions(&self, rooms: &[GMRoom]) -> Result<Vec<AddRoom>, String> {
        rooms.iter().map(|i| {
            Ok(AddRoom {
                name: self.resolve_string_ref(&i.name)?,
                caption: self.resolve_string_ref(&i.caption)?,
                width: i.width,
                height: i.height,
                speed: i.speed,
                persistent: i.persistent,
                background_color: i.background_color,
                draw_background_color: i.draw_background_color,
                creation_code: self.resolve_optional_code_ref(&i.creation_code)?,
                flags: i.flags.clone(),
                backgrounds: self.convert_room_backgrounds_additions(&i.backgrounds)?,
                views: self.convert_room_views_additions(&i.views)?,
                game_objects: self.convert_room_game_objects_additions(&i.game_objects)?,
                tiles: self.convert_room_tiles_additions(&i.tiles)?,
                world: i.world,
                top: i.top,
                left: i.left,
                right: i.right,
                bottom: i.bottom,
                gravity_x: i.gravity_x,
                gravity_y: i.gravity_y,
                meters_per_pixel: i.meters_per_pixel,
                layers: if let Some(ref layers) = i.layers {
                    Some(self.convert_room_layers_additions(layers)?)
                } else {
                    None
                },
                sequences: if let Some(ref sequences) = i.sequences {
                    Some(self.convert_sequences_additions(sequences)?)
                } else {
                    None
                },
            })
        }).collect()
    }

    pub fn convert_rooms(&self, changes: &GModUnorderedListChanges<GMRoom>) -> Result<EditUnorderedList<AddRoom, EditRoom>, String> {
        let additions = self.convert_room_additions(changes.additions)?;
        let mut edits = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            static EMPTY_VEC_LAYERS: Vec<GMRoomLayer> = Vec::new();
            static EMPTY_VEC_SEQUENCES: Vec<GMSequence> = Vec::new();

            let orig_creation_code: Option<ModUnorderedRef> = if let Some(code) = &original.creation_code {
                Some(self.resolve_code_ref(code)?)
            } else { None };
            let mod_creation_code: Option<ModUnorderedRef> = if let Some(code) = &modified.creation_code {
                Some(self.resolve_code_ref(code)?)
            } else { None };

            edits.insert(*index, EditRoom {
                name: edit_field(&self.resolve_string_ref(&original.name)?, &self.resolve_string_ref(&modified.name)?),
                caption: edit_field(&self.resolve_string_ref(&original.caption)?, &self.resolve_string_ref(&modified.caption)?),
                width: edit_field(&original.width, &modified.width),
                height: edit_field(&original.height, &modified.height),
                speed: edit_field(&original.speed, &modified.speed),
                persistent: edit_field(&original.persistent, &modified.persistent),
                background_color: edit_field(&original.background_color, &modified.background_color),
                draw_background_color: edit_field(&original.draw_background_color, &modified.draw_background_color),
                creation_code: edit_field_option(&orig_creation_code, &mod_creation_code).clone(),
                flags: Some(self.convert_room_flags(&original.flags, &modified.flags)),
                backgrounds: Some(self.convert_room_backgrounds(export_changes_unordered_list(&original.backgrounds, &modified.backgrounds)?)?),
                views: Some(self.convert_room_views(&export_changes_unordered_list(&original.views, &modified.views)?)?),
                game_objects: Some(self.convert_room_game_objects(&export_changes_unordered_list(&original.game_objects, &modified.game_objects)?)?),
                tiles: Some(self.convert_room_tiles(&export_changes_unordered_list(&original.tiles, &modified.tiles)?)?),
                world: edit_field(&original.world, &modified.world),
                top: edit_field(&original.top, &modified.top),
                left: edit_field(&original.left, &modified.left),
                right: edit_field(&original.right, &modified.right),
                bottom: edit_field(&original.bottom, &modified.bottom),
                gravity_x: edit_field(&original.gravity_x, &modified.gravity_x),
                gravity_y: edit_field(&original.gravity_y, &modified.gravity_y),
                meters_per_pixel: edit_field(&original.meters_per_pixel, &modified.meters_per_pixel),
                layers: Some(self.convert_room_layers(&export_changes_unordered_list(
                    &original.layers.as_ref().unwrap_or_else(|| &EMPTY_VEC_LAYERS),
                    &modified.layers.as_ref().unwrap_or_else(|| &EMPTY_VEC_LAYERS),
                )?)?),
                sequences: Some(self.convert_sequences(export_changes_unordered_list(
                    &original.sequences.as_ref().unwrap_or_else(|| &EMPTY_VEC_SEQUENCES),
                    &modified.sequences.as_ref().unwrap_or_else(|| &EMPTY_VEC_SEQUENCES),
                )?)?),
            });
        }

        Ok(EditUnorderedList { additions, edits })
    }

    pub fn convert_room_flags(&self, original: &GMRoomFlags, modified: &GMRoomFlags) -> EditRoomFlags {
        EditRoomFlags {
            enable_views: flag_field(original.enable_views, modified.enable_views),
            show_color: flag_field(original.show_color, modified.show_color),
            dont_clear_display_buffer: flag_field(original.dont_clear_display_buffer, modified.dont_clear_display_buffer),
            is_gms2: flag_field(original.is_gms2, modified.is_gms2),
            is_gms2_3: flag_field(original.is_gms2_3, modified.is_gms2_3),
        }
    }

    pub fn convert_room_backgrounds_additions(&self, backgrounds: &[GMRoomBackground]) -> Result<Vec<AddRoomBackground>, String> {
        let mut mod_backgrounds = Vec::with_capacity(backgrounds.len());

        for i in backgrounds {
            mod_backgrounds.push(AddRoomBackground {
                enabled: i.enabled,
                foreground: i.foreground,
                background_definition: self.resolve_optional_background_ref(&i.background_definition)?,
                x: i.x,
                y: i.y,
                tile_x: i.tile_x,
                tile_y: i.tile_y,
                speed_x: i.speed_x,
                speed_y: i.speed_y,
                stretch: i.stretch,
            });
        }

        Ok(mod_backgrounds)
    }

    pub fn convert_room_backgrounds(&self, changes: GModUnorderedListChanges<GMRoomBackground>) -> Result<EditUnorderedList<AddRoomBackground, EditRoomBackground>, String> {
        let additions: Vec<AddRoomBackground> = self.convert_room_backgrounds_additions(changes.additions)?;
        let mut edits: HashMap<usize, EditRoomBackground> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            let resolved_original_background = original
                .background_definition
                .as_ref()
                .map(|def| self.resolve_background_ref(def))
                .transpose()?; // This gives you an Option<T>, not Result<Option<T>>

            let resolved_modified_background = modified
                .background_definition
                .as_ref()
                .map(|def| self.resolve_background_ref(def))
                .transpose()?;

            edits.insert(*index, EditRoomBackground {
                enabled: edit_field(&original.enabled, &modified.enabled),
                foreground: edit_field(&original.foreground, &modified.foreground),
                background_definition: edit_field_option(&resolved_original_background, &resolved_modified_background).clone(),
                x: edit_field(&original.x, &modified.x),
                y: edit_field(&original.y, &modified.y),
                tile_x: edit_field(&original.tile_x, &modified.tile_x),
                tile_y: edit_field(&original.tile_y, &modified.tile_y),
                speed_x: edit_field(&original.speed_x, &modified.speed_x),
                speed_y: edit_field(&original.speed_y, &modified.speed_y),
                stretch: edit_field(&original.stretch, &modified.stretch),
            });
        }

        Ok(EditUnorderedList { additions, edits })
    }

    pub fn convert_room_views_additions(&self, views: &[GMRoomView]) -> Result<Vec<AddRoomView>, String> {
        let mut mod_views = Vec::with_capacity(views.len());

        for i in views {
            mod_views.push(AddRoomView {
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
                object: self.resolve_optional_game_object_ref(&i.object)?,
            });
        }

        Ok(mod_views)
    }

    pub fn convert_room_views(&self, changes: &GModUnorderedListChanges<GMRoomView>) -> Result<EditUnorderedList<AddRoomView, EditRoomView>, String> {
        let additions = self.convert_room_views_additions(&changes.additions)?;
        let mut edits: HashMap<usize, EditRoomView> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, EditRoomView {
                enabled: edit_field(&original.enabled, &modified.enabled),
                view_x: edit_field(&original.view_x, &modified.view_x),
                view_y: edit_field(&original.view_y, &modified.view_y),
                view_width: edit_field(&original.view_width, &modified.view_width),
                view_height: edit_field(&original.view_height, &modified.view_height),
                port_x: edit_field(&original.port_x, &modified.port_x),
                port_y: edit_field(&original.port_y, &modified.port_y),
                port_width: edit_field(&original.port_width, &modified.port_width),
                port_height: edit_field(&original.port_height, &modified.port_height),
                border_x: edit_field(&original.border_x, &modified.border_x),
                border_y: edit_field(&original.border_y, &modified.border_y),
                speed_x: edit_field(&original.speed_x, &modified.speed_x),
                speed_y: edit_field(&original.speed_y, &modified.speed_y),
                object: None,   // TODO
                // object: edit_field(&self.resolve_game_object_ref(&original.object)?, &self.resolve_game_object_ref(&modified.object)?),
            });
        }

        Ok(EditUnorderedList { additions, edits })
    }

    pub fn convert_room_tiles_additions(&self, tiles: &[GMRoomTile]) -> Result<Vec<AddRoomTile>, String> {
        tiles.iter().map(|i| {
            Ok(AddRoomTile {
                x: i.x,
                y: i.y,
                texture: self.convert_room_tile_texture(&i.texture)?,
                source_x: i.source_x,
                source_y: i.source_y,
                width: i.width,
                height: i.height,
                tile_depth: i.tile_depth,
                instance_id: i.instance_id,
                scale_x: i.scale_x,
                scale_y: i.scale_y,
                color: i.color,
            })
        }).collect()
    }

    pub fn convert_room_tiles(&self, changes: &GModUnorderedListChanges<GMRoomTile>) -> Result<EditUnorderedList<AddRoomTile, EditRoomTile>, String> {
        let additions = self.convert_room_tiles_additions(&changes.additions)?;
        let mut edits: HashMap<usize, EditRoomTile> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, EditRoomTile {
                x: edit_field(&original.x, &modified.x),
                y: edit_field(&original.y, &modified.y),
                texture: edit_field(&self.convert_room_tile_texture(&original.texture)?, &self.convert_room_tile_texture(&modified.texture)?),
                source_x: edit_field(&original.source_x, &modified.source_x),
                source_y: edit_field(&original.source_y, &modified.source_y),
                width: edit_field(&original.width, &modified.width),
                height: edit_field(&original.height, &modified.height),
                tile_depth: edit_field(&original.tile_depth, &modified.tile_depth),
                instance_id: edit_field(&original.instance_id, &modified.instance_id),
                scale_x: edit_field(&original.scale_x, &modified.scale_x),
                scale_y: edit_field(&original.scale_y, &modified.scale_y),
                color: edit_field(&original.color, &modified.color),
            });
        }

        Ok(EditUnorderedList { additions, edits })
    }
    
    fn convert_room_tile_texture(&self, room_tile_texture: &GMRoomTileTexture) -> Result<ModRoomTileTexture, String> { 
        match room_tile_texture {
            GMRoomTileTexture::Sprite(sprite) => Ok(ModRoomTileTexture::Sprite(self.resolve_sprite_ref(&sprite)?)),
            GMRoomTileTexture::Background(background) => Ok(ModRoomTileTexture::Background(self.resolve_background_ref(&background)?)),
        }
    }

    pub fn convert_room_layers_additions(&self, layers: &[GMRoomLayer]) -> Result<Vec<AddRoomLayer>, String> {
        layers.iter().map(|i| {
            Ok(AddRoomLayer {
                layer_name: self.resolve_string_ref(&i.layer_name)?,
                layer_id: i.layer_id,
                layer_type: i.layer_type,
                layer_depth: i.layer_depth,
                x_offset: i.x_offset,
                y_offset: i.y_offset,
                horizontal_speed: i.horizontal_speed,
                vertical_speed: i.vertical_speed,
                is_visible: i.is_visible,
            })
        }).collect()
    }

    pub fn convert_room_layers(&self, changes: &GModUnorderedListChanges<GMRoomLayer>) -> Result<EditUnorderedList<AddRoomLayer, EditRoomLayer>, String> {
        let additions = self.convert_room_layers_additions(&changes.additions)?;
        let mut edits: HashMap<usize, EditRoomLayer> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, EditRoomLayer {
                layer_name: edit_field(&self.resolve_string_ref(&original.layer_name)?, &self.resolve_string_ref(&modified.layer_name)?),
                layer_id: edit_field(&original.layer_id, &modified.layer_id),
                layer_type: edit_field(&original.layer_type, &modified.layer_type),
                layer_depth: edit_field(&original.layer_depth, &modified.layer_depth),
                x_offset: edit_field(&original.x_offset, &modified.x_offset),
                y_offset: edit_field(&original.y_offset, &modified.y_offset),
                horizontal_speed: edit_field(&original.horizontal_speed, &modified.horizontal_speed),
                vertical_speed: edit_field(&original.vertical_speed, &modified.vertical_speed),
                is_visible: edit_field(&original.is_visible, &modified.is_visible),
            });
        }

        Ok(EditUnorderedList { additions, edits })
    }


    pub fn convert_room_game_objects_additions(&self, objects: &[GMRoomGameObject]) -> Result<Vec<AddRoomGameObject>, String> {
        let mut mod_objects = Vec::with_capacity(objects.len());
        for i in objects {
            mod_objects.push(AddRoomGameObject {
                x: i.x,
                y: i.y,
                object_definition: self.resolve_game_object_ref(&i.object_definition)?,
                instance_id: i.instance_id,
                creation_code: self.resolve_optional_code_ref(&i.creation_code)?,
                scale_x: i.scale_x,
                scale_y: i.scale_y,
                image_speed: i.image_speed,
                image_index: i.image_index,
                color: i.color,
                rotation: i.rotation,
                pre_create_code: self.resolve_optional_code_ref(&i.pre_create_code)?,
            });
        }
        Ok(mod_objects)
    }

    pub fn convert_room_game_objects(&self, changes: &GModUnorderedListChanges<GMRoomGameObject>) -> Result<EditUnorderedList<AddRoomGameObject, EditRoomGameObject>, String> {
        let additions = self.convert_room_game_objects_additions(&changes.additions)?;
        let mut edits: HashMap<usize, EditRoomGameObject> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            let orig_creation_code: Option<ModUnorderedRef> = if let Some(code) = &original.creation_code {
                Some(self.resolve_code_ref(code)?)
            } else { None };
            let mod_creation_code: Option<ModUnorderedRef> = if let Some(code) = &modified.creation_code {
                Some(self.resolve_code_ref(code)?)
            } else { None };

            let orig_pre_create_code: Option<ModUnorderedRef> = if let Some(code) = &original.pre_create_code {
                Some(self.resolve_code_ref(code)?)
            } else { None };
            let mod_pre_create_code: Option<ModUnorderedRef> = if let Some(code) = &modified.pre_create_code {
                Some(self.resolve_code_ref(code)?)
            } else { None };

            edits.insert(*index, EditRoomGameObject {
                x: edit_field(&original.x, &modified.x),
                y: edit_field(&original.y, &modified.y),
                object_definition: edit_field(&self.resolve_game_object_ref(&original.object_definition)?, &self.resolve_game_object_ref(&modified.object_definition)?),
                instance_id: edit_field(&original.instance_id, &modified.instance_id),
                creation_code: edit_field_option(&orig_creation_code, &mod_creation_code).clone(),
                scale_x: edit_field(&original.scale_x, &modified.scale_x),
                scale_y: edit_field(&original.scale_y, &modified.scale_y),
                image_speed: edit_field_option(&original.image_speed, &modified.image_speed),
                image_index: edit_field_option(&original.image_index, &modified.image_index),
                color: edit_field(&original.color, &modified.color),
                rotation: edit_field(&original.rotation, &modified.rotation),
                pre_create_code: edit_field_option(&orig_pre_create_code, &mod_pre_create_code).clone(),
            });
        }

        Ok(EditUnorderedList { additions, edits })
    }


}


