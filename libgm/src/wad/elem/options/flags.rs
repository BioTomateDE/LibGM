// SPDX-License-Identifier: GPL-3.0-only

bitflags::bitflags! {
    /// General options/flags for the game.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct OptionFlags: u64 {
        /// If the game should start in fullscreen.
        const FULLSCREEN = 0x1;

        /// If pixels should be interpolated.
        const INTERPOLATE_PIXELS = 0x2;

        /// If the new audio format should be used.
        const USE_NEW_AUDIO = 0x4;

        /// If borderless window should be used.
        const NO_BORDER = 0x8;

        /// If the mouse cursor should be shown.
        const SHOW_CURSOR = 0x10;

        /// If the window should be resizable.
        const SIZEABLE = 0x20;

        /// If the window should stay on top.
        const STAY_ON_TOP = 0x40;

        /// If the resolution can be changed.
        const CHANGE_RESOLUTION = 0x80;

        const NO_BUTTONS = 0x100;
        const SCREEN_KEY = 0x200;
        const HELP_KEY = 0x400;
        const QUIT_KEY = 0x800;
        const SAVE_KEY = 0x1000;
        const SCREENSHOT_KEY = 0x2000;
        const CLOSE_SEC = 0x4000;
        const FREEZE = 0x8000;
        const SHOW_PROGRESS = 0x10000;
        const LOAD_TRANSPARENT = 0x20000;
        const SCALE_PROGRESS = 0x40000;
        const DISPLAY_ERRORS = 0x80000;
        const WRITE_ERRORS = 0x10_0000;
        const ABORT_ERRORS = 0x20_0000;
        const VARIABLE_ERRORS = 0x40_0000;
        const CREATION_EVENT_ORDER = 0x80_0000;
        const USE_FRONT_TOUCH = 0x100_0000;
        const USE_REAR_TOUCH = 0x200_0000;
        const USE_FAST_COLLISION = 0x400_0000;
        const FAST_COLLISION_COMPATIBILITY = 0x800_0000;
        const DISABLE_SANDBOX = 0x1000_0000;
        const ENABLE_COPY_ON_WRITE = 0x2000_0000;
        const LEGACY_JSON_PARSING = 0x4000_0000;
        const LEGACY_NUMBER_CONVERSION = 0x8000_0000;
        const LEGACY_OTHER_BEHAVIOR = 0x1_0000_0000;
        const AUDIO_ERROR_BEHAVIOR = 0x2_0000_0000;
        const ALLOW_INSTANCE_CHANGE = 0x4_0000_0000;
        const LEGACY_PRIMITIVE_DRAWING = 0x8_0000_0000;
    }
}
