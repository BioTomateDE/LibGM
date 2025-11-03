use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct GMOptions {
    pub is_new_format: bool,
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

#[derive(Debug, Clone, Default)]
pub struct GMOptionsFlags {
    pub fullscreen: bool,
    pub interpolate_pixels: bool,
    pub use_new_audio: bool,
    pub no_border: bool,
    pub show_cursor: bool,
    pub sizeable: bool,
    pub stay_on_top: bool,
    pub change_resolution: bool,
    pub no_buttons: bool,
    pub screen_key: bool,
    pub help_key: bool,
    pub quit_key: bool,
    pub save_key: bool,
    pub screenshot_key: bool,
    pub close_sec: bool,
    pub freeze: bool,
    pub show_progress: bool,
    pub load_transparent: bool,
    pub scale_progress: bool,
    pub display_errors: bool,
    pub write_errors: bool,
    pub abort_errors: bool,
    pub variable_errors: bool,
    pub creation_event_order: bool,
    pub use_front_touch: bool,
    pub use_rear_touch: bool,
    pub use_fast_collision: bool,
    pub fast_collision_compatibility: bool,
    pub disable_sandbox: bool,
    pub enable_copy_on_write: bool,
}

impl GMElement for GMOptionsFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw = reader.read_u64()?;
        Ok(GMOptionsFlags {
            fullscreen: 0 != raw & 0x1,
            interpolate_pixels: 0 != raw & 0x2,
            use_new_audio: 0 != raw & 0x4,
            no_border: 0 != raw & 0x8,
            show_cursor: 0 != raw & 0x10,
            sizeable: 0 != raw & 0x20,
            stay_on_top: 0 != raw & 0x40,
            change_resolution: 0 != raw & 0x80,
            no_buttons: 0 != raw & 0x100,
            screen_key: 0 != raw & 0x200,
            help_key: 0 != raw & 0x400,
            quit_key: 0 != raw & 0x800,
            save_key: 0 != raw & 0x1000,
            screenshot_key: 0 != raw & 0x2000,
            close_sec: 0 != raw & 0x4000,
            freeze: 0 != raw & 0x8000,
            show_progress: 0 != raw & 0x10000,
            load_transparent: 0 != raw & 0x20000,
            scale_progress: 0 != raw & 0x40000,
            display_errors: 0 != raw & 0x80000,
            write_errors: 0 != raw & 0x100000,
            abort_errors: 0 != raw & 0x200000,
            variable_errors: 0 != raw & 0x400000,
            creation_event_order: 0 != raw & 0x800000,
            use_front_touch: 0 != raw & 0x1000000,
            use_rear_touch: 0 != raw & 0x2000000,
            use_fast_collision: 0 != raw & 0x4000000,
            fast_collision_compatibility: 0 != raw & 0x8000000,
            disable_sandbox: 0 != raw & 0x10000000,
            enable_copy_on_write: 0 != raw & 0x20000000,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut raw: u64 = 0;
        raw |= self.fullscreen as u64 * 0x1;
        raw |= self.interpolate_pixels as u64 * 0x2;
        raw |= self.use_new_audio as u64 * 0x4;
        raw |= self.no_border as u64 * 0x8;
        raw |= self.show_cursor as u64 * 0x10;
        raw |= self.sizeable as u64 * 0x20;
        raw |= self.stay_on_top as u64 * 0x40;
        raw |= self.change_resolution as u64 * 0x80;
        raw |= self.no_buttons as u64 * 0x100;
        raw |= self.screen_key as u64 * 0x200;
        raw |= self.help_key as u64 * 0x400;
        raw |= self.quit_key as u64 * 0x800;
        raw |= self.save_key as u64 * 0x1000;
        raw |= self.screenshot_key as u64 * 0x2000;
        raw |= self.close_sec as u64 * 0x4000;
        raw |= self.freeze as u64 * 0x8000;
        raw |= self.show_progress as u64 * 0x10000;
        raw |= self.load_transparent as u64 * 0x20000;
        raw |= self.scale_progress as u64 * 0x40000;
        raw |= self.display_errors as u64 * 0x80000;
        raw |= self.write_errors as u64 * 0x100000;
        raw |= self.abort_errors as u64 * 0x200000;
        raw |= self.variable_errors as u64 * 0x400000;
        raw |= self.creation_event_order as u64 * 0x800000;
        raw |= self.use_front_touch as u64 * 0x1000000;
        raw |= self.use_rear_touch as u64 * 0x2000000;
        raw |= self.use_fast_collision as u64 * 0x4000000;
        raw |= self.fast_collision_compatibility as u64 * 0x8000000;
        raw |= self.disable_sandbox as u64 * 0x10000000;
        raw |= self.enable_copy_on_write as u64 * 0x20000000;
        builder.write_u64(raw);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMOptionsConstant {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}

impl GMElement for GMOptionsConstant {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string(&self.value)?;
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
            use_front_touch: false,
            use_rear_touch: false,
            use_fast_collision: false,
            fast_collision_compatibility: false,
            disable_sandbox: false,
            enable_copy_on_write: false,
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
