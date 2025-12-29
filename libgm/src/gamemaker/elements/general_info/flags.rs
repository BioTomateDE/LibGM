use crate::util::bitfield::bitfield_struct;

bitfield_struct! {
    /// Contains general information flags for GameMaker games.
    Flags : u32 {
        /// Start the game as fullscreen.
        fullscreen: 0,

        /// Use synchronization to avoid tearing.
        sync_vertex1: 1,

        /// Use synchronization to avoid tearing. (???)
        sync_vertex2: 2,

        /// Use synchronization to avoid tearing. (???)
        sync_vertex3: 8,

        /// Interpolate colours between pixels.
        interpolate: 3,

        /// Keep aspect ratio during scaling.
        scale: 4,

        /// Display mouse cursor.
        show_cursor: 5,

        /// Allows window to be resized.
        sizeable: 6,

        /// Allows fullscreen switching. (???)
        screen_key: 7,

        studio_version_b1: 9,

        studio_version_b2: 10,

        studio_version_b3: 11,

        steam_enabled: 12,

        local_data_enabled: 13,

        /// Whether the game supports borderless window
        borderless_window: 14,

        /// Tells the runner to run JavaScript code
        javascript_mode: 15,

        license_exclusions: 16,
    }
}
