// SPDX-License-Identifier: GPL-3.0-only
pub mod event;

pub use self::event::Event;
pub use self::event::EventGroups;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::element_stub;
use crate::wad::elem::sprite::GMSprite;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMGameObjects {
    pub game_objects: Vec<Option<GMGameObject>>,
    pub exists: bool,
}

gm_named_list_chunk!(OBJT, GMGameObjects, GMGameObject, game_objects, nullable);

impl GMElement for GMGameObjects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let mut game_objects: Vec<Option<GMGameObject>> = vec![None; pointers.len()];

        for (i, pointer) in pointers.into_iter().enumerate() {
            if pointer == 0 {
                continue;
            }
            reader.assert_pos(pointer, "Game Object")?;

            let name: GMRef<String> = reader.read_gm_string()?;
            let sprite: GMRef<GMSprite> = reader.read_resource_by_id()?;

            let visible = reader.read_bool32()?;
            let mut managed: Option<bool> = None;
            if reader.general_info.version >= (2022, 5) {
                managed = Some(reader.read_bool32()?);
            }
            let solid = reader.read_bool32()?;
            let depth = reader.read_i32()?;
            let persistent = reader.read_bool32()?;

            let parent_id = reader.read_i32()?;
            let parent: GMRef<GMGameObject> = match parent_id {
                -100 => GMRef::none(), // No parent
                -1 => GMRef::from(i),  // Parent is Self
                n if n < 0 => bail!("Unexpected negative Parent ID {n}"),
                _ => GMRef::new(parent_id),
            };

            let texture_mask: GMRef<GMSprite> = reader.read_resource_by_id()?;

            let uses_physics = reader.read_bool32()?;
            let is_sensor = reader.read_bool32()?;
            let collision_shape: CollisionShape = reader.read_enum()?;
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

            let events = EventGroups::deserialize(reader).ctx("parsing game object events")?;

            game_objects[i] = Some(GMGameObject {
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
        let pointer_list_pos = builder.pos();
        for _ in 0..self.game_objects.len() {
            builder.write_u32(0);
        }

        for (gm_ref, game_object) in self.element_refs() {
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, gm_ref.index as usize)?;

            builder.write_gm_string(game_object.name)?;
            builder.write_resource_id(game_object.sprite);
            builder.write_bool32(game_object.visible);
            builder.write_if_ver(&game_object.managed, "Managed", (2022, 5))?;
            builder.write_bool32(game_object.solid);
            builder.write_i32(game_object.depth);
            builder.write_bool32(game_object.persistent);
            match game_object.parent {
                x if x.is_none() => builder.write_i32(-100), // No Parent
                obj_ref if obj_ref == gm_ref => builder.write_i32(-1), // Parent is Self
                obj_ref => builder.write_resource_id(obj_ref), // Normal Parent
            }
            builder.write_resource_id(game_object.texture_mask);
            builder.write_bool32(game_object.uses_physics);
            builder.write_bool32(game_object.is_sensor);
            builder.write_enum(game_object.collision_shape);
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
            game_object.events.serialize(builder)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMGameObject {
    /// The name of the game object.
    pub name: GMRef<String>,

    /// The sprite this game object uses.
    pub sprite: GMRef<GMSprite>,

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
    pub parent: GMRef<Self>,

    /// The texture mask this game object is using.
    pub texture_mask: GMRef<GMSprite>,

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
    pub events: EventGroups,
}
element_stub!(GMGameObject);

gm_enum!(CollisionShape {
    Circle = 0,
    Box = 1,
    Custom = 2,
});
