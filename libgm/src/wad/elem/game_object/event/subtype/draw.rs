// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::GMEnum;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::elem::game_object::event::EventSubtype;

gm_enum!(
/// Triggered when the game loop is in the rendering/drawing stage.
///
/// This occurs every step/frame, but is called with different
/// timing and with a different purpose than Step events.
Draw {
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
});

impl EventSubtype for Draw {
    fn parse(subtype: i32) -> Result<Self> {
        Self::from_i32(subtype)
    }

    fn build(self) -> i32 {
        self.as_i32()
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
