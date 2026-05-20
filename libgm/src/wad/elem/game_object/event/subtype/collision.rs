use crate::prelude::*;
use crate::wad::elem::game_object::GMGameObject;
use crate::wad::elem::game_object::event::EventSubtype;

/// Triggered when this game object instance collides
/// with another game object (any instance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Collision {
    /// The other game object to check collision against.
    ///
    /// This becomes the [`InstanceType::Other`] context.
    ///
    /// [`InstanceType::Other`]: crate::gml::instruction::InstanceType::Other
    pub object: GMRef<GMGameObject>,
}

impl EventSubtype for Collision {
    fn parse(object_id: u32) -> Result<Self> {
        Ok(Self::new(GMRef::new(object_id)))
    }

    fn build(self) -> u32 {
        self.object.index
    }
}

impl Collision {
    /// Creates a new [`Collision`] with the given game object reference.
    #[must_use]
    pub const fn new(game_object_ref: GMRef<GMGameObject>) -> Self {
        Self { object: game_object_ref }
    }
}

impl From<GMRef<GMGameObject>> for Collision {
    fn from(game_object_ref: GMRef<GMGameObject>) -> Self {
        Self::new(game_object_ref)
    }
}

impl From<Collision> for GMRef<GMGameObject> {
    fn from(collision: Collision) -> Self {
        collision.object
    }
}
