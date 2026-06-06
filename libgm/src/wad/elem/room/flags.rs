// SPDX-License-Identifier: GPL-3.0-only

bitflags::bitflags! {
    /// Certain flags a room can have.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Flags: u32 {
        /// Whether the room has Views enabled.
        const ENABLE_VIEWS = 0x1;

        /// Clears the application surface with the window colour before drawing each frame.
        const CLEAR_VIEW_BACKGROUND = 0x2;

        /// Whether the room should **not** clear the display buffer.
        const DONT_CLEAR_DISPLAY_BUFFER = 0x4;

        /// Whether the room was made in GameMaker Studio 2 or above.
        const GM2 = 0x20000;

        /// Whether the room was made in GameMaker Studio 2.3 or above.
        const GM2_3 = 0x10000;

        /// Whether the room was made in GameMaker 2024.13 or above.
        const GM_2024_13 = 0x40000;
    }
}
