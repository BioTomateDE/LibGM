// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::GMEnum;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::elem::game_object::event::EventSubtype;

gm_enum!(
/// Triggered on every game step (aka. frame).
///
/// The call order is as follows:
/// * [`Step::Step`]
/// * [`Step::BeginStep`]
/// * [`Self::EndStep`]
Step {
    /// Normal step event.
    Step = 0,

    /// The begin step event.
    BeginStep = 1,

    /// The end step event.
    EndStep = 2,
});

impl EventSubtype for Step {
    fn parse(subtype: i32) -> Result<Self> {
        Self::from_i32(subtype)
    }

    fn build(self) -> i32 {
        self.as_i32()
    }
}
