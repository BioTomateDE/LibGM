use crate::util::bitfield::bitfield_struct;

bitfield_struct! {
    /// Certain flags a room can have.
    Flags : u32 {
        /// Whether the room has Views enabled.
        enable_views: 0,

        /// TODO(doc) not exactly sure, probably similar to `GMRoom.draw_background_color`?
        show_color: 1,

        /// Whether the room should not clear the display buffer.
        dont_clear_display_buffer: 2,

        /// Whether the room was made in GameMaker Studio 2 or above.
        is_gm2: 17,

        /// Whether the room was made in GameMaker Studio 2.3 or above.
        is_gm2_3: 16,

        /// Whether the room was made in GameMaker 2024.13 or above.
        is_gm_2024_13: 18,
    }
}
