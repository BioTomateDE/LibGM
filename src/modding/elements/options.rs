use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::options::{GMOptions, GMOptionsFlags};
use crate::modding::export::{edit_field, edit_field_convert, edit_field_convert_option, flag_field, ModExporter, ModRef, RootChanges};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

macro_rules! prevent_changing {
    ($original:expr, $modified:expr, $field:ident, $name:expr) => {{
        if $original.$field != $modified.$field {
            return Err(format!("Changing options field {} is not allowed!", $name))
        }
    }};
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOptions {
    pub flags: EditOptionsFlags,
    // pub window_scale: Option<i32>,           // removed because legacy/unused
    // pub window_color: Option<RgbaColor>,     // removed because legacy/unused
    /// Only used in GMVersion <= 8
    pub color_depth: Option<u32>,
    /// Only used in GMVersion <= 8
    pub resolution: Option<u32>,
    /// Only used in GMVersion <= 8
    pub frequency: Option<u32>,
    /// Only used in GMVersion <= 8
    pub vertex_sync: Option<i32>,
    /// Only used in GMVersion <= 8
    pub priority: Option<i32>,
    /// Only used in GMVersion <?= 8
    pub back_image: Option<ModRef>,     // TexturePageItem ref
    /// Only used in GMVersion <?= 8
    pub front_image: Option<ModRef>,    // TexturePageItem ref
    /// Only used in GMVersion <?= 8
    pub load_image: Option<ModRef>,     // TexturePageItem ref
    /// Only used in GMVersion <?= 8
    pub load_alpha: Option<u32>,
    /// wow, it's actually used
    pub constants: Vec<DataChange<AddOptionsConstant, EditOptionsConstant>>,
}

impl RootChanges for EditOptions {
    fn has_changes(&self) -> bool {
        self.flags.has_changes() ||
        self.color_depth.is_some() ||
        self.resolution.is_some() ||
        self.frequency.is_some() ||
        self.vertex_sync.is_some() ||
        self.priority.is_some() ||
        self.back_image.is_some() ||
        self.front_image.is_some() ||
        self.load_image.is_some() ||
        self.load_alpha.is_some() ||
        self.constants.has_changes()
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOptionsFlags {
    pub start_in_fullscreen: Option<bool>,
    pub interpolate_pixels: Option<bool>,
    pub use_new_audio_format: Option<bool>,
    pub no_border: Option<bool>,
    pub show_cursor: Option<bool>,
    pub sizeable: Option<bool>,
    pub stay_on_top: Option<bool>,
    pub allow_changing_resolution: Option<bool>,
    pub no_buttons: Option<bool>,
    pub screen_key: Option<bool>,
    pub help_key: Option<bool>,
    pub quit_key: Option<bool>,
    pub save_key: Option<bool>,
    pub screenshot_key: Option<bool>,
    pub close_delay: Option<bool>,
    pub freeze: Option<bool>,
    pub show_progress: Option<bool>,
    pub load_transparent: Option<bool>,
    pub scale_progress: Option<bool>,
    pub display_errors: Option<bool>,
    pub write_errors: Option<bool>,
    pub abort_errors: Option<bool>,
    pub variable_errors: Option<bool>,
    pub creation_event_order: Option<bool>,
    pub use_front_touch: Option<bool>,
    pub use_rear_touch: Option<bool>,
    pub use_fast_collision: Option<bool>,
    pub fast_collision_compatibility: Option<bool>,
    pub disable_sandbox: Option<bool>,
    pub enable_copy_on_write: Option<bool>,
}

impl RootChanges for EditOptionsFlags {
    fn has_changes(&self) -> bool {
        self.start_in_fullscreen.is_some() ||
        self.interpolate_pixels.is_some() ||
        self.use_new_audio_format.is_some() ||
        self.no_border.is_some() ||
        self.show_cursor.is_some() ||
        self.sizeable.is_some() ||
        self.stay_on_top.is_some() ||
        self.allow_changing_resolution.is_some() ||
        self.no_buttons.is_some() ||
        self.screen_key.is_some() ||
        self.help_key.is_some() ||
        self.quit_key.is_some() ||
        self.save_key.is_some() ||
        self.screenshot_key.is_some() ||
        self.close_delay.is_some() ||
        self.freeze.is_some() ||
        self.show_progress.is_some() ||
        self.load_transparent.is_some() ||
        self.scale_progress.is_some() ||
        self.display_errors.is_some() ||
        self.write_errors.is_some() ||
        self.abort_errors.is_some() ||
        self.variable_errors.is_some() ||
        self.creation_event_order.is_some() ||
        self.use_front_touch.is_some() ||
        self.use_rear_touch.is_some() ||
        self.use_fast_collision.is_some() ||
        self.fast_collision_compatibility.is_some() ||
        self.disable_sandbox.is_some() ||
        self.enable_copy_on_write.is_some()
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOptionsConstant {
    pub name: Option<ModRef>,   // String ref
    pub value: Option<ModRef>,  // String ref
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddOptionsConstant {
    pub name: ModRef,   // String ref
    pub value: ModRef,  // String ref
}


impl ModExporter<'_, '_> {
    pub fn export_options(&self) -> Result<EditOptions, String> {
        let o: &GMOptions = &self.original_data.options;
        let m: &GMOptions = &self.modified_data.options;
        
        prevent_changing!(o, m, unknown1, "Unknown Value 1");
        prevent_changing!(o, m, unknown2, "Unknown Value 2");
        prevent_changing!(o, m, window_scale, "Window Scale (legacy)");
        prevent_changing!(o, m, window_color, "Window Color (legacy)");
        
        Ok(EditOptions {
            flags: edit_flags(&o.flags, &m.flags),
            color_depth: edit_field(&o.color_depth, &m.color_depth),
            resolution: edit_field(&o.resolution, &m.resolution),
            frequency: edit_field(&o.frequency, &m.frequency),
            vertex_sync: edit_field(&o.vertex_sync, &m.vertex_sync),
            priority: edit_field(&o.priority, &m.priority),
            back_image: edit_field_convert_option(&o.back_image, &m.back_image, |r| self.convert_texture_ref(r))?.unwrap_or(None),
            front_image: edit_field_convert_option(&o.front_image, &m.front_image, |r| self.convert_texture_ref(r))?.unwrap_or(None),
            load_image: edit_field_convert_option(&o.load_image, &m.load_image, |r| self.convert_texture_ref(r))?.unwrap_or(None),
            load_alpha: edit_field(&o.load_alpha, &m.load_alpha),
            constants: export_changes_ordered_list(
                &o.constants,
                &m.constants,
                |i| Ok(AddOptionsConstant {
                    name: self.convert_string_ref(&i.name)?,
                    value: self.convert_string_ref(&i.value)?,
                }),
                |o, m| Ok(EditOptionsConstant {
                    name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                    value: edit_field_convert(&o.value, &m.value, |r| self.convert_string_ref(r))?,
                }),
            )?,
        })
    }
}


fn edit_flags(o: &GMOptionsFlags, m: &GMOptionsFlags) -> EditOptionsFlags {
    EditOptionsFlags {
        start_in_fullscreen: flag_field(o.fullscreen, m.fullscreen),
        interpolate_pixels: flag_field(o.interpolate_pixels, m.interpolate_pixels),
        use_new_audio_format: flag_field(o.use_new_audio, m.use_new_audio),
        no_border: flag_field(o.no_border, m.no_border),
        show_cursor: flag_field(o.show_cursor, m.show_cursor),
        sizeable: flag_field(o.sizeable, m.sizeable),
        stay_on_top: flag_field(o.stay_on_top, m.stay_on_top),
        allow_changing_resolution: flag_field(o.change_resolution, m.change_resolution),
        no_buttons: flag_field(o.no_buttons, m.no_buttons),
        screen_key: flag_field(o.screen_key, m.screen_key),
        help_key: flag_field(o.help_key, m.help_key),
        quit_key: flag_field(o.quit_key, m.quit_key),
        save_key: flag_field(o.save_key, m.save_key),
        screenshot_key: flag_field(o.screenshot_key, m.screenshot_key),
        close_delay: flag_field(o.close_sec, m.close_sec),
        freeze: flag_field(o.freeze, m.freeze),
        show_progress: flag_field(o.show_progress, m.show_progress),
        load_transparent: flag_field(o.load_transparent, m.load_transparent),
        scale_progress: flag_field(o.scale_progress, m.scale_progress),
        display_errors: flag_field(o.display_errors, m.display_errors),
        write_errors: flag_field(o.write_errors, m.write_errors),
        abort_errors: flag_field(o.abort_errors, m.abort_errors),
        variable_errors: flag_field(o.variable_errors, m.variable_errors),
        creation_event_order: flag_field(o.creation_event_order, m.creation_event_order),
        use_front_touch: flag_field(o.use_front_touch, m.use_front_touch),
        use_rear_touch: flag_field(o.use_rear_touch, m.use_rear_touch),
        use_fast_collision: flag_field(o.use_fast_collision, m.use_fast_collision),
        fast_collision_compatibility: flag_field(o.fast_collision_compatibility, m.fast_collision_compatibility),
        disable_sandbox: flag_field(o.disable_sandbox, m.disable_sandbox),
        enable_copy_on_write: flag_field(o.enable_copy_on_write, m.enable_copy_on_write),
    }
}

