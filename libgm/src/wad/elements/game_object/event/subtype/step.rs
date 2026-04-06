use macros::num_enum;

use crate::prelude::*;
use crate::util::init::num_enum_from;
use crate::wad::elements::game_object::event::EventSubtype;

/// The subtype for [`EventType::Step`].
///
/// The call order is as follows:
/// * [`Step::Step`]
/// * [`Step::BeginStep`]
/// * [`Step::EndStep`]
#[num_enum(u32)]
pub enum Step {
    /// Normal step event.
    Step = 0,

    /// The begin step event.
    BeginStep = 1,

    /// The end step event.
    EndStep = 2,
}

impl EventSubtype for Step {
    fn parse(subtype: u32) -> Result<Self> {
        num_enum_from(subtype)
    }

    fn build(self) -> u32 {
        self.into()
    }
}
