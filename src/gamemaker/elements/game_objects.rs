use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::elements::sprites::GMSprite;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::utility::num_enum_from;

#[derive(Debug, Clone)]
pub struct GMGameObjects {
    pub game_objects: Vec<GMGameObject>,
    pub exists: bool,
}

impl GMChunkElement for GMGameObjects {
    fn empty() -> Self {
        Self { game_objects: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMGameObjects {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let pointers: Vec<usize> = reader.read_simple_list()?;
        let mut game_objects: Vec<GMGameObject> = Vec::with_capacity(pointers.len());

        for pointer in pointers {
            reader.cur_pos = pointer;
            let name: GMRef<String> = reader.read_gm_string()?;
            let sprite_index: i32 = reader.read_i32()?;
            let sprite: Option<GMRef<GMSprite>> = if sprite_index == -1 {
                None
            } else {
                let index: u32 = sprite_index.try_into().map_err(|_| format!(
                    "Invalid negative sprite index {} for game object's sprite \"{}\" at absolute position {}",
                    sprite_index, reader.display_gm_str(name), reader.cur_pos,
                ))?;
                Some(GMRef::new(index))
            };
            let visible: bool = reader.read_bool32()?;
            let mut managed: Option<bool> = None;
            if reader.general_info.is_version_at_least((2022, 5)) {
                managed = Some(reader.read_bool32()?);
            }
            let solid: bool = reader.read_bool32()?;
            let depth: i32 = reader.read_i32()?;
            let persistent: bool = reader.read_bool32()?;
            let parent_id: i32 = reader.read_i32()?;
            let parent: Option<GMRef<GMGameObject>> = match parent_id {
                -100 => None,   // No parent
                -1 => Some(GMRef::new(game_objects.len() as u32)),    // Parent is Self
                _ => {
                    let parent_id: u32 = u32::try_from(parent_id)
                        .map_err(|_| format!("Invalid game object parent id {parent_id}"))?;
                    Some(GMRef::new(parent_id))
                },
            };
            let sprite_index: i32 = reader.read_i32()?;
            let texture_mask: Option<GMRef<GMSprite>> = if sprite_index == -1 {
                None
            } else {
                let index: u32 = sprite_index.try_into().map_err(|_| format!(
                    "Invalid negative sprite index {} for texture mask of game object \"{}\" at absolute position {}",
                    sprite_index, reader.display_gm_str(name), reader.cur_pos,
                ))?;
                Some(GMRef::new(index))
            };
            let uses_physics: bool = reader.read_bool32()?;
            let is_sensor: bool = reader.read_bool32()?;
            let collision_shape: GMGameObjectCollisionShape = num_enum_from(reader.read_u32()?)
                .map_err(|e| format!("{e} while parsing Game Object"))?;
            let density: f32 = reader.read_f32()?;
            let restitution: f32 = reader.read_f32()?;
            let group: u32 = reader.read_u32()?;
            let linear_damping: f32 = reader.read_f32()?;
            let angular_damping: f32 = reader.read_f32()?;
            let physics_shape_vertex_count: i32 = reader.read_i32()?;
            let uses_physics_shape_vertex: bool = physics_shape_vertex_count != -1;
            let physics_shape_vertex_count: usize = if physics_shape_vertex_count < 0 { 0 } else { physics_shape_vertex_count as usize };
            let friction: f32 = reader.read_f32()?;
            let awake: bool = reader.read_bool32()?;
            let kinematic: bool = reader.read_bool32()?;
            let mut physics_shape_vertices: Vec<(f32, f32)> = Vec::with_capacity(physics_shape_vertex_count);
            for _ in 0..physics_shape_vertex_count {
                let x: f32 = reader.read_f32()?;
                let y: f32 = reader.read_f32()?;
                physics_shape_vertices.push((x, y));
            }
            let events: Vec<GMGameObjectEvents> = reader.read_pointer_list()?;

            game_objects.push(GMGameObject {
                name,
                sprite,
                visible,
                managed,
                solid,
                depth,
                persistent,
                parent,
                texture_mask,
                uses_physics,
                is_sensor,
                collision_shape,
                density,
                restitution,
                group,
                linear_damping,
                angular_damping,
                friction,
                awake,
                kinematic,
                physics_shape_vertices,
                uses_physics_shape_vertex,
                events,
            });
        }

        Ok(Self { game_objects, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_usize(self.game_objects.len())?;
        let pointer_list_pos: usize = builder.len();
        for _ in 0..self.game_objects.len() {
            builder.write_u32(0xDEADC0DE);
        }

        for (i, game_object) in self.game_objects.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_pos + 4*i)?;

            builder.write_gm_string(&game_object.name)?;
            builder.write_resource_id_opt(&game_object.sprite);
            builder.write_bool32(game_object.visible);
            game_object.managed.serialize_if_gm_ver(builder, "Managed", (2022, 5))?;
            builder.write_bool32(game_object.solid);
            builder.write_i32(game_object.depth);
            builder.write_bool32(game_object.persistent);
            match game_object.parent {
                None => builder.write_i32(-100),    // No Parent
                Some(obj_ref) if obj_ref.index == i as u32 => builder.write_i32(-1),    // Parent is Self
                Some(obj_ref) => builder.write_resource_id(&obj_ref),   // Normal Parent
            }
            builder.write_resource_id_opt(&game_object.texture_mask);
            builder.write_bool32(game_object.uses_physics);
            builder.write_bool32(game_object.is_sensor);
            builder.write_u32(game_object.collision_shape.into());
            builder.write_f32(game_object.density);
            builder.write_f32(game_object.restitution);
            builder.write_u32(game_object.group);
            builder.write_f32(game_object.linear_damping);
            builder.write_f32(game_object.angular_damping);
            builder.write_usize(game_object.physics_shape_vertices.len())?;   // "new meaning" according to UTMT?
            builder.write_f32(game_object.friction);
            builder.write_bool32(game_object.awake);
            builder.write_bool32(game_object.kinematic);
            for (x, y) in &game_object.physics_shape_vertices {
                builder.write_f32(*x);
                builder.write_f32(*y);
            }
            builder.write_pointer_list(&game_object.events)?;
        }
        Ok(())
    }
}

impl GMGameObjects {
    pub fn get_object_ref_by_name(&self, name: &str, gm_strings: &GMStrings) -> Result<GMRef<GMGameObject>, String> {
        for (i, game_object) in self.game_objects.iter().enumerate() {
            let object_name: &String = game_object.name.resolve(&gm_strings.strings)?;
            if object_name == name {
                return Ok(GMRef::new(i as u32))
            }
        }
        Err(format!("Could not resolve game object with name \"{name}\""))
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObject {
    pub name: GMRef<String>,
    pub sprite: Option<GMRef<GMSprite>>,
    pub visible: bool,
    /// Introduced in 2022.5.
    pub managed: Option<bool>,
    pub solid: bool,
    pub depth: i32,
    pub persistent: bool,
    pub parent: Option<GMRef<GMGameObject>>,
    pub texture_mask: Option<GMRef<GMSprite>>,
    pub uses_physics: bool,
    pub is_sensor: bool,
    pub collision_shape: GMGameObjectCollisionShape,
    pub density: f32,
    pub restitution: f32,
    pub group: u32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub friction: f32,
    pub awake: bool,
    pub kinematic: bool,
    pub physics_shape_vertices: Vec<(f32, f32)>,
    pub uses_physics_shape_vertex: bool,
    pub events: Vec<GMGameObjectEvents>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObjectEvents {
    pub events: Vec<GMGameObjectEvent>,
}
impl GMElement for GMGameObjectEvents {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let events: Vec<GMGameObjectEvent> = reader.read_pointer_list()?;
        Ok(Self { events })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_pointer_list(&self.events)
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObjectEvent {
    pub subtype: u32,
    pub actions: Vec<GMGameObjectEventAction>,
}
impl GMElement for GMGameObjectEvent {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let subtype: u32 = reader.read_u32()?;
        let actions: Vec<GMGameObjectEventAction> = reader.read_pointer_list()?;
        Ok(Self { subtype, actions })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.subtype);
        builder.write_pointer_list(&self.actions)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObjectEventAction {
    pub lib_id: u32,
    pub id: u32,
    pub kind: u32,
    pub use_relative: bool,
    pub is_question: bool,
    pub use_apply_to: bool,
    pub exe_type: u32,
    pub action_name: Option<GMRef<String>>,
    pub code: Option<GMRef<GMCode>>,
    pub argument_count: u32,
    pub who: i32,
    pub relative: bool,
    pub is_not: bool,
    pub unknown_always_zero: u32,
}
impl GMElement for GMGameObjectEventAction {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let lib_id: u32 = reader.read_u32()?;
        let id: u32 = reader.read_u32()?;
        let kind: u32 = reader.read_u32()?;
        let use_relative: bool = reader.read_bool32()?;
        let is_question: bool = reader.read_bool32()?;
        let use_apply_to: bool = reader.read_bool32()?;
        let exe_type: u32 = reader.read_u32()?;
        let action_name: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let code: Option<GMRef<GMCode>> = reader.read_resource_by_id_opt()?;
        let argument_count: u32 = reader.read_u32()?;
        let who: i32 = reader.read_i32()?;
        let relative: bool = reader.read_bool32()?;
        let is_not: bool = reader.read_bool32()?;
        let unknown_always_zero: u32 = reader.read_u32()?;

        Ok(GMGameObjectEventAction {
            lib_id,
            id,
            kind,
            use_relative,
            is_question,
            use_apply_to,
            exe_type,
            action_name,
            code,
            argument_count,
            who,
            relative,
            is_not,
            unknown_always_zero,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.lib_id);
        builder.write_u32(self.id);
        builder.write_u32(self.kind);
        builder.write_bool32(self.use_relative);
        builder.write_bool32(self.is_question);
        builder.write_bool32(self.use_apply_to);
        builder.write_u32(self.exe_type);
        builder.write_gm_string_opt(&self.action_name)?;
        builder.write_resource_id_opt(&self.code);
        builder.write_u32(self.argument_count);
        builder.write_i32(self.who);
        builder.write_bool32(self.relative);
        builder.write_bool32(self.is_not);
        builder.write_u32(self.unknown_always_zero);
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum GMGameObjectCollisionShape {
    Circle = 0,
    Box = 1,
    Custom = 2,
}

