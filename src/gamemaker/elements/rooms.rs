use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::backgrounds::GMBackground;
use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::elements::fonts::GMFont;
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::elements::particles::GMParticleSystem;
use crate::gamemaker::elements::sequence::{GMAnimSpeedType, GMSequence};
use crate::gamemaker::elements::sprites::GMSprite;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::LTSBranch;
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::init::{num_enum_from, vec_with_capacity};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cmp::min;

#[derive(Debug, Clone)]
pub struct GMRooms {
    pub rooms: Vec<GMRoom>,
    pub exists: bool,
}
impl GMChunkElement for GMRooms {
    fn stub() -> Self {
        Self { rooms: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMRooms {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let rooms: Vec<GMRoom> = reader.read_pointer_list()?;
        Ok(Self { rooms, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.rooms)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)] // Need explicit layout so memory addresses for gm pointers don't collide
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
    pub instance_creation_order_ids: Vec<i32>,
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let caption: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let speed = reader.read_u32()?;
        let persistent = reader.read_bool32()?;
        let background_color: u32 = reader.read_u32()? | 0xFF000000; // make alpha 255 (background color doesn't have transparency)
        let draw_background_color = reader.read_bool32()?;
        let creation_code: Option<GMRef<GMCode>> = reader.read_resource_by_id_opt()?;
        let flags = GMRoomFlags::deserialize(reader)?;

        let backgrounds_ptr = reader.read_u32()?;
        let views_ptr = reader.read_u32()?;
        let game_objects_ptr = reader.read_u32()?;
        let tiles_ptr = reader.read_u32()?;
        let instances_ptr = if reader.general_info.is_version_at_least((2024, 13)) {
            reader.read_u32()?
        } else {
            0
        };

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
        let backgrounds: Vec<GMRoomBackground> = reader.read_pointer_list()?;

        reader.assert_pos(views_ptr, "Room Views")?;
        let views: Vec<GMRoomView> = reader.read_pointer_list()?;

        reader.assert_pos(game_objects_ptr, "Room Game Objects")?;
        let game_objects: Vec<GMRoomGameObject> = reader.read_pointer_list()?;

        reader.assert_pos(tiles_ptr, "Room Tiles")?;
        let tiles: Vec<GMRoomTile> = reader.read_pointer_list()?;

        let instance_creation_order_ids: Vec<i32> = if reader.general_info.is_version_at_least((2024, 13)) {
            reader.assert_pos(instances_ptr, "Room Instance Creation Order IDs")?;
            reader.read_simple_list()?
        } else {
            Vec::new()
        };

        let layers: Vec<GMRoomLayer> = if reader.general_info.is_version_at_least((2, 0)) {
            reader.assert_pos(layers_ptr, "Room Layers")?;
            reader.read_pointer_list()?
        } else {
            Vec::new()
        };

        let sequences: Vec<GMSequence> = if reader.general_info.is_version_at_least((2, 3)) {
            reader.assert_pos(sequences_ptr, "Room Sequences")?;
            reader.read_pointer_list()?
        } else {
            Vec::new()
        };

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
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string_opt(&self.caption)?;
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        builder.write_u32(self.speed);
        builder.write_bool32(self.persistent);
        builder.write_u32(self.background_color ^ 0xFF000000); // remove alpha (background color doesn't have alpha)
        builder.write_bool32(self.draw_background_color);
        builder.write_resource_id_opt(&self.creation_code);
        self.flags.serialize(builder)?;
        builder.write_pointer(&self.backgrounds)?;
        builder.write_pointer(&self.views)?;
        builder.write_pointer(&self.game_objects)?;
        builder.write_pointer(&self.tiles)?;
        if builder.is_gm_version_at_least((2024, 13)) {
            builder.write_pointer(&self.instance_creation_order_ids)?;
        }
        builder.write_bool32(self.world);
        builder.write_u32(self.top);
        builder.write_u32(self.left);
        builder.write_u32(self.right);
        builder.write_u32(self.bottom);
        builder.write_f32(self.gravity_x);
        builder.write_f32(self.gravity_y);
        builder.write_f32(self.meters_per_pixel);
        if builder.is_gm_version_at_least((2, 0)) {
            builder.write_pointer(&self.layers)?;
        }
        if builder.is_gm_version_at_least((2, 3)) {
            builder.write_pointer(&self.sequences)?;
        }
        builder.resolve_pointer(&self.backgrounds)?;
        builder.write_pointer_list(&self.backgrounds)?;
        builder.resolve_pointer(&self.views)?;
        builder.write_pointer_list(&self.views)?;
        builder.resolve_pointer(&self.game_objects)?;
        builder.write_pointer_list(&self.game_objects)?;
        builder.resolve_pointer(&self.tiles)?;
        builder.write_pointer_list(&self.tiles)?;
        if builder.is_gm_version_at_least((2024, 13)) {
            builder.resolve_pointer(&self.instance_creation_order_ids)?;
            builder.write_pointer_list(&self.instance_creation_order_ids)?;
        }
        if builder.is_gm_version_at_least((2, 0)) {
            builder.resolve_pointer(&self.layers)?;
            builder.write_pointer_list(&self.layers)?;
        }
        if builder.is_gm_version_at_least((2, 3)) {
            builder.resolve_pointer(&self.sequences)?;
            builder.write_pointer_list(&self.sequences)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomFlags {
    pub enable_views: bool,              // Views are enabled
    pub show_color: bool,                // Meaning uncertain
    pub dont_clear_display_buffer: bool, // Don't clear display buffer
    pub is_gms2: bool,                   // Room was made in GameMaker: Studio 2
    pub is_gms2_3: bool,                 // Room was made in GameMaker: Studio 2.3
}
impl GMElement for GMRoomFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw = reader.read_u32()?;
        Ok(GMRoomFlags {
            enable_views: 0 != raw & 0x1,
            show_color: 0 != raw & 0x2,
            dont_clear_display_buffer: 0 != raw & 0x4,
            is_gms2: 0 != raw & 0x20000,
            is_gms2_3: 0 != raw & 0x10000,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut raw: u32 = 0;
        raw |= self.enable_views as u32 * 0x1;
        raw |= self.show_color as u32 * 0x2;
        raw |= self.dont_clear_display_buffer as u32 * 0x4;
        raw |= self.is_gms2 as u32 * 0x20000;
        raw |= self.is_gms2_3 as u32 * 0x10000;
        builder.write_u32(raw);
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let enabled = reader.read_bool32()?;
        let view_x = reader.read_i32()?;
        let view_y = reader.read_i32()?;
        let view_width = reader.read_i32()?;
        let view_height = reader.read_i32()?;
        let port_x = reader.read_i32()?;
        let port_y = reader.read_i32()?;
        let port_width = reader.read_i32()?;
        let port_height = reader.read_i32()?;
        let border_x = reader.read_u32()?;
        let border_y = reader.read_u32()?;
        let speed_x = reader.read_i32()?;
        let speed_y = reader.read_i32()?;
        let object: Option<GMRef<GMGameObject>> = reader.read_resource_by_id_opt()?;

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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.enabled);
        builder.write_i32(self.view_x);
        builder.write_i32(self.view_y);
        builder.write_i32(self.view_width);
        builder.write_i32(self.view_height);
        builder.write_i32(self.port_x);
        builder.write_i32(self.port_y);
        builder.write_i32(self.port_width);
        builder.write_i32(self.port_height);
        builder.write_u32(self.border_x);
        builder.write_u32(self.border_y);
        builder.write_i32(self.speed_x);
        builder.write_i32(self.speed_y);
        builder.write_resource_id_opt(&self.object);
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let enabled = reader.read_bool32()?;
        let foreground = reader.read_bool32()?;
        let background_definition: Option<GMRef<GMBackground>> = reader.read_resource_by_id_opt()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let tile_x = reader.read_i32()?; // Idk if this should be an int instead of a bool
        let tile_y = reader.read_i32()?; // ^
        let speed_x = reader.read_i32()?;
        let speed_y = reader.read_i32()?;
        let stretch = reader.read_bool32()?;

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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.enabled);
        builder.write_bool32(self.foreground);
        builder.write_resource_id_opt(&self.background_definition);
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_i32(self.tile_x);
        builder.write_i32(self.tile_y);
        builder.write_i32(self.speed_x);
        builder.write_i32(self.speed_y);
        builder.write_bool32(self.stretch);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomTile {
    pub x: i32,
    pub y: i32,
    pub texture: Option<GMRoomTileTexture>,
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let texture: Option<GMRoomTileTexture> = if reader.general_info.is_version_at_least((2, 0)) {
            reader.read_resource_by_id_opt()?.map(GMRoomTileTexture::Sprite)
        } else {
            reader.read_resource_by_id_opt()?.map(GMRoomTileTexture::Background)
        };
        let source_x = reader.read_u32()?;
        let source_y = reader.read_u32()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let tile_depth = reader.read_i32()?;
        let instance_id = reader.read_u32()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let color = reader.read_u32()?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        // TODO this is going to cause mod compatibility issues
        match self.texture {
            Some(GMRoomTileTexture::Sprite(sprite_ref)) => {
                if builder.is_gm_version_at_least((2, 0)) {
                    builder.write_resource_id(&sprite_ref)
                } else {
                    bail!("Room tile texture should be a Background reference before GMS2; not a Sprite reference");
                }
            }
            Some(GMRoomTileTexture::Background(background_ref)) => {
                if builder.is_gm_version_at_least((2, 0)) {
                    bail!("Room tile texture should be a Sprite reference in GMS2+; not a Background reference");
                } else {
                    builder.write_resource_id(&background_ref)
                }
            }
            None => builder.write_u32(0),
        }
        builder.write_u32(self.source_x);
        builder.write_u32(self.source_y);
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        builder.write_i32(self.tile_depth);
        builder.write_u32(self.instance_id);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_u32(self.color);
        Ok(())
    }
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
    pub effect_data_2022_1: Option<GMRoomLayer2022_1>,
    pub data: GMRoomLayerData,
}
impl GMElement for GMRoomLayer {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let layer_name: GMRef<String> = reader.read_gm_string()?;
        let layer_id = reader.read_u32()?;
        let layer_type: GMRoomLayerType = num_enum_from(reader.read_u32()?)?;
        let layer_depth = reader.read_i32()?;
        let x_offset = reader.read_f32()?;
        let y_offset = reader.read_f32()?;
        let horizontal_speed = reader.read_f32()?;
        let vertical_speed = reader.read_f32()?;
        let is_visible = reader.read_bool32()?;
        let effect_data_2022_1: Option<GMRoomLayer2022_1> = reader.deserialize_if_gm_version((2022, 1))?;

        let data: GMRoomLayerData = match layer_type {
            GMRoomLayerType::Path | GMRoomLayerType::Path2 => GMRoomLayerData::None,
            GMRoomLayerType::Background => GMRoomLayerData::Background(GMRoomLayerDataBackground::deserialize(reader)?),
            GMRoomLayerType::Instances => GMRoomLayerData::Instances(GMRoomLayerDataInstances::deserialize(reader)?),
            GMRoomLayerType::Assets => GMRoomLayerData::Assets(GMRoomLayerDataAssets::deserialize(reader)?),
            GMRoomLayerType::Tiles => GMRoomLayerData::Tiles(GMRoomLayerDataTiles::deserialize(reader)?),
            GMRoomLayerType::Effect => {
                if reader.general_info.is_version_at_least((2022, 1)) {
                    let effect_type: GMRef<String> = effect_data_2022_1
                        .as_ref()
                        .unwrap()
                        .effect_type
                        .context("Effect Type not set for Room Layer 2022.1+ (this error could be a mistake)")?;
                    let properties: Vec<GMRoomLayerEffectProperty> =
                        effect_data_2022_1.as_ref().unwrap().effect_properties.clone();
                    GMRoomLayerData::Effect(GMRoomLayerDataEffect { effect_type, properties })
                } else {
                    GMRoomLayerData::Effect(GMRoomLayerDataEffect::deserialize(reader)?)
                }
            }
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
            effect_data_2022_1,
            data,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.layer_name)?;
        builder.write_u32(self.layer_id);
        builder.write_u32(self.layer_type.into());
        builder.write_i32(self.layer_depth);
        builder.write_f32(self.x_offset);
        builder.write_f32(self.y_offset);
        builder.write_f32(self.horizontal_speed);
        builder.write_f32(self.vertical_speed);
        builder.write_bool32(self.is_visible);
        self.effect_data_2022_1
            .serialize_if_gm_ver(builder, "Effect Data", (2022, 1))?;
        match &self.data {
            GMRoomLayerData::None => {}
            GMRoomLayerData::Instances(data) => data.serialize(builder)?,
            GMRoomLayerData::Tiles(data) => data.serialize(builder)?,
            GMRoomLayerData::Background(data) => data.serialize(builder)?,
            GMRoomLayerData::Assets(data) => data.serialize(builder)?,
            GMRoomLayerData::Effect(data) => {
                if !builder.is_gm_version_at_least((2022, 1)) {
                    data.serialize(builder)?
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayer2022_1 {
    pub effect_enabled: bool,
    pub effect_type: Option<GMRef<String>>,
    pub effect_properties: Vec<GMRoomLayerEffectProperty>,
}
impl GMElement for GMRoomLayer2022_1 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let effect_enabled = reader.read_bool32()?;
        let effect_type: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let effect_properties: Vec<GMRoomLayerEffectProperty> = reader.read_simple_list()?;
        Ok(GMRoomLayer2022_1 { effect_enabled, effect_type, effect_properties })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.effect_enabled);
        builder.write_gm_string_opt(&self.effect_type)?;
        builder.write_simple_list(&self.effect_properties)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerEffectProperty {
    pub kind: GMRoomLayerEffectPropertyType,
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}
impl GMElement for GMRoomLayerEffectProperty {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let kind: GMRoomLayerEffectPropertyType = num_enum_from(reader.read_i32()?)?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(GMRoomLayerEffectProperty { kind, name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.kind.into());
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string(&self.value)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
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
}
impl GMElement for GMRoomLayerDataInstances {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let instances: Vec<u32> = reader.read_simple_list()?;
        Ok(GMRoomLayerDataInstances { instances })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.instances)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataTiles {
    pub background: Option<GMRef<GMBackground>>,
    /// Flattened 2D Array. Access using `tile_data[row + width * col]`.
    pub tile_data: Vec<u32>,
    pub width: u32,
    pub height: u32,
}
impl GMElement for GMRoomLayerDataTiles {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let background: Option<GMRef<GMBackground>> = reader.read_resource_by_id_opt()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let mut tile_data: Vec<u32> = vec_with_capacity(width * height)?;

        if reader.general_info.is_version_at_least((2024, 2)) {
            Self::read_compressed_tile_data(reader, &mut tile_data)?;
        } else {
            for _y in 0..height {
                for _x in 0..width {
                    tile_data.push(reader.read_u32()?);
                }
            }
        }

        Ok(GMRoomLayerDataTiles { background, tile_data, width, height })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id_opt(&self.background);
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        if builder.is_gm_version_at_least((2024, 2)) {
            self.build_compressed_tile_data(builder)?;
        } else {
            for id in &self.tile_data {
                builder.write_u32(*id);
            }
        }
        Ok(())
    }
}
impl GMRoomLayerDataTiles {
    fn read_compressed_tile_data(reader: &mut DataReader, tile_data: &mut Vec<u32>) -> Result<()> {
        let total_size: usize = tile_data.capacity();
        if total_size == 0 {
            return Ok(());
        }

        'outer: loop {
            let length = reader.read_u8()?;
            if length >= 128 {
                // Repeat run
                let run_length: u8 = (length & 0x7F) + 1;
                let tile = reader.read_u32()?;
                for _ in 0..run_length {
                    tile_data.push(tile);
                    if tile_data.len() >= total_size {
                        break 'outer;
                    }
                }
            } else {
                // Verbatim run
                for _ in 0..length {
                    let tile = reader.read_u32()?;
                    tile_data.push(tile);
                    if tile_data.len() >= total_size {
                        break 'outer;
                    }
                }
            }
        }

        // Due to a GMAC bug, 2 blank tiles are inserted into the layer
        // If the last 2 tiles in the layer are different.
        // This is a certified YoyoGames moment right here.
        let has_padding: bool = if tile_data.len() == 1 {
            true // Single tile always has padding
        } else if tile_data.len() >= 2 {
            let len = tile_data.len();
            tile_data[len - 1] != tile_data[len - 2]
        } else {
            false // no tiles => no padding (should never happen though?)
        };
        if has_padding {
            let length = reader.read_u8()?;
            let tile = reader.read_u32()?;

            // Sanity check: run of 2 empty tiles
            if length != 0x81 {
                bail!("Expected 0x81 for run length of compressed tile data padding; got 0x{length:02X}");
            }
            if tile != u32::MAX {
                bail!("Expected -1 for tile of compressed tile data padding; got 0x{length:02X}");
            }
        }

        if reader.general_info.is_version_at_least((2024, 4)) {
            reader.align(4)?;
        }
        Ok(())
    }

    fn build_compressed_tile_data(&self, builder: &mut DataBuilder) -> Result<()> {
        let tile_count: usize = self.tile_data.len();
        if tile_count == 0 {
            return Ok(());
        }

        // Perform run-length encoding using process identical to GameMaker's logic.
        // This only serializes data when outputting a repeat run, upon which the
        // Previous verbatim run is serialized first.
        // We also iterate in 1D, which requires some division and modulo to work with
        // The 2D array we have for representation here.
        let mut last_tile: u32 = self.tile_data[0];
        let mut num_verbatim: i32 = 0;
        let mut verbatim_start: i32 = 0;
        let mut i = 1;

        // Note: we go out of bounds to ensure a repeat run at the end
        while i <= tile_count + 1 {
            let mut curr_tile: u32 = if i >= tile_count { u32::MAX } else { self.tile_data[i] };
            i += 1;

            if curr_tile != last_tile {
                // We have different tiles, so just increase the number of tiles in this verbatim run
                num_verbatim += 1;
                last_tile = curr_tile;
                continue;
            }

            // We have two tiles in a row - construct a repeating run.
            // Figure out how far this repeat goes, first.
            let mut num_repeats: i32 = 2;
            while i < tile_count {
                if curr_tile != self.tile_data[i] {
                    break;
                }
                num_repeats += 1;
                i += 1;
            }

            // Serialize the preceding verbatim run, splitting into 127-length chunks
            while num_verbatim > 0 {
                let num_to_write: i32 = min(num_verbatim, 127);
                builder.write_u8(num_to_write as u8);
                for j in 0..num_to_write {
                    let tile: u32 = self.tile_data[(verbatim_start + j) as usize];
                    builder.write_u32(tile);
                }
                num_verbatim -= num_to_write;
                verbatim_start += num_to_write;
            }

            // Serialize this repeat run, splitting into 128-length chunks
            while num_repeats > 0 {
                let num_to_write: i32 = min(num_verbatim, 128);
                builder.write_u8((num_to_write as u8 - 1) | 0x80);
                builder.write_u32(last_tile);
                num_repeats -= num_to_write;
            }

            // Update our current tile to be the one after the run
            curr_tile = if i >= tile_count { 0 } else { self.tile_data[i] };

            // Update the start of our next verbatim run, and move on
            verbatim_start = i as i32;
            num_verbatim = 0;
            i += 1;
            last_tile = curr_tile;
        }

        if builder.is_gm_version_at_least((2024, 4)) {
            builder.align(4);
        }
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let visible = reader.read_bool32()?;
        let foreground = reader.read_bool32()?;
        let sprite: Option<GMRef<GMSprite>> = reader.read_resource_by_id_opt()?;
        let tiled_horizontally = reader.read_bool32()?;
        let tiled_vertically = reader.read_bool32()?;
        let stretch = reader.read_bool32()?;
        let color = reader.read_u32()?;
        let first_frame = reader.read_f32()?;
        let animation_speed = reader.read_f32()?;
        let animation_speed_type: GMAnimSpeedType = num_enum_from(reader.read_u32()?)?;

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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.visible);
        builder.write_bool32(self.foreground);
        builder.write_resource_id_opt(&self.sprite);
        builder.write_bool32(self.tiled_horizontally);
        builder.write_bool32(self.tiled_vertically);
        builder.write_bool32(self.stretch);
        builder.write_u32(self.color);
        builder.write_f32(self.first_frame);
        builder.write_f32(self.animation_speed);
        builder.write_u32(self.animation_speed_type.into());
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let legacy_tiles_pointer = reader.read_u32()?;
        let sprites_pointer = reader.read_u32()?;
        let mut sequences_pointer: u32 = 0;
        let mut nine_slices_pointer: u32 = 0;
        let mut particle_systems_pointer: u32 = 0;
        let mut text_items_pointer: u32 = 0;

        if reader.general_info.is_version_at_least((2, 3)) {
            sequences_pointer = reader.read_u32()?;
            if !reader.general_info.is_version_at_least((2, 3, 2)) {
                nine_slices_pointer = reader.read_u32()?;
            }
            if reader.general_info.is_version_at_least((2023, 2, LTSBranch::PostLTS)) {
                particle_systems_pointer = reader.read_u32()?;
            }
            if reader.general_info.is_version_at_least((2024, 6)) {
                text_items_pointer = reader.read_u32()?;
            }
        }

        reader.assert_pos(legacy_tiles_pointer, "Legacy Tiles")?;
        let legacy_tiles: Vec<GMRoomTile> = reader.read_pointer_list()?;

        reader.assert_pos(sprites_pointer, "Sprite Instances")?;
        let sprites: Vec<GMSpriteInstance> = reader.read_pointer_list()?;

        let mut sequences: Vec<GMSequenceInstance> = Vec::new();
        let mut nine_slices: Vec<GMSpriteInstance> = Vec::new();
        let mut particle_systems: Vec<GMParticleSystemInstance> = Vec::new();
        let mut text_items: Vec<GMTextItemInstance> = Vec::new();

        if reader.general_info.is_version_at_least((2, 3)) {
            reader.assert_pos(sequences_pointer, "Sequences")?;
            sequences = reader.read_pointer_list()?;

            if !reader.general_info.is_version_at_least((2, 3, 2)) {
                reader.assert_pos(nine_slices_pointer, "Nine Slices")?;
                nine_slices = reader.read_pointer_list()?;
            }
            if reader.general_info.is_version_at_least((2023, 2, LTSBranch::PostLTS)) {
                reader.assert_pos(particle_systems_pointer, "Particle Systems")?;
                particle_systems = reader.read_pointer_list()?;
            }

            if reader.general_info.is_version_at_least((2024, 6)) {
                reader.assert_pos(text_items_pointer, "Text Items")?;
                text_items = reader.read_pointer_list()?;
            }
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer(&self.legacy_tiles)?;
        builder.write_pointer(&self.sprites)?;

        if builder.is_gm_version_at_least((2, 3)) {
            builder.write_pointer(&self.sequences)?;
            if !builder.is_gm_version_at_least((2, 3, 2)) {
                builder.write_pointer(&self.nine_slices)?;
            }
            if builder.is_gm_version_at_least((2023, 2, LTSBranch::PostLTS)) {
                builder.write_pointer(&self.particle_systems)?;
            }
            if builder.is_gm_version_at_least((2024, 6)) {
                builder.write_pointer(&self.text_items)?;
            }
        }
        builder.resolve_pointer(&self.legacy_tiles)?;
        builder.write_pointer_list(&self.legacy_tiles)?;

        builder.resolve_pointer(&self.sprites)?;
        builder.write_pointer_list(&self.sprites)?;

        if builder.is_gm_version_at_least((2, 3)) {
            builder.resolve_pointer(&self.sequences)?;
            builder.write_pointer_list(&self.sequences)?;

            if !builder.is_gm_version_at_least((2, 3, 2)) {
                builder.resolve_pointer(&self.nine_slices)?;
                builder.write_pointer_list(&self.nine_slices)?;
            }
            if builder.is_gm_version_at_least((2023, 2, LTSBranch::PostLTS)) {
                builder.resolve_pointer(&self.particle_systems)?;
                builder.write_pointer_list(&self.particle_systems)?;
            }
            if builder.is_gm_version_at_least((2024, 6)) {
                builder.resolve_pointer(&self.text_items)?;
                builder.write_pointer_list(&self.text_items)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMRoomLayerDataEffect {
    pub effect_type: GMRef<String>,
    pub properties: Vec<GMRoomLayerEffectProperty>,
}
impl GMElement for GMRoomLayerDataEffect {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        // {~~} dont serialize_old if >= 2022.1??
        let effect_type: GMRef<String> = reader.read_gm_string()?;
        let properties: Vec<GMRoomLayerEffectProperty> = reader.read_simple_list()?;
        Ok(GMRoomLayerDataEffect { effect_type, properties })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.effect_type)?;
        builder.write_simple_list(&self.properties)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteInstance {
    pub name: GMRef<String>,
    pub sprite: Option<GMRef<GMSprite>>,
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let sprite: Option<GMRef<GMSprite>> = reader.read_resource_by_id_opt()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let color = reader.read_u32()?;
        let animation_speed = reader.read_f32()?;
        let animation_speed_type: GMAnimSpeedType = num_enum_from(reader.read_u32()?)?;
        let frame_index = reader.read_f32()?;
        let rotation = reader.read_f32()?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_resource_id_opt(&self.sprite);
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_u32(self.color);
        builder.write_f32(self.animation_speed);
        builder.write_u32(self.animation_speed_type.into());
        builder.write_f32(self.frame_index);
        builder.write_f32(self.rotation);
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let sequence: GMRef<GMSequence> = reader.read_resource_by_id()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let color = reader.read_u32()?;
        let animation_speed = reader.read_f32()?;
        let animation_speed_type: GMAnimSpeedType = num_enum_from(reader.read_u32()?)?;
        let frame_index = reader.read_f32()?;
        let rotation = reader.read_f32()?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_resource_id(&self.sequence);
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_u32(self.color);
        builder.write_f32(self.animation_speed);
        builder.write_u32(self.animation_speed_type.into());
        builder.write_f32(self.frame_index);
        builder.write_f32(self.rotation);
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let particle_system: GMRef<GMParticleSystem> = reader.read_resource_by_id()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let color = reader.read_u32()?;
        let rotation = reader.read_f32()?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_resource_id(&self.particle_system);
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_u32(self.color);
        builder.write_f32(self.rotation);
        Ok(())
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let font: GMRef<GMFont> = reader.read_resource_by_id()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let rotation = reader.read_f32()?;
        let color = reader.read_u32()?;
        let origin_x = reader.read_f32()?;
        let origin_y = reader.read_f32()?;
        let text: GMRef<String> = reader.read_gm_string()?;
        let alignment = reader.read_i32()?;
        let character_spacing = reader.read_f32()?;
        let line_spacing = reader.read_f32()?;
        let frame_width = reader.read_f32()?;
        let frame_height = reader.read_f32()?;
        let wrap = reader.read_bool32()?;

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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_resource_id(&self.font);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_f32(self.rotation);
        builder.write_u32(self.color);
        builder.write_f32(self.origin_x);
        builder.write_f32(self.origin_y);
        builder.write_gm_string(&self.text)?;
        builder.write_i32(self.alignment);
        builder.write_f32(self.character_spacing);
        builder.write_f32(self.line_spacing);
        builder.write_f32(self.frame_width);
        builder.write_f32(self.frame_height);
        builder.write_bool32(self.wrap);
        Ok(())
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
    pub image_index: Option<u32>,
    pub color: u32,
    pub rotation: f32,
    pub pre_create_code: Option<GMRef<GMCode>>,
}
impl GMElement for GMRoomGameObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let object_definition: GMRef<GMGameObject> = reader.read_resource_by_id()?;
        let instance_id = reader.read_u32()?;
        let creation_code: Option<GMRef<GMCode>> = reader.read_resource_by_id_opt()?;
        let scale_x = reader.read_f32()?;
        let scale_y = reader.read_f32()?;
        let mut image_speed: Option<f32> = None;
        let mut image_index: Option<u32> = None;
        if reader.general_info.is_version_at_least((2, 2, 2, 302)) {
            image_speed = Some(reader.read_f32()?);
            image_index = Some(reader.read_u32()?);
        }
        let color = reader.read_u32()?;
        let rotation = reader.read_f32()?; // {~~} FloatAsInt (negative zero handling stuff)

        // [From UndertaleModTool] "is that dependent on bytecode or something else?"
        let pre_create_code: Option<GMRef<GMCode>> = if reader.general_info.bytecode_version >= 16 {
            reader.read_resource_by_id_opt()?
        } else {
            None
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_resource_id(&self.object_definition);
        builder.write_u32(self.instance_id);
        builder.write_resource_id_opt(&self.creation_code);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        self.image_speed
            .serialize_if_gm_ver(builder, "Image Speed", (2, 2, 2, 302))?;
        self.image_index
            .serialize_if_gm_ver(builder, "Image Index", (2, 2, 2, 302))?;
        builder.write_u32(self.color);
        builder.write_f32(self.rotation);
        if builder.bytecode_version() >= 16 {
            builder.write_resource_id_opt(&self.pre_create_code);
        };
        Ok(())
    }
}
