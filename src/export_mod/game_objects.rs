// use serde::{Deserialize, Serialize};
// use crate::deserialize::chunk_reading::GMRef;
// use crate::deserialize::code::GMCode;
// use crate::deserialize::game_objects::{GMGameObjectCollisionShape, GMGameObjectEvent, GMGameObjectEventAction};
// use crate::deserialize::sprites::GMSprite;
// use crate::export_mod::export::{convert_additions, ModExporter, ModRef};
// use crate::export_mod::ordered_list::DataChange;
// use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};
// 
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddGameObject {
//     pub name: ModRef,
//     pub sprite: Option<ModRef>,
//     pub visible: bool,
//     pub managed: Option<bool>,
//     pub solid: bool,
//     pub depth: i32,
//     pub persistent: bool,
//     pub parent_id: i32,
//     pub texture_mask: Option<ModRef>,   // Sprite ref
//     pub uses_physics: bool,
//     pub is_sensor: bool,
//     pub collision_shape: GMGameObjectCollisionShape,
//     pub density: f32,
//     pub restitution: f32,
//     pub group: u32,
//     pub linear_damping: f32,
//     pub angular_damping: f32,
//     pub friction: f32,
//     pub awake: bool,
//     pub kinematic: bool,
//     pub physics_shape_vertices: Vec<(f32, f32)>,
//     pub uses_physics_shape_vertex: bool,
//     pub events: Vec<Vec<AddGameObjectEvent>>,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddGameObjectEvent {
//     pub subtype: u32,
//     pub actions: Vec<AddGameObjectEventAction>,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddGameObjectEventAction {
//     pub code: Option<ModRef>,
// }
// 
// #[serde_with::skip_serializing_none]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct EditGameObject {
//     pub name: Option<ModRef>,
//     pub sprite: Option<Option<ModRef>>,
//     pub visible: Option<bool>,
//     pub managed: Option<Option<bool>>,
//     pub solid: Option<bool>,
//     pub depth: Option<i32>,
//     pub persistent: Option<bool>,
//     pub parent_id: Option<i32>,      // TODO replace this in near future with modref
//     pub texture_mask: Option<Option<ModRef>>,   // Sprite ref
//     pub uses_physics: Option<bool>,
//     pub is_sensor: Option<bool>,
//     pub collision_shape: Option<GMGameObjectCollisionShape>,
//     pub density: Option<f32>,
//     pub restitution: Option<f32>,
//     pub group: Option<u32>,
//     pub linear_damping: Option<f32>,
//     pub angular_damping: Option<f32>,
//     pub friction: Option<f32>,
//     pub awake: Option<bool>,
//     pub kinematic: Option<bool>,
//     pub physics_shape_vertices: Vec<(f32, f32)>,
//     pub uses_physics_shape_vertex: Option<bool>,
//     pub events: Vec<DataChange<Vec<DataChange<EditGameObjectEvent>>>>,  // TODO double order??
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct EditGameObjectEvent {
//     pub subtype: Option<u32>,
//     pub actions: EditUnorderedList<AddGameObjectEventAction, EditGameObjectEventAction>,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct EditGameObjectEventAction {
//     pub code: Option<Option<ModRef>>,
// }
// 
// impl ModExporter<'_, '_> {
//     pub fn export_game_objects(&self) -> Result<EditUnorderedList<AddGameObject, EditGameObject>, String> {
//         export_changes_unordered_list(
//             &self.original_data.game_objects.game_objects_by_index,
//             &self.modified_data.game_objects.game_objects_by_index,
//             |i| Ok(AddGameObject {
//                 name: self.convert_string_ref(i.name)?,
//                 sprite: self.convert_sprite_ref_opt(i.sprite)?,
//                 visible: i.visible,
//                 managed: i.managed,
//                 solid: i.solid,
//                 depth: i.depth,
//                 persistent: i.persistent,
//                 parent_id: i.parent_id,
//                 texture_mask: self.convert_sprite_ref_opt(i.texture_mask)?,
//                 uses_physics: i.uses_physics,
//                 is_sensor: i.is_sensor,
//                 collision_shape: i.collision_shape,
//                 density: i.density,
//                 restitution: i.restitution,
//                 group: i.group,
//                 linear_damping: i.linear_damping,
//                 angular_damping: i.angular_damping,
//                 friction: i.friction,
//                 awake: i.awake,
//                 kinematic: i.kinematic,
//                 physics_shape_vertices: i.physics_shape_vertices.clone(),
//                 uses_physics_shape_vertex: i.uses_physics_shape_vertex,
//                 // TODO i think the order of events actions is irrelevant? verify that anyways
//                 events: convert_additions(
//                     &i.events, |i| convert_additions(
//                         &i, |i| self.add_event(i)
//                     )
//                 )?,
//             }),
//             |o, m| Ok(EditGameObject {
//                 
//             })
//         )
//     }
//     
//     fn add_event(&self, i: &GMGameObjectEvent) -> Result<AddGameObjectEvent, String> {
//         Ok(AddGameObjectEvent {
//             subtype: i.subtype,
//             actions: convert_additions(&i.actions, |i| self.add_event_action(i))?,
//         })
//     }
//     
//     fn add_event_action(&self, i: &GMGameObjectEventAction) -> Result<AddGameObjectEventAction, String> {
//         Ok(AddGameObjectEventAction {
//             code: self.convert_code_ref_opt(i.code)?,
//         })
//     }
// }
// 
