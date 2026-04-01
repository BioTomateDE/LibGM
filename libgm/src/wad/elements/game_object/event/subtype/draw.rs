use macros::num_enum;

use crate::{
    prelude::*, util::init::num_enum_from, wad::elements::game_object::event::EventSubtype,
};

/// The subtype for [`EventType::Draw`].
#[num_enum(u32)]
pub enum Draw {
    /// Normal draw event.
    Draw = 0,

    /// The draw GUI event.
    DrawGUI = 64,

    /// The resize event.
    Resize = 65,

    /// The draw begin event.
    DrawBegin = 72,

    /// The draw end event.
    DrawEnd = 73,

    /// The draw GUI begin event.
    DrawGUIBegin = 74,

    /// The draw GUI end event.
    DrawGUIEnd = 75,

    /// The pre-draw event.
    PreDraw = 76,

    /// The post-draw event.
    PostDraw = 77,
}

impl EventSubtype for Draw {
    fn parse(subtype: u32) -> Result<Self> {
        num_enum_from(subtype)
    }

    fn build(self) -> u32 {
        self.into()
    }
}

impl Default for Draw {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Draw {
    pub const DEFAULT: Self = Self::Draw;

    /// Whether this draw event happens during the GUI drawing stage.
    #[must_use]
    pub const fn is_gui(self) -> bool {
        matches!(self, Self::DrawGUI | Self::DrawGUIBegin | Self::DrawGUIEnd)
    }
}
