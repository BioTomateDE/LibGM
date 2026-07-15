// SPDX-License-Identifier: GPL-3.0-only
mod background;
mod flags;
mod game_object;
pub mod layer;
pub mod tile;
mod view;

pub use self::background::RoomBackground;
pub use self::flags::RoomFlags;
pub use self::game_object::RoomGameObject;
pub use self::layer::RoomLayer;
pub use self::tile::RoomTile;
pub use self::view::RoomView;
use crate::gml::Code;
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::sequence::Sequence;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rooms {
    pub elems: Vec<Option<Room>>,
    pub exists: bool,
}

gm_named_list_chunk!(ROOM, Rooms, Room, nullable);

impl GMElement for Rooms {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<Room>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)] // Need explicit layout so memory addresses for gm pointers don't collide
pub struct Room {
    pub name: GMRef<String>,
    pub caption: GMRef<String>,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub background_color: u32,
    pub draw_background_color: bool,
    pub creation_code: GMRef<Code>,
    pub flags: RoomFlags,
    pub backgrounds: Vec<RoomBackground>,
    pub views: Vec<RoomView>,
    pub game_objects: Vec<RoomGameObject>,
    pub tiles: Vec<RoomTile>,
    pub instance_creation_order: Vec<InstanceID>,
    pub world: bool,
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub meters_per_pixel: f32,
    pub layers: Vec<RoomLayer>,
    pub sequences: Vec<Sequence>,
}

impl GMElement for Room {
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
        let creation_code: GMRef<Code> = reader.read_resource_by_id()?;
        let flags = reader.read_u32()?;
        let flags =
            RoomFlags::from_bits(flags).ok_or_else(|| format!("Invalid Room Flags {flags:08X}"))?;

        let backgrounds_ptr = reader.read_u32()?;
        let views_ptr = reader.read_u32()?;
        let game_objects_ptr = reader.read_u32()?;
        let tiles_ptr = reader.read_u32()?;
        let instances_ptr = reader
            .deserialize_if_version(GMVersion::GM2024_13)?
            .unwrap_or(0);

        let world = reader.read_bool32()?;
        let top = reader.read_u32()?;
        let left = reader.read_u32()?;
        let right = reader.read_u32()?;
        let bottom = reader.read_u32()?;
        let gravity_x = reader.read_f32()?;
        let gravity_y = reader.read_f32()?;
        let meters_per_pixel = reader.read_f32()?;

        let layers_ptr: u32 = reader
            .deserialize_if_version(GMVersion::Studio2)?
            .unwrap_or(0);
        let sequences_ptr: u32 = reader
            .deserialize_if_version(GMVersion::Studio2_3)?
            .unwrap_or(0);

        reader.assert_pos(backgrounds_ptr, "Room Backgrounds")?;
        let backgrounds: Vec<RoomBackground> = reader.read_pointer_list()?;

        reader.assert_pos(views_ptr, "Room Views")?;
        let views: Vec<RoomView> = reader.read_pointer_list()?;

        reader.assert_pos(game_objects_ptr, "Room Game Objects")?;
        let game_objects: Vec<RoomGameObject> = reader.read_pointer_list()?;

        reader.assert_pos(tiles_ptr, "Room Tiles")?;
        let tiles: Vec<RoomTile> = reader.read_pointer_list()?;

        let instance_creation_order: Vec<InstanceID> = if reader.version >= GMVersion::GM2024_13 {
            reader.assert_pos(instances_ptr, "Room Instance Creation Order IDs")?;
            reader.read_simple_list()?
        } else {
            Vec::new()
        };

        let layers: Vec<RoomLayer> = if reader.version >= GMVersion::Studio2 {
            reader.assert_pos(layers_ptr, "Room Layers")?;
            reader.read_pointer_list()?
        } else {
            Vec::new()
        };

        let sequences: Vec<Sequence> = if reader.version >= GMVersion::Studio2_3 {
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
            instance_creation_order,
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

        if builder.version() >= GMVersion::GM2024_13 {
            builder.write_pointer(&self.instance_creation_order);
        }

        builder.write_bool32(self.world);
        builder.write_u32(self.top);
        builder.write_u32(self.left);
        builder.write_u32(self.right);
        builder.write_u32(self.bottom);
        builder.write_f32(self.gravity_x);
        builder.write_f32(self.gravity_y);
        builder.write_f32(self.meters_per_pixel);

        if builder.version() >= GMVersion::Studio2 {
            builder.write_pointer(&self.layers);
        }

        if builder.version() >= GMVersion::Studio2_3 {
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

        if builder.version() >= GMVersion::GM2024_13 {
            builder.resolve_pointer(&self.instance_creation_order)?;
            builder.write_simple_list(&self.instance_creation_order)?;
        }

        if builder.version() >= GMVersion::Studio2 {
            builder.resolve_pointer(&self.layers)?;
            builder.write_pointer_list(&self.layers)?;
        }

        if builder.version() >= GMVersion::Studio2_3 {
            builder.resolve_pointer(&self.sequences)?;
            builder.write_pointer_list(&self.sequences)?;
        }

        Ok(())
    }
}

impl Default for Room {
    fn default() -> Self {
        let view = RoomView {
            enabled: false,
            view_x: 0,
            view_y: 0,
            view_width: 640,
            view_height: 480,
            port_x: 0,
            port_y: 0,
            port_width: 640,
            port_height: 480,
            border_x: 32,
            border_y: 32,
            speed_x: -1,
            speed_y: -1,
            object: GMRef::none(),
        };

        Self {
            name: GMRef::none(), // "room{idx}"
            caption: GMRef::none(),
            width: 640,
            height: 480,
            speed: 0,
            persistent: false,
            background_color: 0xFF000000,
            draw_background_color: false,
            creation_code: Default::default(),
            flags: RoomFlags::ENABLE_VIEWS
                | RoomFlags::CLEAR_VIEW_BACKGROUND
                | RoomFlags::GM2
                | RoomFlags::GM2_3,
            backgrounds: Vec::new(),
            views: vec![view; 8],
            game_objects: Vec::new(),
            tiles: Vec::new(),
            instance_creation_order: Vec::new(),
            world: false,
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            gravity_x: 0.0,
            gravity_y: 0.0,
            meters_per_pixel: 0.1,
            layers: Vec::new(),
            sequences: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceID(pub i32);

impl GMElement for InstanceID {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let id = reader.read_i32()?;
        Ok(Self(id))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.0);
        Ok(())
    }
}
