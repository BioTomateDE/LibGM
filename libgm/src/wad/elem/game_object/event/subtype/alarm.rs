// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::game_object::event::EventSubtype;

/// Triggered when a user-set alarm reaches 0.
///
/// An alarm event type which can be triggered modifying
/// the builtin `alarm` variable (array) in other scripts.
///
/// This is simply an alarm array index `0..12`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Alarm {
    pub index: u8,
}

impl EventSubtype for Alarm {
    fn parse(index: i32) -> Result<Self> {
        Self::new(index)
    }

    fn build(self) -> i32 {
        self.index as i32
    }
}

impl Alarm {
    /// Creates a new [`Alarm`] with the given Alarm ID.
    ///
    /// This function will fail for `index >= 12`.
    pub fn new(index: i32) -> Result<Self> {
        if index < 0 || index >= 12 {
            bail!("Alarm index must between 0 and 12; got {index}");
        }
        Ok(Self { index: index as u8 })
    }
}
