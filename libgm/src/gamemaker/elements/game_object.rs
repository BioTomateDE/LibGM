pub mod event;

pub use event::{Event, Events};
use macros::{named_list_chunk, num_enum};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, element_stub, sprite::GMSprite},
        reference::GMRef,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
    util::init::{num_enum_from, vec_with_capacity},
};

#[named_list_chunk("OBJT")]
pub struct GMGameObjects {
    pub game_objects: Vec<GMGameObject>,
    pub exists: bool,
}

impl GMElement for GMGameObjects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let mut game_objects: Vec<GMGameObject> = Vec::with_capacity(pointers.len());

        for pointer in pointers {
            reader.assert_pos(pointer, "Game Object")?;

            let name: String = reader.read_gm_string()?;
            let sprite: Option<GMRef<GMSprite>> = reader.read_resource_by_id_opt()?;

            let visible = reader.read_bool32()?;
            let mut managed: Option<bool> = None;
            if reader.general_info.is_version_at_least((2022, 5)) {
                managed = Some(reader.read_bool32()?);
            }
            let solid = reader.read_bool32()?;
            let depth = reader.read_i32()?;
            let persistent = reader.read_bool32()?;

            let parent_id = reader.read_i32()?;
            let parent: Option<GMRef<GMGameObject>> = match parent_id {
                -100 => None,                          // No parent
                -1 => Some(game_objects.len().into()), // Parent is Self
                _ => Some(GMRef::new(parent_id as u32)),
            };

            let texture_mask: Option<GMRef<GMSprite>> = reader.read_resource_by_id_opt()?;

            let uses_physics = reader.read_bool32()?;
            let is_sensor = reader.read_bool32()?;
            let collision_shape: CollisionShape = num_enum_from(reader.read_i32()?)?;
            let density = reader.read_f32()?;
            let restitution = reader.read_f32()?;
            let group = reader.read_u32()?;
            let linear_damping = reader.read_f32()?;
            let angular_damping = reader.read_f32()?;
            let physics_shape_vertex_count = reader.read_count("Physics Shape Vertex Count")?;
            let friction = reader.read_f32()?;
            let awake = reader.read_bool32()?;
            let kinematic = reader.read_bool32()?;
            let mut physics_shape_vertices: Vec<(f32, f32)> =
                vec_with_capacity(physics_shape_vertex_count)?;
            for _ in 0..physics_shape_vertex_count {
                let x = reader.read_f32()?;
                let y = reader.read_f32()?;
                physics_shape_vertices.push((x, y));
            }
            let events: Vec<Events> = reader.read_pointer_list()?;

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
                events,
            });
        }

        Ok(Self { game_objects, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.game_objects.len())?;
        let pointer_list_pos: usize = builder.len();
        for _ in 0..self.game_objects.len() {
            builder.write_u32(0xDEAD_C0DE);
        }

        for (i, game_object) in self.game_objects.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_pos + 4 * i)?;

            builder.write_gm_string(&game_object.name);
            builder.write_resource_id_opt(game_object.sprite);
            builder.write_bool32(game_object.visible);
            game_object
                .managed
                .serialize_if_gm_ver(builder, "Managed", (2022, 5))?;
            builder.write_bool32(game_object.solid);
            builder.write_i32(game_object.depth);
            builder.write_bool32(game_object.persistent);
            match game_object.parent {
                None => builder.write_i32(-100), // No Parent
                Some(obj_ref) if obj_ref.index == i as u32 => {
                    builder.write_i32(-1);
                }, // Parent is Self
                Some(obj_ref) => builder.write_resource_id(obj_ref), // Normal Parent
            }
            builder.write_resource_id_opt(game_object.texture_mask);
            builder.write_bool32(game_object.uses_physics);
            builder.write_bool32(game_object.is_sensor);
            builder.write_i32(game_object.collision_shape.into());
            builder.write_f32(game_object.density);
            builder.write_f32(game_object.restitution);
            builder.write_u32(game_object.group);
            builder.write_f32(game_object.linear_damping);
            builder.write_f32(game_object.angular_damping);
            builder.write_usize(game_object.physics_shape_vertices.len())?; // "new meaning" according to UTMT?
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObject {
    /// The name of the game object.
    pub name: String,
    /// The sprite this game object uses.
    pub sprite: Option<GMRef<GMSprite>>,
    /// Whether the game object is visible.
    pub visible: bool,
    /// Introduced in 2022.5.
    pub managed: Option<bool>,
    /// Whether the game object is solid.
    pub solid: bool,
    /// The depth level of the game object.
    pub depth: i32,
    /// Whether the game object is persistent.
    pub persistent: bool,
    /// The parent game object this is inheriting from.
    pub parent: Option<GMRef<Self>>,
    /// The texture mask this game object is using.
    pub texture_mask: Option<GMRef<GMSprite>>,
    /// Whether this object uses GameMaker physics.
    pub uses_physics: bool,
    /// Whether this game object should act as a sensor fixture.
    pub is_sensor: bool,
    /// The collision shape the game object should use.
    pub collision_shape: CollisionShape,
    /// The physics density of the game object.
    pub density: f32,
    /// The physics restitution of the game object.
    pub restitution: f32,
    /// The physics collision group this game object belongs to.
    pub group: u32,
    /// The physics linear damping this game object uses.
    pub linear_damping: f32,
    /// The physics angular damping this game object uses.
    pub angular_damping: f32,
    /// The physics friction this game object uses.
    pub friction: f32,
    /// Whether this game object should start awake in the physics simulation.
    pub awake: bool,
    /// Whether this game object is kinematic.
    pub kinematic: bool,
    /// The vertices used for a [`CollisionShape::Custom`].
    pub physics_shape_vertices: Vec<(f32, f32)>,
    /// All the events that this game object has.
    pub events: Vec<Events>,
}
element_stub!(GMGameObject);

#[num_enum(i32)]
pub enum CollisionShape {
    Circle = 0,
    Box = 1,
    Custom = 2,
}
