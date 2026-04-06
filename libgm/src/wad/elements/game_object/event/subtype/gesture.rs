use macros::num_enum;

use crate::prelude::*;
use crate::util::init::num_enum_from;
use crate::wad::elements::game_object::event::EventSubtype;

/// The subtype for [`EventType::Gesture`].
#[num_enum(u32)]
pub enum Gesture {
    /// The tap event.
    Tap = 0,

    /// The double tap event.
    DoubleTap = 1,

    /// The drag start event.
    DragStart = 2,

    /// The dragging event.
    DragMove = 3,

    /// The drag end event.
    DragEnd = 4,

    /// The flick event.
    Flick = 5,

    /// The pinch start event.
    PinchStart = 6,

    /// The pinch in event.
    PinchIn = 7,

    /// The pinch out event.
    PinchOut = 8,

    /// The pinch end event.
    PinchEnd = 9,

    /// The rotate start event.
    RotateStart = 10,

    /// The rotating event.
    Rotating = 11,

    /// The rotate end event.
    RotateEnd = 12,

    /// The global tap event.
    GlobalTap = 64,

    /// The global double tap event.
    GlobalDoubleTap = 65,

    /// The global drag start event.
    GlobalDragStart = 66,

    /// The global dragging event.
    GlobalDragMove = 67,

    /// The global drag end event.
    GlobalDragEnd = 68,

    /// The global flick event.
    GlobalFlick = 69,

    /// The global pinch start event.
    GlobalPinchStart = 70,

    /// The global pinch in event.
    GlobalPinchIn = 71,

    /// The global pinch out event.
    GlobalPinchOut = 72,

    /// The global pinch end event.
    GlobalPinchEnd = 73,

    /// The global rotate start event.
    GlobalRotateStart = 74,

    /// The global rotating event.
    GlobalRotating = 75,

    /// The global rotate end event.
    GlobalRotateEnd = 76,
}

impl EventSubtype for Gesture {
    fn parse(subtype: u32) -> Result<Self> {
        num_enum_from(subtype)
    }

    fn build(self) -> u32 {
        self.into()
    }
}

impl Gesture {
    /// Whether this is a **global** gesture event.
    ///
    /// "Normal" (non-global) events only get triggered when the gesture
    /// collides with the collision box of the game object instance,
    /// Global events **always** get triggered; even if the gesture position is
    /// far away.
    #[must_use]
    pub const fn is_global(self) -> bool {
        match self {
            Self::GlobalTap
            | Self::GlobalDoubleTap
            | Self::GlobalDragStart
            | Self::GlobalDragMove
            | Self::GlobalDragEnd
            | Self::GlobalFlick
            | Self::GlobalPinchStart
            | Self::GlobalPinchIn
            | Self::GlobalPinchOut
            | Self::GlobalPinchEnd
            | Self::GlobalRotateStart
            | Self::GlobalRotating
            | Self::GlobalRotateEnd => true,
            _ => false,
        }
    }
}
