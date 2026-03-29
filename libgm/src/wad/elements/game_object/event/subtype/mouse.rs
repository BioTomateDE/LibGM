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
