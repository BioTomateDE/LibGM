use macros::num_enum;

use crate::{
    prelude::*, util::init::num_enum_from, wad::elements::game_object::event::EventSubtype,
};

/// The subtype for [`EventType::Mouse`].
#[num_enum(u32)]
pub enum Mouse {
    /// The left-mouse button down event.
    LeftButton = 0,

    /// The right-mouse button down event.
    RightButton = 1,

    /// The middle-mouse button down event.
    MiddleButton = 2,

    /// The no-mouse input event.
    ///
    /// I don't know why this exits.
    NoButton = 3,

    /// The left-mouse button pressed event.
    LeftPressed = 4,

    /// The right-mouse button pressed event.
    RightPressed = 5,

    /// The middle-mouse button pressed event.
    MiddlePressed = 6,

    /// The left-mouse button released event.
    LeftReleased = 7,

    /// The right-mouse button released event.
    RightReleased = 8,

    /// The middle-mouse button released event.
    MiddleReleased = 9,

    /// The mouse enter event.
    MouseEnter = 10,

    /// The mouse leave event.
    MouseLeave = 11,

    /// The Joystick1 left event. Is only used in Pre-GameMaker Studio.
    Joystick1Left = 16,

    /// The Joystick1 right event. Is only used in Pre-GameMaker Studio.
    Joystick1Right = 17,

    /// The Joystick1 up event. Is only used in Pre-GameMaker Studio.
    Joystick1Up = 18,

    /// The Joystick1 down event. Is only used in Pre-GameMaker Studio.
    Joystick1Down = 19,

    /// The Joystick1 button1 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button1 = 21,

    /// The Joystick1 button2 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button2 = 22,

    /// The Joystick1 button3 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button3 = 23,

    /// The Joystick1 button4 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button4 = 24,

    /// The Joystick1 button5 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button5 = 25,

    /// The Joystick1 button6 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button6 = 26,

    /// The Joystick1 button7 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button7 = 27,

    /// The Joystick1 button8 event. Is only used in Pre-GameMaker Studio.
    Joystick1Button8 = 28,

    /// The Joystick2 left event. Is only used in Pre-GameMaker Studio.
    Joystick2Left = 31,

    /// The Joystick2 right event. Is only used in Pre-GameMaker Studio.
    Joystick2Right = 32,

    /// The Joystick2 up event. Is only used in Pre-GameMaker Studio.
    Joystick2Up = 33,

    /// The Joystick2 down event. Is only used in Pre-GameMaker Studio.
    Joystick2Down = 34,

    /// The Joystick2 button1 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button1 = 36,

    /// The Joystick2 button2 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button2 = 37,

    /// The Joystick2 button3 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button3 = 38,

    /// The Joystick2 button4 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button4 = 39,

    /// The Joystick2 button5 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button5 = 40,

    /// The Joystick2 button6 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button6 = 41,

    /// The Joystick2 button7 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button7 = 42,

    /// The Joystick2 button8 event. Is only used in Pre-GameMaker Studio.
    Joystick2Button8 = 43,

    /// The global left-mouse button down event.
    GlobLeftButton = 50,

    /// The global right-mouse button down event.
    GlobRightButton = 51,

    /// The global middle-mouse button down event.
    GlobMiddleButton = 52,

    /// The global left-mouse button pressed event.
    GlobLeftPressed = 53,

    /// The global right-mouse button pressed event.
    GlobRightPressed = 54,

    /// The global middle-mouse button pressed event.
    GlobMiddlePressed = 55,

    /// The global left-mouse button released event.
    GlobLeftReleased = 56,

    /// The global right-mouse button released event.
    GlobRightReleased = 57,

    /// The global middle-mouse button released event.
    GlobMiddleReleased = 58,

    /// The mouse-wheel up event.
    MouseWheelUp = 60,

    /// The mouse-wheel down event.
    MouseWheelDown = 61,
}

impl EventSubtype for Mouse {
    fn parse(subtype: u32) -> Result<Self> {
        num_enum_from(subtype)
    }

    fn build(self) -> u32 {
        self.into()
    }
}

impl Mouse {
    /// Whether this mouse event is actually a controller/joystick related event.]
    ///
    /// These events only exist in GMS1.
    #[must_use]
    pub const fn is_joystick(self) -> bool {
        match self {
            Self::Joystick1Left
            | Self::Joystick1Right
            | Self::Joystick1Up
            | Self::Joystick1Down
            | Self::Joystick1Button1
            | Self::Joystick1Button2
            | Self::Joystick1Button3
            | Self::Joystick1Button4
            | Self::Joystick1Button5
            | Self::Joystick1Button6
            | Self::Joystick1Button7
            | Self::Joystick1Button8
            | Self::Joystick2Left
            | Self::Joystick2Right
            | Self::Joystick2Up
            | Self::Joystick2Down
            | Self::Joystick2Button1
            | Self::Joystick2Button2
            | Self::Joystick2Button3
            | Self::Joystick2Button4
            | Self::Joystick2Button5
            | Self::Joystick2Button6
            | Self::Joystick2Button7
            | Self::Joystick2Button8 => true,
            _ => false,
        }
    }

    /// Whether this mouse event is related to a mouse button being pressed/released/held.
    ///
    /// This will return `false` for:
    /// * joystick events (see [`Mouse::is_joystick`])
    /// * mouse enter / leave event
    /// * mouse wheel up / down event
    /// * no button event
    #[must_use]
    pub const fn is_button(self) -> bool {
        match self {
            Self::LeftButton
            | Self::RightButton
            | Self::MiddleButton
            | Self::LeftPressed
            | Self::RightPressed
            | Self::MiddlePressed
            | Self::LeftReleased
            | Self::RightReleased
            | Self::MiddleReleased
            | Self::GlobLeftButton
            | Self::GlobRightButton
            | Self::GlobMiddleButton
            | Self::GlobLeftPressed
            | Self::GlobRightPressed
            | Self::GlobMiddlePressed
            | Self::GlobLeftReleased
            | Self::GlobRightReleased
            | Self::GlobMiddleReleased => true,
            _ => false,
        }
    }

    /// Whether this mouse event gets triggered when the left mouse button is pressed/released/held.
    #[must_use]
    pub const fn is_left_button(self) -> bool {
        match self {
            Self::LeftButton
            | Self::LeftPressed
            | Self::LeftReleased
            | Self::GlobLeftButton
            | Self::GlobLeftPressed
            | Self::GlobLeftReleased => true,
            _ => false,
        }
    }

    /// Whether this mouse event gets triggered when the right mouse button is pressed/released/held.
    #[must_use]
    pub const fn is_right_button(self) -> bool {
        match self {
            Self::RightButton
            | Self::RightPressed
            | Self::RightReleased
            | Self::GlobRightButton
            | Self::GlobRightPressed
            | Self::GlobRightReleased => true,
            _ => false,
        }
    }

    /// Whether this mouse event gets triggered when the middle mouse button is pressed/released/held.
    #[must_use]
    pub const fn is_middle_button(self) -> bool {
        match self {
            Self::MiddleButton
            | Self::MiddlePressed
            | Self::MiddleReleased
            | Self::GlobMiddleButton
            | Self::GlobMiddlePressed
            | Self::GlobMiddleReleased => true,
            _ => false,
        }
    }

    /// Whether this event gets triggered while the button is held
    /// (instead of only when pressed or released).
    #[must_use]
    pub const fn is_held(self) -> bool {
        match self {
            Self::LeftButton
            | Self::RightButton
            | Self::MiddleButton
            | Self::GlobLeftButton
            | Self::GlobRightButton
            | Self::GlobMiddleButton => true,
            _ => false,
        }
    }

    /// Whether this event gets triggered once when the button is pressed down
    /// (instead of while held or when released).
    #[must_use]
    pub const fn is_pressed(self) -> bool {
        match self {
            Self::LeftPressed
            | Self::RightPressed
            | Self::MiddlePressed
            | Self::GlobLeftPressed
            | Self::GlobRightPressed
            | Self::GlobMiddlePressed => true,
            _ => false,
        }
    }

    /// Whether this event gets triggered once when the button is released
    /// (instead of while held or when pressed).
    #[must_use]
    pub const fn is_released(self) -> bool {
        match self {
            Self::LeftReleased
            | Self::RightReleased
            | Self::MiddleReleased
            | Self::GlobLeftReleased
            | Self::GlobRightReleased
            | Self::GlobMiddleReleased => true,
            _ => false,
        }
    }

    /// Whether this is a **global** mouse event.
    ///
    /// "Normal" (non-global) events only get triggered when the mouse
    /// collides with the collision box of the game object instance,
    /// Global events **always** get triggered; even if the mouse position is far away.
    #[must_use]
    pub const fn is_global(self) -> bool {
        match self {
            Self::GlobLeftButton
            | Self::GlobRightButton
            | Self::GlobMiddleButton
            | Self::GlobLeftPressed
            | Self::GlobRightPressed
            | Self::GlobMiddlePressed
            | Self::GlobLeftReleased
            | Self::GlobRightReleased
            | Self::GlobMiddleReleased => true,
            _ => false,
        }
    }
}
