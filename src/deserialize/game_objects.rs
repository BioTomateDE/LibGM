use num_enum::TryFromPrimitive;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::{GMStringRef, GMStrings};

#[derive(Debug, Clone)]
pub struct GMGameObject {
    pub name: GMStringRef,
    pub sprite_index: i32,
    pub visible: bool,
    pub managed: Option<bool>,
    pub solid: bool,
    pub depth: i32,
    pub persistent: bool,
    pub parent_id: i32,
    pub texture_mask_id: i32,                   // {!!} change type to sprite ref
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
    pub events: Vec<Vec<GMGameObjectEvent>>,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum GMGameObjectCollisionShape {
    Circle = 0,
    Box = 1,
    Custom = 2,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GMGameObjectRef {
    index: usize,
}
impl GMGameObjectRef {
    pub fn resolve<'a>(&self, game_objects: &'a GMGameObjects) -> Result<&'a GMGameObject, String> {
        match game_objects.game_objects_by_index.get(self.index) {
            Some(object) => Ok(object),
            None => Err(format!(
                "Could not resolve game object with index {} in list with length {}.",
                self.index, game_objects.game_objects_by_index.len(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GMGameObjects {
    pub game_objects_by_index: Vec<GMGameObject>,
}
impl GMGameObjects {
    pub fn get_game_object_by_index(&self, index: usize) -> Option<GMGameObjectRef> {
        if index >= self.game_objects_by_index.len() {
            return None;
        }
        Some(GMGameObjectRef {index})
    }
}


#[derive(Debug, Clone)]
pub struct GMGameObjectEvent {
    pub subtype: u32,
    pub actions: Vec<GMGameObjectEventAction>,
}

#[derive(Debug, Clone)]
pub struct GMGameObjectEventAction {
    pub lib_id: u32,
    pub id: u32,
    pub kind: u32,
    pub use_relative: bool,
    pub is_question: bool,
    pub use_apply_to: bool,
    pub exe_type: u32,
    pub action_name: GMStringRef,
    pub code_id: i32,                   // {!!} change type to code ref
    pub argument_count: u32,
    pub who: i32,
    pub relative: bool,
    pub is_not: bool,
    pub unknown_always_zero: u32,
}


pub fn parse_chunk_objt(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMGameObjects, String> {
    chunk.file_index = 0;
    let game_objects_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(game_objects_count);
    for _ in 0..game_objects_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut game_objects_by_index: Vec<GMGameObject> = Vec::with_capacity(game_objects_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
        let name: GMStringRef = chunk.read_gm_string(strings)?;
        let sprite_index: i32 = chunk.read_i32()?;        // TODO usize, sprite ref
        let visible: bool = chunk.read_u32()? != 0;
        let mut managed: Option<bool> = None;
        if general_info.is_version_at_least(2022, 5, 0, 0) {
            managed = Some(chunk.read_u32()? != 0);
        }
        let solid: bool = chunk.read_u32()? != 0;
        let depth: i32 = chunk.read_i32()?;
        let persistent: bool = chunk.read_u32()? != 0;
        let parent_id: i32 = chunk.read_i32()?;         // TODO usize, object ref  | parent can be: -100 (undefined), -2 (other [not here]), or -1 (self)
        let texture_mask_id: i32 = chunk.read_i32()?;   // TODO sprite ref
        let uses_physics: bool = chunk.read_u32()? != 0;
        let is_sensor: bool = chunk.read_u32()? != 0;
        let collision_shape: u32 = chunk.read_u32()?;
        let collision_shape: GMGameObjectCollisionShape = match collision_shape.try_into() {
            Ok(shape) => shape,
            Err(_) => return Err(format!(
                "Invalid Collision Shape 0x{:04X} at position {} while parsing Game Object at position {} in chunk '{}'.",
                collision_shape, chunk.file_index, start_position, chunk.name,
            )),
        };
        let density: f32 = chunk.read_f32()?;
        let restitution: f32 = chunk.read_f32()?;
        let group: u32 = chunk.read_u32()?;
        let linear_damping: f32 = chunk.read_f32()?;
        let angular_damping: f32 = chunk.read_f32()?;
        let physics_shape_vertex_count: i32 = chunk.read_i32()?;
        let physics_shape_vertex_count: usize = if physics_shape_vertex_count < 0 {0} else {physics_shape_vertex_count as usize};
        let friction: f32 = chunk.read_f32()?;
        let awake: bool = chunk.read_u32()? != 0;
        let kinematic: bool = chunk.read_u32()? != 0;
        let mut physics_shape_vertices: Vec<(f32, f32)> = Vec::with_capacity(physics_shape_vertex_count);
        for _ in 0..physics_shape_vertex_count {
            let x: f32 = chunk.read_f32()?;
            let y: f32 = chunk.read_f32()?;
            physics_shape_vertices.push((x, y));
        }
        let events: Vec<Vec<GMGameObjectEvent>> = parse_game_object_events(chunk, strings)?;
        // println!("\n\n\n############################ {} ##################################", name.resolve(strings)?);
        // for i in &events {
        //     for j in i {
        //         j.print(strings)?;
        //     }
        // }

        game_objects_by_index.push(GMGameObject {
            name,
            sprite_index,
            visible,
            managed,
            solid,
            depth,
            persistent,
            parent_id,
            texture_mask_id,
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
            events,
        })
    }

    Ok(GMGameObjects {game_objects_by_index})
}


fn parse_game_object_events(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<Vec<GMGameObjectEvent>>, String> {
    let events_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(events_count);
    for _ in 0..events_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut events: Vec<Vec<GMGameObjectEvent>> = Vec::with_capacity(events_count);

    for start_position in start_positions {
        chunk.file_index = start_position;
        // there's "events" like OnCreate, OnDestroy. for each "event type" there can be multiple event "instances" or "actions".
        let event: Vec<GMGameObjectEvent> = parse_game_object_event_instances(chunk, strings)?;
        events.push(event);
    }

    // chunk.file_index = old_position;
    Ok(events)
}


fn parse_game_object_event_instances(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMGameObjectEvent>, String> {
    let event_instances_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(event_instances_count);
    for _ in 0..event_instances_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let old_position: usize = chunk.file_index;
    let mut events: Vec<GMGameObjectEvent> = Vec::with_capacity(event_instances_count);
    for start_position in start_positions {
        chunk.file_index = start_position;
        let subtype: u32 = chunk.read_u32()?;
        let actions: Vec<GMGameObjectEventAction> = parse_game_object_events_actions(chunk, strings)?;

        events.push(GMGameObjectEvent {
            subtype,
            actions,
        });
    }

    chunk.file_index = old_position;
    Ok(events)
}


fn parse_game_object_events_actions(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMGameObjectEventAction>, String> {
    let actions_count: usize = chunk.read_usize()?;
    let mut start_positions: Vec<usize> = Vec::with_capacity(actions_count);
    for _ in 0..actions_count {
        start_positions.push(chunk.read_usize()? - chunk.abs_pos);
    }
    let old_position: usize = chunk.file_index;
    let mut actions: Vec<GMGameObjectEventAction> = Vec::with_capacity(actions_count);

    for start_position in start_positions {
        chunk.file_index = start_position;
        let lib_id = chunk.read_u32()?;
        let id = chunk.read_u32()?;
        let kind = chunk.read_u32()?;
        let use_relative = chunk.read_u32()? != 0;
        let is_question = chunk.read_u32()? != 0;
        let use_apply_to = chunk.read_u32()? != 0;
        let exe_type = chunk.read_u32()?;
        let action_name = chunk.read_gm_string(strings)?;
        let code_id = chunk.read_i32()?;                    // {!!} replace type with code ref
        let argument_count = chunk.read_u32()?;
        let who = chunk.read_i32()?;
        let relative = chunk.read_u32()? != 0;
        let is_not = chunk.read_u32()? != 0;
        let unknown_always_zero = chunk.read_u32()?;

        actions.push(GMGameObjectEventAction {
            lib_id,
            id,
            kind,
            use_relative,
            is_question,
            use_apply_to,
            exe_type,
            action_name,
            code_id,
            argument_count,
            who,
            relative,
            is_not,
            unknown_always_zero,
        })
    }

    chunk.file_index = old_position;
    Ok(actions)
}

