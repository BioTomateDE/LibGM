use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::strings::GMStrings;
use crate::deserialize::texture_page_items::{GMTexturePageItem, GMTexturePageItems};

#[derive(Debug, Clone)]
pub struct GMOptions {
    pub is_new_format: bool,
    pub unknown1: u32,
    pub unknown2: u32,
    pub flags: GMOptionsFlags,
    pub window_scale: i32,
    pub window_color: GMOptionsWindowColor,
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMOptionsWindowColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMOptionsConstant {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}


pub fn parse_chunk_optn(chunk: &mut GMChunk, strings: &GMStrings, textures: &GMTexturePageItems) -> Result<GMOptions, String> {
    chunk.cur_pos = 0;
    let is_new_format: bool = chunk.read_u32()? == 0x80000000;
    chunk.cur_pos = 0;

    let options: GMOptions = if is_new_format {
        parse_options_new(chunk, strings, textures)?
    } else {
        parse_options_old(chunk, strings, textures)?
    };
    Ok(options)
}


fn parse_options_new(chunk: &mut GMChunk, strings: &GMStrings, textures: &GMTexturePageItems) -> Result<GMOptions, String> {
    let unknown1: u32 = chunk.read_u32()?;
    let unknown2: u32 = chunk.read_u32()?;
    let flags: GMOptionsFlags = parse_options_flags(chunk.read_u64()?);
    let scale: i32 = chunk.read_i32()?;
    let window_color: GMOptionsWindowColor = parse_options_window_color(chunk)?;
    let color_depth: u32 = chunk.read_u32()?;
    let resolution: u32 = chunk.read_u32()?;
    let frequency: u32 = chunk.read_u32()?;
    let vertex_sync: u32 = chunk.read_u32()?;
    let priority: u32 = chunk.read_u32()?;
    let back_image: Option<GMRef<GMTexturePageItem>> = parse_options_image(chunk, textures)?;
    let front_image: Option<GMRef<GMTexturePageItem>> = parse_options_image(chunk, textures)?;
    let load_image: Option<GMRef<GMTexturePageItem>> = parse_options_image(chunk, textures)?;
    let load_alpha: u32 = chunk.read_u32()?;
    let constants: Vec<GMOptionsConstant> = parse_constants(chunk, strings)?;

    Ok(GMOptions {
        is_new_format: true,
        unknown1,
        unknown2,
        flags,
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
    })
}

fn parse_options_old(chunk: &mut GMChunk, strings: &GMStrings, textures: &GMTexturePageItems) -> Result<GMOptions, String> {
    let flag_fullscreen: bool = chunk.read_bool32()?;
    let flag_interpolate_pixels: bool = chunk.read_bool32()?;
    let flag_use_new_audio: bool = chunk.read_bool32()?;
    let flag_no_border: bool = chunk.read_bool32()?;
    let flag_show_cursor: bool = chunk.read_bool32()?;

    let scale: i32 = chunk.read_i32()?;

    let flag_sizeable: bool = chunk.read_bool32()?;
    let flag_stay_on_top: bool = chunk.read_bool32()?;

    let window_color: GMOptionsWindowColor = parse_options_window_color(chunk)?;

    let flag_change_resolution: bool = chunk.read_bool32()?;

    let color_depth: u32 = chunk.read_u32()?;
    let resolution: u32 = chunk.read_u32()?;
    let frequency: u32 = chunk.read_u32()?;

    let flag_no_buttons: bool = chunk.read_bool32()?;

    let vertex_sync: u32 = chunk.read_u32()?;

    let flag_screen_key: bool = chunk.read_bool32()?;
    let flag_help_key: bool = chunk.read_bool32()?;
    let flag_quit_key: bool = chunk.read_bool32()?;
    let flag_save_key: bool = chunk.read_bool32()?;
    let flag_screenshot_key: bool = chunk.read_bool32()?;
    let flag_close_sec: bool = chunk.read_bool32()?;

    let priority: u32 = chunk.read_u32()?;

    let flag_freeze: bool = chunk.read_bool32()?;
    let flag_show_progress: bool = chunk.read_bool32()?;

    let back_image: Option<GMRef<GMTexturePageItem>> = parse_options_image(chunk, textures)?;
    let front_image: Option<GMRef<GMTexturePageItem>> = parse_options_image(chunk, textures)?;
    let load_image: Option<GMRef<GMTexturePageItem>> = parse_options_image(chunk, textures)?;

    let flag_load_transparent: bool = chunk.read_bool32()?;

    let load_alpha: u32 = chunk.read_u32()?;

    let flag_scale_progress: bool = chunk.read_bool32()?;
    let flag_display_errors: bool = chunk.read_bool32()?;
    let flag_write_errors: bool = chunk.read_bool32()?;
    let flag_abort_errors: bool = chunk.read_bool32()?;
    let flag_variable_errors: bool = chunk.read_bool32()?;
    let flag_creation_event_order: bool = chunk.read_bool32()?;

    let constants: Vec<GMOptionsConstant> = parse_constants(chunk, strings)?;

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
    })
}

fn parse_options_flags(raw: u64) -> GMOptionsFlags {
    GMOptionsFlags {
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
    }
}

fn parse_constants(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMOptionsConstant>, String> {
    let constants_count: usize = chunk.read_usize_count()?;
    let mut constants: Vec<GMOptionsConstant> = Vec::with_capacity(constants_count);

    for _ in 0..constants_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let value: GMRef<String> = chunk.read_gm_string(strings)?;
        constants.push(GMOptionsConstant {
            name,
            value,
        })
    }

    Ok(constants)
}

fn parse_options_image(chunk: &mut GMChunk, textures: &GMTexturePageItems) -> Result<Option<GMRef<GMTexturePageItem>>, String> {
    let absolute_position: usize = chunk.read_usize()?;
    if absolute_position == 0 {
        return Ok(None)
    }

    let texture: GMRef<GMTexturePageItem> = textures.abs_pos_to_ref.get(&absolute_position)
        .ok_or_else(|| format!("Could not get Options image with absolute texture position {absolute_position}"))?
        .clone();

    Ok(Some(texture))
}

fn parse_options_window_color(chunk: &mut GMChunk) -> Result<GMOptionsWindowColor, String> {
    // TODO check if rgba or abgr
    let window_color_r: u8 = chunk.read_u8()?;
    let window_color_g: u8 = chunk.read_u8()?;
    let window_color_b: u8 = chunk.read_u8()?;
    let window_color_a: u8 = chunk.read_u8()?;
    let window_color = GMOptionsWindowColor {
        r: window_color_r,
        g: window_color_g,
        b: window_color_b,
        a: window_color_a,
    };
    Ok(window_color)
}

