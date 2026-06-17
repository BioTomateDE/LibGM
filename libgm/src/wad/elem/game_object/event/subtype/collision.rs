// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::game_object::GameObject;
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
    pub object: GMRef<GameObject>,
}

impl EventSubtype for Collision {
    fn parse(object_id: i32) -> Result<Self> {
        Ok(Self::new(GMRef::new(object_id)))
    }

    fn build(self) -> i32 {
        self.object.index
    }
}

impl Collision {
    /// Creates a new [`Collision`] with the given game object reference.
    #[must_use]
    pub const fn new(game_object_ref: GMRef<GameObject>) -> Self {
        Self { object: game_object_ref }
    }
}

impl From<GMRef<GameObject>> for Collision {
    fn from(game_object_ref: GMRef<GameObject>) -> Self {
        Self::new(game_object_ref)
    }
}

impl From<Collision> for GMRef<GameObject> {
    fn from(collision: Collision) -> Self {
        collision.object
    }
}
