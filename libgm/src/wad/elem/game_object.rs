// SPDX-License-Identifier: GPL-3.0-only
pub mod event;

pub use self::event::Event;
pub use self::event::EventGroups;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::GMVersion;
use crate::wad::chunk::gm_asset_list_chunk;
use crate::wad::elem::GMComplexElement;
use crate::wad::elem::GMElement;
use crate::wad::elem::sprite::Sprite;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GameObjects {
    pub elems: Vec<Option<GameObject>>,
}

gm_asset_list_chunk!(OBJT, GameObjects, GameObject);

impl GMElement for GameObjects {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let mut elems: Vec<Option<GameObject>> = vec![None; pointers.len()];

        for (i, pointer) in pointers.into_iter().enumerate() {
            if pointer == 0 {
                continue;
            }
            reader.assert_pos(pointer, "Game Object")?;
            let object = reader.read_elem_ctx(GMRef::from(i))?;
            elems[i] = Some(object);
        }

        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_usize(self.elems.len())?;
        let pointer_list_pos = writer.pos();
        for _ in 0..self.elems.len() {
            writer.write_u32(0);
        }

        for (gm_ref, obj) in self.element_refs() {
            writer.patch_pointer_list(pointer_list_pos, gm_ref.index as usize)?;
            writer.write_elem_ctx(obj, gm_ref)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
    /// The name of the game object.
    pub name: GMRef<String>,

    /// The sprite this game object uses.
    pub sprite: GMRef<Sprite>,

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
    pub texture_mask: GMRef<Sprite>,

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

impl GMComplexElement for GameObject {
    /// Current element ID
    type ReadContext = GMRef<Self>;
    /// Current element ID
    type WriteContext = GMRef<Self>;

    fn deserialize_complex(reader: &mut DataReader, self_ref: GMRef<Self>) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let sprite: GMRef<Sprite> = reader.read_resource_by_id()?;

        let visible = reader.read_bool32()?;
        let mut managed: Option<bool> = None;
        if reader.version >= GMVersion::GM2022_5 {
            managed = Some(reader.read_bool32()?);
        }
        let solid = reader.read_bool32()?;
        let depth = reader.read_i32()?;
        let persistent = reader.read_bool32()?;

        let parent_id = reader.read_i32()?;
        let parent: GMRef<Self> = match parent_id {
            -100 => GMRef::none(), // No parent
            -1 => self_ref,        // Parent is Self
            n if n < 0 => bail!("Unexpected negative Parent ID {n}"),
            _ => GMRef::new(parent_id),
        };

        let texture_mask: GMRef<Sprite> = reader.read_resource_by_id()?;

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

        Ok(Self {
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
        })
    }

    fn serialize_complex(&self, writer: &mut DataWriter, self_ref: GMRef<Self>) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_resource_id(self.sprite);
        writer.write_bool32(self.visible);
        writer.write_if_ver(&self.managed, "Managed", GMVersion::GM2022_5)?;
        writer.write_bool32(self.solid);
        writer.write_i32(self.depth);
        writer.write_bool32(self.persistent);
        match self.parent {
            x if x.is_none() => writer.write_i32(-100), // No Parent
            obj_ref if obj_ref == self_ref => writer.write_i32(-1), // Parent is Self
            obj_ref => writer.write_resource_id(obj_ref), // Normal Parent
        }
        writer.write_resource_id(self.texture_mask);
        writer.write_bool32(self.uses_physics);
        writer.write_bool32(self.is_sensor);
        writer.write_enum(self.collision_shape);
        writer.write_f32(self.density);
        writer.write_f32(self.restitution);
        writer.write_u32(self.group);
        writer.write_f32(self.linear_damping);
        writer.write_f32(self.angular_damping);
        writer.write_usize(self.physics_shape_vertices.len())?; // "new meaning" according to UTMT?
        writer.write_f32(self.friction);
        writer.write_bool32(self.awake);
        writer.write_bool32(self.kinematic);
        for (x, y) in &self.physics_shape_vertices {
            writer.write_f32(*x);
            writer.write_f32(*y);
        }
        writer.write_elem(&self.events)?;
        Ok(())
    }
}

gm_enum!(CollisionShape {
    Circle = 0,
    Box = 1,
    Custom = 2,
});
