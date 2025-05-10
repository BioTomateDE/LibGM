use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::deserialize::rooms::{GMRoom, GMRoomBackground, GMRoomFlags, GMRoomGameObject, GMRoomLayer, GMRoomLayerType, GMRoomTile, GMRoomTileTexture, GMRoomView};
use crate::deserialize::sequence::GMSequence;
use crate::export_mod::export::{edit_field, edit_field_option, flag_field, GModData, ModUnorderedRef};
use crate::export_mod::sequences::ModSequence;
use crate::export_mod::unordered_list::{export_changes_unordered_list, AModUnorderedListChanges, GModUnorderedListChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRoom {
    pub name: Option<ModUnorderedRef>,
    pub caption: Option<ModUnorderedRef>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub speed: Option<u32>,
    pub persistent: Option<bool>,
    pub background_color: Option<u32>,
    pub draw_background_color: Option<bool>,
    pub creation_code: Option<ModUnorderedRef>,
    pub flags: Option<ModRoomFlags>,
    pub backgrounds: Option<AModUnorderedListChanges<ModRoomBackground>>,
    pub views: Option<AModUnorderedListChanges<ModRoomView>>,
    pub game_objects: Option<AModUnorderedListChanges<ModRoomGameObject>>,
    pub tiles: Option<AModUnorderedListChanges<ModRoomTile>>,
    pub world: Option<bool>,
    pub top: Option<u32>,
    pub left: Option<u32>,
    pub right: Option<u32>,
    pub bottom: Option<u32>,
    pub gravity_x: Option<f32>,
    pub gravity_y: Option<f32>,
    pub meters_per_pixel: Option<f32>,
    pub layers: Option<AModUnorderedListChanges<ModRoomLayer>>,
    pub sequences: Option<AModUnorderedListChanges<ModSequence>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRoomFlags {
    pub enable_views: Option<bool>,
    pub show_color: Option<bool>,
    pub dont_clear_display_buffer: Option<bool>,
    pub is_gms2: Option<bool>,
    pub is_gms2_3: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRoomView {
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
pub struct ModRoomBackground {
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
pub struct ModRoomTile {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModRoomTileTexture {
    Sprite(ModUnorderedRef),     // GMSprite
    Background(ModUnorderedRef), // GMBackground
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRoomLayer {
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
pub struct ModRoomGameObject {
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


impl GModData<'_, '_> {
    pub fn convert_room_additions(&self, gm_rooms: &Vec<GMRoom>) -> Result<Vec<ModRoom>, String> {
        let mut mod_rooms = Vec::with_capacity(gm_rooms.len());

        for room in gm_rooms {
            mod_rooms.push(ModRoom {
                name: Some(self.resolve_string_ref(&room.name)?),
                caption: Some(self.resolve_string_ref(&room.caption)?),
                width: Some(room.width),
                height: Some(room.height),
                speed: Some(room.speed),
                persistent: Some(room.persistent),
                background_color: Some(room.background_color),
                draw_background_color: Some(room.draw_background_color),
                creation_code: if let Some(ref code) = room.creation_code { Some(self.resolve_code_ref(code)?) } else { None },
                flags: Some(self.convert_room_flags_additions(&room.flags)),
                backgrounds: Some(AModUnorderedListChanges {additions: self.convert_room_backgrounds_additions(&room.backgrounds)?, edits: HashMap::new()}),
                views: Some(AModUnorderedListChanges {additions: self.convert_room_views_additions(&room.views)?, edits: HashMap::new()}),
                game_objects: Some(AModUnorderedListChanges {additions: self.convert_room_game_objects_additions(&room.game_objects)?, edits: HashMap::new()}),
                tiles: Some(AModUnorderedListChanges {additions: self.convert_room_tiles_additions(&room.tiles)?, edits: HashMap::new()}),
                world: Some(room.world),
                top: Some(room.top),
                left: Some(room.left),
                right: Some(room.right),
                bottom: Some(room.bottom),
                gravity_x: Some(room.gravity_x),
                gravity_y: Some(room.gravity_y),
                meters_per_pixel: Some(room.meters_per_pixel),
                layers: if let Some(ref layers) = room.layers { Some(AModUnorderedListChanges {
                    additions: self.convert_room_layers_additions(&layers)?,
                    edits: HashMap::new()
                })} else { None },
                sequences: Default::default(),  // {~~} TODO implement sequence
                // sequences: self.convert_room_sequences(&room.sequences)?,
            });
        }

        Ok(mod_rooms)
    }

    pub fn convert_rooms(&self, changes: &GModUnorderedListChanges<GMRoom>) -> Result<AModUnorderedListChanges<ModRoom>, String> {
        let additions = self.convert_room_additions(&changes.additions)?;
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

            edits.insert(*index, ModRoom {
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

        Ok(AModUnorderedListChanges { additions, edits })
    }

    pub fn convert_room_flags_additions(&self, flags: &GMRoomFlags) -> ModRoomFlags {
        ModRoomFlags {
            enable_views: Some(flags.enable_views),
            show_color: Some(flags.show_color),
            dont_clear_display_buffer: Some(flags.dont_clear_display_buffer),
            is_gms2: Some(flags.is_gms2),
            is_gms2_3: Some(flags.is_gms2_3),
        }
    }

    pub fn convert_room_flags(&self, original: &GMRoomFlags, modified: &GMRoomFlags) -> ModRoomFlags {
        ModRoomFlags {
            enable_views: flag_field(original.enable_views, modified.enable_views),
            show_color: flag_field(original.show_color, modified.show_color),
            dont_clear_display_buffer: flag_field(original.dont_clear_display_buffer, modified.dont_clear_display_buffer),
            is_gms2: flag_field(original.is_gms2, modified.is_gms2),
            is_gms2_3: flag_field(original.is_gms2_3, modified.is_gms2_3),
        }
    }

    pub fn convert_room_backgrounds_additions(&self, backgrounds: &Vec<GMRoomBackground>) -> Result<Vec<ModRoomBackground>, String> {
        let mut mod_backgrounds = Vec::with_capacity(backgrounds.len());

        for bg in backgrounds {
            mod_backgrounds.push(ModRoomBackground {
                enabled: Some(bg.enabled),
                foreground: Some(bg.foreground),
                background_definition: match &bg.background_definition {
                    Some(def) => Some(self.resolve_background_ref(def)?),
                    None => None,
                },
                x: Some(bg.x),
                y: Some(bg.y),
                tile_x: Some(bg.tile_x),
                tile_y: Some(bg.tile_y),
                speed_x: Some(bg.speed_x),
                speed_y: Some(bg.speed_y),
                stretch: Some(bg.stretch),
            });
        }

        Ok(mod_backgrounds)
    }

    pub fn convert_room_backgrounds(&self, changes: GModUnorderedListChanges<GMRoomBackground>) -> Result<AModUnorderedListChanges<ModRoomBackground>, String> {
        let additions: Vec<ModRoomBackground> = self.convert_room_backgrounds_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModRoomBackground> = HashMap::new();

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

            edits.insert(*index, ModRoomBackground {
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

        Ok(AModUnorderedListChanges { additions, edits })
    }

    pub fn convert_room_views_additions(&self, views: &Vec<GMRoomView>) -> Result<Vec<ModRoomView>, String> {
        let mut mod_views: Vec<ModRoomView> = Vec::with_capacity(views.len());

        for view in views {
            mod_views.push(ModRoomView {
                enabled: Some(view.enabled),
                view_x: Some(view.view_x),
                view_y: Some(view.view_y),
                view_width: Some(view.view_width),
                view_height: Some(view.view_height),
                port_x: Some(view.port_x),
                port_y: Some(view.port_y),
                port_width: Some(view.port_width),
                port_height: Some(view.port_height),
                border_x: Some(view.border_x),
                border_y: Some(view.border_y),
                speed_x: Some(view.speed_x),
                speed_y: Some(view.speed_y),
                object: if let Some(ref obj) = view.object { Some(self.resolve_game_object_ref(&obj)?) } else { None },
            });
        }

        Ok(mod_views)
    }

    pub fn convert_room_views(&self, changes: &GModUnorderedListChanges<GMRoomView>) -> Result<AModUnorderedListChanges<ModRoomView>, String> {
        let additions: Vec<ModRoomView> = self.convert_room_views_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModRoomView> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, ModRoomView {
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

        Ok(AModUnorderedListChanges { additions, edits })
    }

    pub fn convert_room_tiles_additions(&self, tiles: &Vec<GMRoomTile>) -> Result<Vec<ModRoomTile>, String> {
        let mut mod_tiles: Vec<ModRoomTile> = Vec::with_capacity(tiles.len());

        for tile in tiles {
            mod_tiles.push(ModRoomTile {
                x: Some(tile.x),
                y: Some(tile.y),
                texture: Some(match &tile.texture {
                    GMRoomTileTexture::Sprite(sprite) => ModRoomTileTexture::Sprite(self.resolve_sprite_ref(sprite)?),
                    GMRoomTileTexture::Background(background) => ModRoomTileTexture::Background(self.resolve_background_ref(background)?),
                }),
                source_x: Some(tile.source_x),
                source_y: Some(tile.source_y),
                width: Some(tile.width),
                height: Some(tile.height),
                tile_depth: Some(tile.tile_depth),
                instance_id: Some(tile.instance_id),
                scale_x: Some(tile.scale_x),
                scale_y: Some(tile.scale_y),
                color: Some(tile.color),
            });
        }

        Ok(mod_tiles)
    }

    pub fn convert_room_tiles(&self, changes: &GModUnorderedListChanges<GMRoomTile>) -> Result<AModUnorderedListChanges<ModRoomTile>, String> {
        let additions: Vec<ModRoomTile> = self.convert_room_tiles_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModRoomTile> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, ModRoomTile {
                x: edit_field(&original.x, &modified.x),
                y: edit_field(&original.y, &modified.y),
                texture: edit_field(&match &original.texture {
                    GMRoomTileTexture::Sprite(sprite) => ModRoomTileTexture::Sprite(self.resolve_sprite_ref(&sprite)?),
                    GMRoomTileTexture::Background(background) => ModRoomTileTexture::Background(self.resolve_background_ref(&background)?),
                }, &match &modified.texture {
                    GMRoomTileTexture::Sprite(sprite) => ModRoomTileTexture::Sprite(self.resolve_sprite_ref(&sprite)?),
                    GMRoomTileTexture::Background(background) => ModRoomTileTexture::Background(self.resolve_background_ref(&background)?),
                }),
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

        Ok(AModUnorderedListChanges { additions, edits })
    }

    pub fn convert_room_layers_additions(&self, layers: &Vec<GMRoomLayer>) -> Result<Vec<ModRoomLayer>, String> {
        let mut mod_layers: Vec<ModRoomLayer> = Vec::with_capacity(layers.len());

        for layer in layers {
            mod_layers.push(ModRoomLayer {
                layer_name: Some(self.resolve_string_ref(&layer.layer_name)?),
                layer_id: Some(layer.layer_id),
                layer_type: Some(layer.layer_type.clone()),
                layer_depth: Some(layer.layer_depth),
                x_offset: Some(layer.x_offset),
                y_offset: Some(layer.y_offset),
                horizontal_speed: Some(layer.horizontal_speed),
                vertical_speed: Some(layer.vertical_speed),
                is_visible: Some(layer.is_visible),
            });
        }

        Ok(mod_layers)
    }

    pub fn convert_room_layers(&self, changes: &GModUnorderedListChanges<GMRoomLayer>) -> Result<AModUnorderedListChanges<ModRoomLayer>, String> {
        let additions: Vec<ModRoomLayer> = self.convert_room_layers_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModRoomLayer> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, ModRoomLayer {
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

        Ok(AModUnorderedListChanges { additions, edits })
    }


    pub fn convert_room_game_objects_additions(&self, objects: &Vec<GMRoomGameObject>) -> Result<Vec<ModRoomGameObject>, String> {
        let mut mod_objects = Vec::with_capacity(objects.len());

        for obj in objects {
            mod_objects.push(ModRoomGameObject {
                x: Some(obj.x),
                y: Some(obj.y),
                object_definition: Some(self.resolve_game_object_ref(&obj.object_definition)?),
                instance_id: Some(obj.instance_id),
                creation_code: if let Some(ref code) = obj.creation_code { Some(self.resolve_code_ref(&code)?) } else { None },
                scale_x: Some(obj.scale_x),
                scale_y: Some(obj.scale_y),
                image_speed: obj.image_speed,
                image_index: obj.image_index,
                color: Some(obj.color),
                rotation: Some(obj.rotation),
                pre_create_code: if let Some(ref code) = obj.pre_create_code { Some(self.resolve_code_ref(&code)?) } else { None },
            });
        }

        Ok(mod_objects)
    }

    pub fn convert_room_game_objects(&self, changes: &GModUnorderedListChanges<GMRoomGameObject>) -> Result<AModUnorderedListChanges<ModRoomGameObject>, String> {
        let additions: Vec<ModRoomGameObject> = self.convert_room_game_objects_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModRoomGameObject> = HashMap::new();

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

            edits.insert(*index, ModRoomGameObject {
                x: edit_field(&original.x, &modified.x),
                y: edit_field(&original.y, &modified.y),
                object_definition: edit_field(&self.resolve_game_object_ref(&original.object_definition)?, &self.resolve_game_object_ref(&modified.object_definition)?),
                instance_id: edit_field(&original.instance_id, &modified.instance_id),
                creation_code: edit_field_option(&orig_creation_code, &mod_creation_code).clone(),
                scale_x: edit_field(&original.scale_x, &modified.scale_x),
                scale_y: edit_field(&original.scale_y, &modified.scale_y),
                image_speed: *edit_field_option(&original.image_speed, &modified.image_speed),
                image_index: *edit_field_option(&original.image_index, &modified.image_index),
                color: edit_field(&original.color, &modified.color),
                rotation: edit_field(&original.rotation, &modified.rotation),
                pre_create_code: edit_field_option(&orig_pre_create_code, &mod_pre_create_code).clone(),
            });
        }

        Ok(AModUnorderedListChanges { additions, edits })
    }


}


