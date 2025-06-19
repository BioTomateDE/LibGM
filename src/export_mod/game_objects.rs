use serde::{Deserialize, Serialize};
use crate::deserialize::game_objects::{GMGameObjectCollisionShape, GMGameObjectEvent, GMGameObjectEventAction};
use crate::export_mod::export::{convert_additions, edit_field, edit_field_convert, edit_field_convert_option, ModExporter, ModRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};

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
    pub events: Vec<Vec<AddGameObjectEvent>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGameObjectEvent {
    pub subtype: u32,
    pub actions: Vec<AddGameObjectEventAction>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGameObjectEventAction {
    pub code: Option<ModRef>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGameObject {
    pub name: Option<ModRef>,
    pub sprite: Option<Option<ModRef>>,
    pub visible: Option<bool>,
    pub managed: Option<Option<bool>>,
    pub solid: Option<bool>,
    pub depth: Option<i32>,
    pub persistent: Option<bool>,
    pub parent: Option<Option<ModRef>>,   // GameObject ref
    pub texture_mask: Option<Option<ModRef>>,   // Sprite ref
    pub uses_physics: Option<bool>,
    pub is_sensor: Option<bool>,
    pub collision_shape: Option<GMGameObjectCollisionShape>,
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
    pub events: Vec<EditUnorderedList<AddGameObjectEvent, EditGameObjectEvent>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGameObjectEvent {
    pub subtype: Option<u32>,
    pub actions: EditUnorderedList<AddGameObjectEventAction, EditGameObjectEventAction>,    // not sure if action order matters
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGameObjectEventAction {
    pub code: Option<Option<ModRef>>,
}

impl ModExporter<'_, '_> {
    pub fn export_game_objects(&self) -> Result<EditUnorderedList<AddGameObject, EditGameObject>, String> {
        export_changes_unordered_list(
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
                collision_shape: i.collision_shape,
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
                // TODO i think the order of events actions is irrelevant? verify that anyways
                events: convert_additions(
                    &i.events, |i| convert_additions(
                        &i, |i| self.add_event(i)
                    )
                )?,
            }),
            |o, m| Ok(EditGameObject {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(&r))?,
                sprite: edit_field_convert_option(&o.sprite, &m.sprite, |r| self.convert_sprite_ref(r))?,
                visible: edit_field(&o.visible, &m.visible),
                managed: edit_field(&o.managed, &m.managed),
                solid: edit_field(&o.solid, &m.solid),
                depth: edit_field(&o.depth, &m.depth),
                persistent: edit_field(&o.persistent, &m.persistent),
                parent: edit_field_convert_option(&o.parent, &m.parent, |r| self.convert_game_object_ref(r))?,
                texture_mask: edit_field_convert_option(&o.texture_mask, &m.texture_mask, |r| self.convert_sprite_ref(r))?,
                uses_physics: edit_field(&o.uses_physics, &m.uses_physics),
                is_sensor: edit_field(&o.is_sensor, &m.is_sensor),
                collision_shape: edit_field(&o.collision_shape, &m.collision_shape),
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
            false,
        )
    }
    
    fn add_event(&self, i: &GMGameObjectEvent) -> Result<AddGameObjectEvent, String> {
        Ok(AddGameObjectEvent {
            subtype: i.subtype,
            actions: convert_additions(&i.actions, |i| self.add_event_action(i))?,
        })
    }
    
    fn add_event_action(&self, i: &GMGameObjectEventAction) -> Result<AddGameObjectEventAction, String> {
        Ok(AddGameObjectEventAction {
            code: self.convert_code_ref_opt(&i.code)?,
        })
    }
    
    fn edit_events(
        &self,
        original_events: &Vec<Vec<GMGameObjectEvent>>,
        modified_events: &Vec<Vec<GMGameObjectEvent>>,
    ) -> Result<Vec<EditUnorderedList<AddGameObjectEvent, EditGameObjectEvent>>, String> {
        let mut edited_events = Vec::with_capacity(modified_events.len());
        for (i, modified_event) in modified_events.iter().enumerate() {
            // outer event list (for event "types") should be the same length (always 12 or 14 or whatever).
            // if different; probably different gamemaker version; default to empty inner event list.
            let original_event: &[GMGameObjectEvent] = match original_events.get(i) {
                Some(i) => i.as_slice(),
                None => &[],
            };
            if modified_event == original_event {
                continue
            }
            
            edited_events.push(export_changes_unordered_list(
                original_event,
                modified_event,
                |i| Ok(AddGameObjectEvent {
                    subtype: i.subtype,
                    actions: convert_additions(&i.actions, |i| self.add_event_action(i))?,
                }),
                |o, m| Ok(EditGameObjectEvent {
                    subtype: edit_field(&o.subtype, &m.subtype),
                    actions: export_changes_unordered_list(
                        &o.actions,
                        &m.actions,
                        |i| self.add_event_action(i),
                        |o, m| self.edit_event_action(o, m),
                        false,
                    )?,
                }),
                true,
            )?);
        }
        Ok(edited_events)
    }
    
    fn edit_event_action(&self, o: &GMGameObjectEventAction, m: &GMGameObjectEventAction) -> Result<EditGameObjectEventAction, String> {
        Ok(EditGameObjectEventAction {
            code: edit_field_convert_option(&o.code, &m.code, |r| self.convert_code_ref(r))?,
        })
    }
}


