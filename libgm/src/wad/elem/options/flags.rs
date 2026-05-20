use crate::util::bitfield::bitfield_struct;

bitfield_struct! {
    /// General options/flags for the game.
    Flags : u64 {
        /// If the game should start in fullscreen.
        fullscreen: 0,

        /// If pixels should be interpolated.
        interpolate_pixels: 1,

        /// If the new audio format should be used.
        use_new_audio: 2,

        /// If borderless window should be used.
        no_border: 3,

        /// If the mouse cursor should be shown.
        show_cursor: 4,

        /// If the window should be resizable.
        sizeable: 5,

        /// If the window should stay on top.
        stay_on_top: 6,

        /// If the resolution can be changed.
        change_resolution: 7,

        no_buttons: 8,
        screen_key: 9,
        help_key: 10,
        quit_key: 11,
        save_key: 12,
        screenshot_key: 13,
        close_sec: 14,
        freeze: 15,
        show_progress: 16,
        load_transparent: 17,
        scale_progress: 18,
        display_errors: 19,
        write_errors: 20,
        abort_errors: 21,
        variable_errors: 22,
        creation_event_order: 23,
        use_front_touch: 24,
        use_rear_touch: 25,
        use_fast_collision: 26,
        fast_collision_compatibility: 27,
        disable_sandbox: 28,
        enable_copy_on_write: 29,
        legacy_json_parsing: 30,
        legacy_number_conversion: 31,
        legacy_other_behavior: 32,
        audio_error_behavior: 33,
        allow_instance_change: 34,
        legacy_primitive_drawing: 35,
    }
}
