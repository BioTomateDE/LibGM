use crate::deserialize::chunk_reading::GMRef;
use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::code::GMCode;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::sprites::GMSprite;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObject {
    pub name: GMRef<String>,
    pub sprite: Option<GMRef<GMSprite>>,
    pub visible: bool,
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
    pub events: Vec<Vec<GMGameObjectEvent>>,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum GMGameObjectCollisionShape {
    Circle = 0,
    Box = 1,
    Custom = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObjectEvent {
    pub subtype: u32,
    pub actions: Vec<GMGameObjectEventAction>,
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

#[derive(Debug, Clone)]
pub struct GMGameObjects {
    pub game_objects_by_index: Vec<GMGameObject>,
}


pub fn parse_chunk_objt(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMGameObjects, String> {
    chunk.cur_pos = 0;
    let game_objects_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(game_objects_count);
    for _ in 0..game_objects_count {
        start_positions.push(chunk.read_usize_pos()? - chunk.abs_pos);
    }

    let mut game_objects_by_index: Vec<GMGameObject> = Vec::with_capacity(game_objects_count);
    for start_position in start_positions {
        chunk.cur_pos = start_position;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let sprite: Option<GMRef<GMSprite>> = match chunk.read_i32()? {
            -1 => None,
            index => Some(GMRef::new(index.try_into().map_err(|_| format!(
                "Invalid negative sprite index {} for game object's sprite \"{}\" at absolute position {}",
                index, name.display(strings), start_position + chunk.abs_pos))?)),
        };
        let visible: bool = chunk.read_bool32()?;
        let mut managed: Option<bool> = None;
        if general_info.is_version_at_least(2022, 5, 0, 0) {
            managed = Some(chunk.read_bool32()?);
        }
        let solid: bool = chunk.read_bool32()?;
        let depth: i32 = chunk.read_i32()?;
        let persistent: bool = chunk.read_bool32()?;
        let parent_id: i32 = chunk.read_i32()?;
        let parent: Option<GMRef<GMGameObject>> = match parent_id {
            -100 => None,   // No parent
            -1 => Some(GMRef::new(game_objects_by_index.len())),    // Parent is Self
            _ => {
                let parent_id: usize = u32::try_from(parent_id)
                    .map_err(|_| format!("Invalid game object parent id {parent_id}"))? as usize;
                Some(GMRef::new(parent_id))
            },
        };
        let texture_mask: Option<GMRef<GMSprite>> = match chunk.read_i32()? {
            -1 => None,
            index => Some(GMRef::new(index.try_into().map_err(|_| format!(
                "Invalid negative sprite index {} for game object's texture mask \"{}\" at absolute position {}",
                index, name.display(strings), start_position + chunk.abs_pos))?)),
        };
        let uses_physics: bool = chunk.read_bool32()?;
        let is_sensor: bool = chunk.read_bool32()?;
        let collision_shape: u32 = chunk.read_u32()?;
        let collision_shape: GMGameObjectCollisionShape = collision_shape.try_into().map_err(|_| format!(
            "Invalid Collision Shape 0x{:04X} at position {} while parsing Game Object at position {} in chunk '{}'",
            collision_shape, chunk.cur_pos, start_position, chunk.name,
        ))?;
        let density: f32 = chunk.read_f32()?;
        let restitution: f32 = chunk.read_f32()?;
        let group: u32 = chunk.read_u32()?;
        let linear_damping: f32 = chunk.read_f32()?;
        let angular_damping: f32 = chunk.read_f32()?;
        let physics_shape_vertex_count: i32 = chunk.read_i32()?;
        let uses_physics_shape_vertex: bool = physics_shape_vertex_count != -1;
        let physics_shape_vertex_count: usize = if physics_shape_vertex_count < 0 {0} else {physics_shape_vertex_count as usize};
        let friction: f32 = chunk.read_f32()?;
        let awake: bool = chunk.read_bool32()?;
        let kinematic: bool = chunk.read_bool32()?;
        let mut physics_shape_vertices: Vec<(f32, f32)> = Vec::with_capacity(physics_shape_vertex_count);
        for _ in 0..physics_shape_vertex_count {
            let x: f32 = chunk.read_f32()?;
            let y: f32 = chunk.read_f32()?;
            physics_shape_vertices.push((x, y));
        }
        let events: Vec<Vec<GMGameObjectEvent>> = parse_game_object_events(chunk, strings)?;

        game_objects_by_index.push(GMGameObject {
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
        })
    }

    Ok(GMGameObjects {game_objects_by_index})
}


fn parse_game_object_events(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<Vec<GMGameObjectEvent>>, String> {
    let events_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(events_count);
    for _ in 0..events_count {
        start_positions.push(chunk.read_usize_pos()? - chunk.abs_pos);
    }

    let mut events: Vec<Vec<GMGameObjectEvent>> = Vec::with_capacity(events_count);

    for start_position in start_positions {
        chunk.cur_pos = start_position;
        // there's "events" like OnCreate, OnDestroy. for each "event type" there can be multiple event "instances" or "actions".
        let event: Vec<GMGameObjectEvent> = parse_game_object_event_instances(chunk, strings)?;
        events.push(event);
    }

    // chunk.file_index = old_position;
    Ok(events)
}


fn parse_game_object_event_instances(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMGameObjectEvent>, String> {
    let event_instances_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(event_instances_count);
    for _ in 0..event_instances_count {
        start_positions.push(chunk.read_usize_pos()? - chunk.abs_pos);
    }

    let old_position: usize = chunk.cur_pos;
    let mut events: Vec<GMGameObjectEvent> = Vec::with_capacity(event_instances_count);
    for start_position in start_positions {
        chunk.cur_pos = start_position;
        let subtype: u32 = chunk.read_u32()?;
        let actions: Vec<GMGameObjectEventAction> = parse_game_object_event_actions(chunk, strings)?;

        events.push(GMGameObjectEvent {
            subtype,
            actions,
        });
    }

    chunk.cur_pos = old_position;
    Ok(events)
}


fn parse_game_object_event_actions(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMGameObjectEventAction>, String> {
    let actions_count: usize = chunk.read_usize_count()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(actions_count);
    for _ in 0..actions_count {
        start_positions.push(chunk.read_usize_pos()? - chunk.abs_pos);
    }
    let old_position: usize = chunk.cur_pos;
    let mut actions: Vec<GMGameObjectEventAction> = Vec::with_capacity(actions_count);

    for start_position in start_positions {
        chunk.cur_pos = start_position;
        let lib_id: u32 = chunk.read_u32()?;
        let id: u32 = chunk.read_u32()?;
        let kind: u32 = chunk.read_u32()?;
        let use_relative: bool = chunk.read_bool32()?;
        let is_question: bool = chunk.read_bool32()?;
        let use_apply_to: bool = chunk.read_bool32()?;
        let exe_type: u32 = chunk.read_u32()?;
        let action_name: Option<GMRef<String>> = chunk.read_gm_string_optional(strings)?;
        let code_id: i32 = chunk.read_i32()?;
        let code: Option<GMRef<GMCode>> = if code_id == -1 { None } else { Some(GMRef::new(code_id as usize)) };
        let argument_count: u32 = chunk.read_u32()?;
        let who: i32 = chunk.read_i32()?;
        let relative: bool = chunk.read_bool32()?;
        let is_not: bool = chunk.read_bool32()?;
        let unknown_always_zero: u32 = chunk.read_u32()?;

        actions.push(GMGameObjectEventAction {
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

    chunk.cur_pos = old_position;
    Ok(actions)
}

