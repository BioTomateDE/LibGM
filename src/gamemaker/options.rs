use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::texture_page_items::GMTexturePageItem;
use crate::gm_serialize::DataBuilder;

#[derive(Debug, Clone)]
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
    pub vertex_sync: u32,
    pub priority: u32,
    pub back_image: Option<GMRef<GMTexturePageItem>>,
    pub front_image: Option<GMRef<GMTexturePageItem>>,
    pub load_image: Option<GMRef<GMTexturePageItem>>,
    pub load_alpha: u32,
    pub constants: Vec<GMOptionsConstant>,
    pub exists: bool,
}
impl GMChunkElement for GMOptions {
    /// probably shouldn't be used other than as a stub
    fn empty() -> Self {
        Self {
            is_new_format: false,
            unknown1: 69420,
            unknown2: 69420,
            flags: GMOptionsFlags {
                fullscreen: false,
                interpolate_pixels: false,
                use_new_audio: false,
                no_border: false,
                show_cursor: false,
                sizeable: false,
                stay_on_top: false,
                change_resolution: false,
                no_buttons: false,
                screen_key: false,
                help_key: false,
                quit_key: false,
                save_key: false,
                screenshot_key: false,
                close_sec: false,
                freeze: false,
                show_progress: false,
                load_transparent: false,
                scale_progress: false,
                display_errors: false,
                write_errors: false,
                abort_errors: false,
                variable_errors: false,
                creation_event_order: false,
                use_front_touch: false,
                use_rear_touch: false,
                use_fast_collision: false,
                fast_collision_compatibility: false,
                disable_sandbox: false,
                enable_copy_on_write: false,
            },
            window_scale: 69420,
            window_color: 69420,
            color_depth: 69420,
            resolution: 69420,
            frequency: 69420,
            vertex_sync: 69420,
            priority: 69420,
            back_image: None,
            front_image: None,
            load_image: None,
            load_alpha: 69420,
            constants: vec![],
            exists: false,
        }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMOptions {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let is_new_format: bool = reader.read_u32()? == 0x80000000;
        reader.cur_pos -= 4;
        if is_new_format {
            parse_options_new(reader)
        } else {
            parse_options_old(reader)
        }
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        if self.is_new_format {
            build_options_new(builder, self)?;
        } else {
            build_options_old(builder, self)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone)]
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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let mut raw: u64 = 0;
        if self.fullscreen {raw |= 0x1};
        if self.interpolate_pixels {raw |= 0x2};
        if self.use_new_audio {raw |= 0x4};
        if self.no_border {raw |= 0x8};
        if self.show_cursor {raw |= 0x10};
        if self.sizeable {raw |= 0x20};
        if self.stay_on_top {raw |= 0x40};
        if self.change_resolution {raw |= 0x80};
        if self.no_buttons {raw |= 0x100};
        if self.screen_key {raw |= 0x200};
        if self.help_key {raw |= 0x400};
        if self.quit_key {raw |= 0x800};
        if self.save_key {raw |= 0x1000};
        if self.screenshot_key {raw |= 0x2000};
        if self.close_sec {raw |= 0x4000};
        if self.freeze {raw |= 0x8000};
        if self.show_progress {raw |= 0x10000};
        if self.load_transparent {raw |= 0x20000};
        if self.scale_progress {raw |= 0x40000};
        if self.display_errors {raw |= 0x80000};
        if self.write_errors {raw |= 0x100000};
        if self.abort_errors {raw |= 0x200000};
        if self.variable_errors {raw |= 0x400000};
        if self.creation_event_order {raw |= 0x800000};
        if self.use_front_touch {raw |= 0x1000000};
        if self.use_rear_touch {raw |= 0x2000000};
        if self.use_fast_collision {raw |= 0x4000000};
        if self.fast_collision_compatibility {raw |= 0x8000000};
        if self.disable_sandbox {raw |= 0x10000000};
        if self.enable_copy_on_write {raw |= 0x20000000};
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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        Ok(GMOptionsConstant { name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string(&self.value)?;
        Ok(())
    }
}


fn parse_options_new(reader: &mut DataReader) -> Result<GMOptions, String> {
    let unknown1: u32 = reader.read_u32()?;
    let unknown2: u32 = reader.read_u32()?;
    let flags = GMOptionsFlags::deserialize(reader)?;
    let window_scale: i32 = reader.read_i32()?;
    let window_color: u32 = reader.read_u32()?;
    let color_depth: u32 = reader.read_u32()?;
    let resolution: u32 = reader.read_u32()?;
    let frequency: u32 = reader.read_u32()?;
    let vertex_sync: u32 = reader.read_u32()?;
    let priority: u32 = reader.read_u32()?;
    let back_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let front_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_alpha: u32 = reader.read_u32()?;
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


fn parse_options_old(reader: &mut DataReader) -> Result<GMOptions, String> {
    let flag_fullscreen: bool = reader.read_bool32()?;
    let flag_interpolate_pixels: bool = reader.read_bool32()?;
    let flag_use_new_audio: bool = reader.read_bool32()?;
    let flag_no_border: bool = reader.read_bool32()?;
    let flag_show_cursor: bool = reader.read_bool32()?;

    let scale: i32 = reader.read_i32()?;

    let flag_sizeable: bool = reader.read_bool32()?;
    let flag_stay_on_top: bool = reader.read_bool32()?;

    let window_color: u32 = reader.read_u32()?;

    let flag_change_resolution: bool = reader.read_bool32()?;

    let color_depth: u32 = reader.read_u32()?;
    let resolution: u32 = reader.read_u32()?;
    let frequency: u32 = reader.read_u32()?;

    let flag_no_buttons: bool = reader.read_bool32()?;

    let vertex_sync: u32 = reader.read_u32()?;

    let flag_screen_key: bool = reader.read_bool32()?;
    let flag_help_key: bool = reader.read_bool32()?;
    let flag_quit_key: bool = reader.read_bool32()?;
    let flag_save_key: bool = reader.read_bool32()?;
    let flag_screenshot_key: bool = reader.read_bool32()?;
    let flag_close_sec: bool = reader.read_bool32()?;

    let priority: u32 = reader.read_u32()?;

    let flag_freeze: bool = reader.read_bool32()?;
    let flag_show_progress: bool = reader.read_bool32()?;

    let back_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let front_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;

    let flag_load_transparent: bool = reader.read_bool32()?;

    let load_alpha: u32 = reader.read_u32()?;

    let flag_scale_progress: bool = reader.read_bool32()?;
    let flag_display_errors: bool = reader.read_bool32()?;
    let flag_write_errors: bool = reader.read_bool32()?;
    let flag_abort_errors: bool = reader.read_bool32()?;
    let flag_variable_errors: bool = reader.read_bool32()?;
    let flag_creation_event_order: bool = reader.read_bool32()?;

    let constants: Vec<GMOptionsConstant> = reader.read_simple_list()?;

    Ok(GMOptions {
        is_new_format: false,
        unknown1: 0,     // might not be best practice?
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


fn build_options_old(builder: &mut DataBuilder, options: &GMOptions) -> Result<(), String> {
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

    builder.write_u32(options.vertex_sync);

    builder.write_bool32(options.flags.screen_key);
    builder.write_bool32(options.flags.help_key);
    builder.write_bool32(options.flags.quit_key);
    builder.write_bool32(options.flags.save_key);
    builder.write_bool32(options.flags.screenshot_key);
    builder.write_bool32(options.flags.close_sec);

    builder.write_u32(options.priority);

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


fn build_options_new(builder: &mut DataBuilder, options: &GMOptions) -> Result<(), String> {
    builder.write_u32(options.unknown1);
    builder.write_u32(options.unknown2);
    options.flags.serialize(builder)?;
    builder.write_i32(options.window_scale);
    builder.write_u32(options.window_color);
    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);
    builder.write_u32(options.vertex_sync);
    builder.write_u32(options.priority);
    builder.write_pointer_opt(&options.back_image)?;
    builder.write_pointer_opt(&options.front_image)?;
    builder.write_pointer_opt(&options.load_image)?;
    builder.write_u32(options.load_alpha);
    builder.write_simple_list(&options.constants)?;
    Ok(())
}


