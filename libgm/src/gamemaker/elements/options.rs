use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::reference::GMRef;
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use crate::util::bitfield::bitfield_struct;

#[derive(Debug, Clone, Default)]
pub struct GMOptions {
    is_new_format: bool,
    pub unknown1: u32,
    pub unknown2: u32,
    pub flags: GMOptionsFlags,
    pub window_scale: i32,
    pub window_color: u32,
    pub color_depth: u32,
    pub resolution: u32,
    pub frequency: u32,
    pub vertex_sync: i32,
    pub priority: i32,
    pub back_image: Option<GMRef<GMTexturePageItem>>,
    pub front_image: Option<GMRef<GMTexturePageItem>>,
    pub load_image: Option<GMRef<GMTexturePageItem>>,
    pub load_alpha: u32,
    pub constants: Vec<GMOptionsConstant>,
    pub exists: bool,
}

impl GMChunkElement for GMOptions {
    const NAME: &'static str = "OPTN";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMOptions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let is_new_format: bool = reader.read_u32()? == 0x80000000;
        reader.cur_pos -= 4;
        if is_new_format {
            parse_options_new(reader)
        } else {
            parse_options_old(reader)
        }
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if self.is_new_format {
            build_options_new(builder, self)?;
        } else {
            build_options_old(builder, self)?;
        }
        Ok(())
    }
}

bitfield_struct! {
    /// General options/flags for the game.
    GMOptionsFlags : u64 {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMOptionsConstant {
    pub name: String,
    pub value: String,
}

impl GMElement for GMOptionsConstant {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let value: String = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_gm_string(&self.value);
        Ok(())
    }
}

fn parse_options_new(reader: &mut DataReader) -> Result<GMOptions> {
    let unknown1 = reader.read_u32()?;
    let unknown2 = reader.read_u32()?;
    let flags = GMOptionsFlags::deserialize(reader)?;
    let window_scale = reader.read_i32()?;
    let window_color = reader.read_u32()?;
    let color_depth = reader.read_u32()?;
    let resolution = reader.read_u32()?;
    let frequency = reader.read_u32()?;
    let vertex_sync = reader.read_i32()?;
    let priority = reader.read_i32()?;
    let back_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let front_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_alpha = reader.read_u32()?;
    let constants: Vec<GMOptionsConstant> = reader.read_simple_list()?;

    Ok(GMOptions {
        is_new_format: true,
        unknown1,
        unknown2,
        flags,
        window_scale,
        window_color,
        color_depth,
        resolution,
        frequency,
        vertex_sync,
        priority,
        back_image,
        front_image,
        load_image,
        load_alpha,
        constants,
        exists: true,
    })
}

fn parse_options_old(reader: &mut DataReader) -> Result<GMOptions> {
    let flag_fullscreen = reader.read_bool32()?;
    let flag_interpolate_pixels = reader.read_bool32()?;
    let flag_use_new_audio = reader.read_bool32()?;
    let flag_no_border = reader.read_bool32()?;
    let flag_show_cursor = reader.read_bool32()?;

    let scale = reader.read_i32()?;

    let flag_sizeable = reader.read_bool32()?;
    let flag_stay_on_top = reader.read_bool32()?;

    let window_color = reader.read_u32()?;

    let flag_change_resolution = reader.read_bool32()?;

    let color_depth = reader.read_u32()?;
    let resolution = reader.read_u32()?;
    let frequency = reader.read_u32()?;

    let flag_no_buttons = reader.read_bool32()?;

    let vertex_sync = reader.read_i32()?;

    let flag_screen_key = reader.read_bool32()?;
    let flag_help_key = reader.read_bool32()?;
    let flag_quit_key = reader.read_bool32()?;
    let flag_save_key = reader.read_bool32()?;
    let flag_screenshot_key = reader.read_bool32()?;
    let flag_close_sec = reader.read_bool32()?;

    let priority = reader.read_i32()?;

    let flag_freeze = reader.read_bool32()?;
    let flag_show_progress = reader.read_bool32()?;

    let back_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let front_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;

    let flag_load_transparent = reader.read_bool32()?;

    let load_alpha = reader.read_u32()?;

    let flag_scale_progress = reader.read_bool32()?;
    let flag_display_errors = reader.read_bool32()?;
    let flag_write_errors = reader.read_bool32()?;
    let flag_abort_errors = reader.read_bool32()?;
    let flag_variable_errors = reader.read_bool32()?;
    let flag_creation_event_order = reader.read_bool32()?;

    let constants: Vec<GMOptionsConstant> = reader.read_simple_list()?;

    Ok(GMOptions {
        is_new_format: false,
        unknown1: 0, // Might not be best practice?
        unknown2: 0,
        flags: GMOptionsFlags {
            fullscreen: flag_fullscreen,
            interpolate_pixels: flag_interpolate_pixels,
            use_new_audio: flag_use_new_audio,
            no_border: flag_no_border,
            show_cursor: flag_show_cursor,
            sizeable: flag_sizeable,
            stay_on_top: flag_stay_on_top,
            change_resolution: flag_change_resolution,
            no_buttons: flag_no_buttons,
            screen_key: flag_screen_key,
            help_key: flag_help_key,
            quit_key: flag_quit_key,
            save_key: flag_save_key,
            screenshot_key: flag_screenshot_key,
            close_sec: flag_close_sec,
            freeze: flag_freeze,
            show_progress: flag_show_progress,
            load_transparent: flag_load_transparent,
            scale_progress: flag_scale_progress,
            display_errors: flag_display_errors,
            write_errors: flag_write_errors,
            abort_errors: flag_abort_errors,
            variable_errors: flag_variable_errors,
            creation_event_order: flag_creation_event_order,
            ..Default::default()
        },
        window_scale: scale,
        window_color,
        color_depth,
        resolution,
        frequency,
        vertex_sync,
        priority,
        back_image,
        front_image,
        load_image,
        load_alpha,
        constants,
        exists: true,
    })
}

fn build_options_old(builder: &mut DataBuilder, options: &GMOptions) -> Result<()> {
    builder.write_bool32(options.flags.fullscreen);
    builder.write_bool32(options.flags.interpolate_pixels);
    builder.write_bool32(options.flags.use_new_audio);
    builder.write_bool32(options.flags.no_border);
    builder.write_bool32(options.flags.show_cursor);

    builder.write_i32(options.window_scale);

    builder.write_bool32(options.flags.sizeable);
    builder.write_bool32(options.flags.stay_on_top);

    builder.write_u32(options.window_color);

    builder.write_bool32(options.flags.change_resolution);

    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);

    builder.write_bool32(options.flags.no_buttons);

    builder.write_i32(options.vertex_sync);

    builder.write_bool32(options.flags.screen_key);
    builder.write_bool32(options.flags.help_key);
    builder.write_bool32(options.flags.quit_key);
    builder.write_bool32(options.flags.save_key);
    builder.write_bool32(options.flags.screenshot_key);
    builder.write_bool32(options.flags.close_sec);

    builder.write_i32(options.priority);

    builder.write_bool32(options.flags.freeze);
    builder.write_bool32(options.flags.show_progress);

    builder.write_pointer_opt(&options.back_image)?;
    builder.write_pointer_opt(&options.front_image)?;
    builder.write_pointer_opt(&options.load_image)?;

    builder.write_bool32(options.flags.load_transparent);

    builder.write_u32(options.load_alpha);

    builder.write_bool32(options.flags.scale_progress);
    builder.write_bool32(options.flags.display_errors);
    builder.write_bool32(options.flags.write_errors);
    builder.write_bool32(options.flags.abort_errors);
    builder.write_bool32(options.flags.variable_errors);
    builder.write_bool32(options.flags.creation_event_order);
    Ok(())
}

fn build_options_new(builder: &mut DataBuilder, options: &GMOptions) -> Result<()> {
    builder.write_u32(options.unknown1);
    builder.write_u32(options.unknown2);
    options.flags.serialize(builder)?;
    builder.write_i32(options.window_scale);
    builder.write_u32(options.window_color);
    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);
    builder.write_i32(options.vertex_sync);
    builder.write_i32(options.priority);
    builder.write_pointer_opt(&options.back_image)?;
    builder.write_pointer_opt(&options.front_image)?;
    builder.write_pointer_opt(&options.load_image)?;
    builder.write_u32(options.load_alpha);
    builder.write_simple_list(&options.constants)?;
    Ok(())
}
