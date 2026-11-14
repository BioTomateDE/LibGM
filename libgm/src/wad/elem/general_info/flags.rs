// SPDX-License-Identifier: GPL-3.0-only

bitflags::bitflags! {
    /// Contains general information flags for GameMaker games.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Flags: u32 {
        /// Start the game as fullscreen.
        const FULLSCREEN = 0x1;

        /// Use synchronization to avoid tearing.
        const SYNC_VERTEX1 = 0x2;

        /// Use synchronization to avoid tearing. (???)
        const SYNC_VERTEX2 = 0x4;

        /// Interpolate colours between pixels.
        const INTERPOLATE = 0x8;

        /// Keep aspect ratio during scaling.
        const SCALE = 0x10;

        /// Display mouse cursor.
        const SHOW_CURSOR = 0x20;

        /// Allows window to be resized.
        const SIZEABLE = 0x40;

        /// Allows toggling fullscreen.
        const ALLOW_FULLSCREEN = 0x80;

        /// Use synchronization to avoid tearing. (???)
        const SYNC_VERTEX3 = 0x100;

        const STUDIO_VERSION_1 = 0x200;
        const STUDIO_VERSION_2 = 0x400;
        const STUDIO_VERSION_3 = 0x800;
        const STUDIO_VERSION_4 = 0x600;

        /// Whether Steam (or the `YoYoPlayer`) is enabled.
        const STEAM_ENABLED = 0x1000;

        /// When enabled, the game will write save data to `%appdata%\GameName`.
        /// Otherwise, it will write to `%localappdata%\GameName`.
        const APPDATA_ROAMING = 0x2000;

        /// Whether the game supports borderless window.
        const BORDERLESS_WINDOW = 0x4000;

        /// Tells the runner to run JavaScript code.
        const JAVASCRIPT_MODE = 0x8000;

        /// This flag is set when a game is launched from the
        /// Gamemaker Studio 2 IDE using the 'Run' or 'Debug' options.
        ///
        /// Also known as `override_hobbysplash`.
        const LICENSE_EXCLUSIONS = 0x10000;

        /// For the GX.games runner, allows the browser canvas to be transparent where nothing is drawn.
        const TRANSPARENT_BACKGROUND = 0x20000;

        /// For the Windows runner, reverts the swapchain to an older/legacy swap effect.
        const D3D_SWAP_EFFECT_DISCARD = 0x40000;
    }
}
