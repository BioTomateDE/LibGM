use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::game_objects::{GMGameObjectCollisionShape, GMGameObjectEvent, GMGameObjectEvents};
use crate::modding::export::{convert_additions, edit_field, edit_field_convert, edit_field_convert_option, ModExporter, ModRef};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange, DataChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGameObject {
    pub name: ModRef,
    pub sprite: Option<ModRef>,
    pub visible: bool,
    pub managed: Option<bool>,
    pub solid: bool,
    pub depth: i32,
    pub persistent: bool,
    pub parent: Option<ModRef>,     // GameObject ref
    pub texture_mask: Option<ModRef>,   // Sprite ref
    pub uses_physics: bool,
    pub is_sensor: bool,
    pub collision_shape: ModGameObjectCollisionShape,
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
    pub events: Vec<Vec<AddGameObjectEvent>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGameObjectEvent {
    pub subtype: u32,
    pub actions: Vec<ModRef>,   // code ref
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGameObject {
    pub name: Option<ModRef>,
    pub sprite: Option<ModRef>,
    pub visible: Option<bool>,
    pub managed: Option<bool>,
    pub solid: Option<bool>,
    pub depth: Option<i32>,
    pub persistent: Option<bool>,
    pub parent: Option<Option<ModRef>>,   // GameObject ref
    pub texture_mask: Option<Option<ModRef>>,   // Sprite ref
    pub uses_physics: Option<bool>,
    pub is_sensor: Option<bool>,
    pub collision_shape: Option<ModGameObjectCollisionShape>,
    pub density: Option<f32>,
    pub restitution: Option<f32>,
    pub group: Option<u32>,
    pub linear_damping: Option<f32>,
    pub angular_damping: Option<f32>,
    pub friction: Option<f32>,
    pub awake: Option<bool>,
    pub kinematic: Option<bool>,
    pub physics_shape_vertices: Option<Vec<(f32, f32)>>,
    pub uses_physics_shape_vertex: Option<bool>,
    pub events: Vec<Vec<DataChange<AddGameObjectEvent, EditGameObjectEvent>>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGameObjectEvent {
    pub subtype: Option<u32>,
    pub actions: Vec<DataChange<ModRef, ModRef>>,   // code ref
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(u32)]
pub enum ModGameObjectCollisionShape {
    Circle = 0,
    Box = 1,
    Custom = 2,
}

impl ModExporter<'_, '_> {
    pub fn export_game_objects(&self) -> Result<Vec<DataChange<AddGameObject, EditGameObject>>, String> {
        export_changes_ordered_list(
            &self.original_data.game_objects.game_objects,
            &self.modified_data.game_objects.game_objects,
            |i| Ok(AddGameObject {
                name: self.convert_string_ref(&i.name)?,
                sprite: self.convert_sprite_ref_opt(&i.sprite)?,
                visible: i.visible,
                managed: i.managed,
                solid: i.solid,
                depth: i.depth,
                persistent: i.persistent,
                parent: self.convert_game_object_ref_opt(&i.parent)?,
                texture_mask: self.convert_sprite_ref_opt(&i.texture_mask)?,
                uses_physics: i.uses_physics,
                is_sensor: i.is_sensor,
                collision_shape: convert_collision_shape(i.collision_shape),
                density: i.density,
                restitution: i.restitution,
                group: i.group,
                linear_damping: i.linear_damping,
                angular_damping: i.angular_damping,
                friction: i.friction,
                awake: i.awake,
                kinematic: i.kinematic,
                physics_shape_vertices: i.physics_shape_vertices.clone(),
                uses_physics_shape_vertex: i.uses_physics_shape_vertex,
                events: i.events.iter().map(|i| 
                    convert_additions(&i.events, |i| self.add_event(i)
                )).collect::<Result<Vec<_>, String>>()?,
            }),
            |o, m| Ok(EditGameObject {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(&r))?,
                sprite: edit_field_convert_option(&o.sprite, &m.sprite, |r| self.convert_sprite_ref(r))?.flatten(),
                visible: edit_field(&o.visible, &m.visible),
                managed: edit_field(&o.managed, &m.managed).flatten(),
                solid: edit_field(&o.solid, &m.solid),
                depth: edit_field(&o.depth, &m.depth),
                persistent: edit_field(&o.persistent, &m.persistent),
                parent: edit_field_convert_option(&o.parent, &m.parent, |r| self.convert_game_object_ref(r))?,
                texture_mask: edit_field_convert_option(&o.texture_mask, &m.texture_mask, |r| self.convert_sprite_ref(r))?,
                uses_physics: edit_field(&o.uses_physics, &m.uses_physics),
                is_sensor: edit_field(&o.is_sensor, &m.is_sensor),
                collision_shape: edit_field(&convert_collision_shape(o.collision_shape), &convert_collision_shape(m.collision_shape)),
                density: edit_field(&o.density, &m.density),
                restitution: edit_field(&o.restitution, &m.restitution),
                group: edit_field(&o.group, &m.group),
                linear_damping: edit_field(&o.linear_damping, &m.linear_damping),
                angular_damping: edit_field(&o.angular_damping, &m.angular_damping),
                friction: edit_field(&o.friction, &m.friction),
                awake: edit_field(&o.awake, &m.awake),
                kinematic: edit_field(&o.kinematic, &m.kinematic),
                physics_shape_vertices: edit_field(&o.physics_shape_vertices, &m.physics_shape_vertices),
                uses_physics_shape_vertex: edit_field(&o.uses_physics_shape_vertex, &m.uses_physics_shape_vertex),
                events: self.edit_events(&o.events, &m.events)?,
            }),
        )
    }
    
    fn add_event(&self, i: &GMGameObjectEvent) -> Result<AddGameObjectEvent, String> {
        Ok(AddGameObjectEvent {
            subtype: i.subtype,
            actions: convert_additions(&i.actions, |i| Ok(i.code))?
                .into_iter().filter_map(|i| self.convert_code_ref_opt(&i).transpose()).collect::<Result<Vec<_>, String>>()?,
        })
    }
    
    fn edit_event(&self, o: &GMGameObjectEvent, m: &GMGameObjectEvent) -> Result<EditGameObjectEvent, String> {
        Ok(EditGameObjectEvent {
            subtype: edit_field(&o.subtype, &m.subtype),
            actions: export_changes_ordered_list(
                &o.actions,
                &m.actions,
                |i| self.convert_code_ref_opt(&i.code),
                |_, m| self.convert_code_ref_opt(&m.code),
            )?.flatten()
        })
    }
    
    fn edit_events(
        &self,
        original_events: &Vec<GMGameObjectEvents>,
        modified_events: &Vec<GMGameObjectEvents>,
    ) -> Result<Vec<Vec<DataChange<AddGameObjectEvent, EditGameObjectEvent>>>, String> {
        let mut edited_events = Vec::with_capacity(modified_events.len());
        for (i, modified_event) in modified_events.iter().enumerate() {
            // outer event list (for event "types") should be the same length (always 12 or 14 or whatever).
            // if different; probably different gamemaker version; default to empty inner event list.
            let original_event: &[GMGameObjectEvent] = match original_events.get(i) {
                Some(i) => i.events.as_slice(),
                None => &[],
            };
            if modified_event.events == original_event {
                continue
            }
            
            edited_events.push(export_changes_ordered_list(
                original_event,
                &modified_event.events,
                |i| self.add_event(i),
                |o, m| self.edit_event(o, m),
            )?);
        }
        Ok(edited_events)
    }
}


fn convert_collision_shape(i: GMGameObjectCollisionShape) -> ModGameObjectCollisionShape {
    match i {
        GMGameObjectCollisionShape::Circle => ModGameObjectCollisionShape::Circle,
        GMGameObjectCollisionShape::Box => ModGameObjectCollisionShape::Box,
        GMGameObjectCollisionShape::Custom => ModGameObjectCollisionShape::Custom,
    }
}

