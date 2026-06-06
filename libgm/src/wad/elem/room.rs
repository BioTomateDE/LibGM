// SPDX-License-Identifier: GPL-3.0-only
mod background;
mod flags;
mod game_object;
pub mod layer;
pub mod tile;
mod view;

pub use background::Background;
pub use flags::Flags;
pub use game_object::GameObject;
pub use layer::Layer;
pub use tile::Tile;
pub use view::View;

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::sequence::GMSequence;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMRooms {
    pub rooms: Vec<Option<GMRoom>>,
    pub exists: bool,
}

gm_named_list_chunk!(ROOM, GMRooms, GMRoom, rooms, nullable);

impl GMElement for GMRooms {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let rooms: Vec<Option<GMRoom>> = reader.read_pointer_list_opt()?;
        Ok(Self { rooms, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list_opt(&self.rooms)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)] // Need explicit layout so memory addresses for gm pointers don't collide
pub struct GMRoom {
    pub name: GMRef<String>,
    pub caption: GMRef<String>,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub background_color: u32,
    pub draw_background_color: bool,
    pub creation_code: GMRef<GMCode>,
    pub flags: Flags,
    pub backgrounds: Vec<Background>,
    pub views: Vec<View>,
    pub game_objects: Vec<GameObject>,
    pub tiles: Vec<Tile>,
    pub instance_creation_order_ids: Vec<i32>,
    pub world: bool,
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub meters_per_pixel: f32,
    pub layers: Vec<Layer>,
    pub sequences: Vec<GMSequence>,
}

impl GMElement for GMRoom {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let caption: GMRef<String> = reader.read_gm_string()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let speed = reader.read_u32()?;
        let persistent = reader.read_bool32()?;

        // make alpha 255 (background color doesn't have transparency)
        let background_color: u32 = reader.read_u32()? ^ 0xFF00_0000;

        let draw_background_color = reader.read_bool32()?;
        let creation_code: GMRef<GMCode> = reader.read_resource_by_id()?;
        let flags = reader.read_u32()?;
        let flags =
            Flags::from_bits(flags).ok_or_else(|| format!("Invalid Room Flags {flags:08X}"))?;

        let backgrounds_ptr = reader.read_u32()?;
        let views_ptr = reader.read_u32()?;
        let game_objects_ptr = reader.read_u32()?;
        let tiles_ptr = reader.read_u32()?;
        let instances_ptr = reader.deserialize_if_gm_version((2024, 13))?.unwrap_or(0);

        let world = reader.read_bool32()?;
        let top = reader.read_u32()?;
        let left = reader.read_u32()?;
        let right = reader.read_u32()?;
        let bottom = reader.read_u32()?;
        let gravity_x = reader.read_f32()?;
        let gravity_y = reader.read_f32()?;
        let meters_per_pixel = reader.read_f32()?;

        let layers_ptr: u32 = reader.deserialize_if_gm_version((2, 0))?.unwrap_or(0);
        let sequences_ptr: u32 = reader.deserialize_if_gm_version((2, 3))?.unwrap_or(0);

        reader.assert_pos(backgrounds_ptr, "Room Backgrounds")?;
        let backgrounds: Vec<Background> = reader.read_pointer_list()?;

        reader.assert_pos(views_ptr, "Room Views")?;
        let views: Vec<View> = reader.read_pointer_list()?;

        reader.assert_pos(game_objects_ptr, "Room Game Objects")?;
        let game_objects: Vec<GameObject> = reader.read_pointer_list()?;

        reader.assert_pos(tiles_ptr, "Room Tiles")?;
        let tiles: Vec<Tile> = reader.read_pointer_list()?;

        let instance_creation_order_ids: Vec<i32> = if reader.general_info.version >= (2024, 13) {
            reader.assert_pos(instances_ptr, "Room Instance Creation Order IDs")?;
            reader.read_simple_list()?
        } else {
            Vec::new()
        };

        let layers: Vec<Layer> = if reader.general_info.version >= (2, 0) {
            reader.assert_pos(layers_ptr, "Room Layers")?;
            reader.read_pointer_list()?
        } else {
            Vec::new()
        };

        let sequences: Vec<GMSequence> = if reader.general_info.version >= (2, 3) {
            reader.assert_pos(sequences_ptr, "Room Sequences")?;
            reader.read_pointer_list()?
        } else {
            Vec::new()
        };

        Ok(Self {
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
            instance_creation_order_ids,
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_gm_string(self.caption)?;
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        builder.write_u32(self.speed);
        builder.write_bool32(self.persistent);

        // remove alpha (background color doesn't have alpha)
        builder.write_u32(self.background_color ^ 0xFF00_0000);

        builder.write_bool32(self.draw_background_color);
        builder.write_resource_id(self.creation_code);
        builder.write_u32(self.flags.bits());
        builder.write_pointer(&self.backgrounds);
        builder.write_pointer(&self.views);
        builder.write_pointer(&self.game_objects);
        builder.write_pointer(&self.tiles);

        if builder.version() >= (2024, 13) {
            builder.write_pointer(&self.instance_creation_order_ids);
        }

        builder.write_bool32(self.world);
        builder.write_u32(self.top);
        builder.write_u32(self.left);
        builder.write_u32(self.right);
        builder.write_u32(self.bottom);
        builder.write_f32(self.gravity_x);
        builder.write_f32(self.gravity_y);
        builder.write_f32(self.meters_per_pixel);

        if builder.version() >= 2 {
            builder.write_pointer(&self.layers);
        }

        if builder.version() >= (2, 3) {
            builder.write_pointer(&self.sequences);
        }

        builder.resolve_pointer(&self.backgrounds)?;
        builder.write_pointer_list(&self.backgrounds)?;
        builder.resolve_pointer(&self.views)?;
        builder.write_pointer_list(&self.views)?;
        builder.resolve_pointer(&self.game_objects)?;
        builder.write_pointer_list(&self.game_objects)?;
        builder.resolve_pointer(&self.tiles)?;
        builder.write_pointer_list(&self.tiles)?;

        if builder.version() >= (2024, 13) {
            builder.resolve_pointer(&self.instance_creation_order_ids)?;
            builder.write_pointer_list(&self.instance_creation_order_ids)?;
        }

        if builder.version() >= 2 {
            builder.resolve_pointer(&self.layers)?;
            builder.write_pointer_list(&self.layers)?;
        }

        if builder.version() >= (2, 3) {
            builder.resolve_pointer(&self.sequences)?;
            builder.write_pointer_list(&self.sequences)?;
        }

        Ok(())
    }
}
