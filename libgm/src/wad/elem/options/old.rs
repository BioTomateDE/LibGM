// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::options::Constant;
use crate::wad::elem::options::OptionFlags;
use crate::wad::elem::options::Options;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

pub fn parse(reader: &mut DataReader) -> Result<Options> {
    let flag_fullscreen = reader.read_bool32()?;
    let flag_interpolate_pixels = reader.read_bool32()?;
    let flag_use_new_audio = reader.read_bool32()?;
    let flag_no_border = reader.read_bool32()?;
    let flag_show_cursor = reader.read_bool32()?;

    let window_scale = reader.read_i32()?;

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

    let back_image: GMRef<TexturePageItem> = reader.read_gm_texture()?;
    let front_image: GMRef<TexturePageItem> = reader.read_gm_texture()?;
    let load_image: GMRef<TexturePageItem> = reader.read_gm_texture()?;

    let flag_load_transparent = reader.read_bool32()?;

    let load_alpha = reader.read_u32()?;

    let flag_scale_progress = reader.read_bool32()?;
    let flag_display_errors = reader.read_bool32()?;
    let flag_write_errors = reader.read_bool32()?;
    let flag_abort_errors = reader.read_bool32()?;
    let flag_variable_errors = reader.read_bool32()?;
    let flag_creation_event_order = reader.read_bool32()?;

    let constants: Vec<Constant> = reader.read_simple_list()?;

    let flags: OptionFlags = OptionFlags::empty()
        | f(flag_fullscreen, OptionFlags::FULLSCREEN)
        | f(flag_interpolate_pixels, OptionFlags::INTERPOLATE_PIXELS)
        | f(flag_interpolate_pixels, OptionFlags::INTERPOLATE_PIXELS)
        | f(flag_use_new_audio, OptionFlags::USE_NEW_AUDIO)
        | f(flag_no_border, OptionFlags::NO_BORDER)
        | f(flag_show_cursor, OptionFlags::SHOW_CURSOR)
        | f(flag_sizeable, OptionFlags::SIZEABLE)
        | f(flag_stay_on_top, OptionFlags::STAY_ON_TOP)
        | f(flag_change_resolution, OptionFlags::CHANGE_RESOLUTION)
        | f(flag_no_buttons, OptionFlags::NO_BUTTONS)
        | f(flag_screen_key, OptionFlags::SCREEN_KEY)
        | f(flag_help_key, OptionFlags::HELP_KEY)
        | f(flag_quit_key, OptionFlags::QUIT_KEY)
        | f(flag_save_key, OptionFlags::SAVE_KEY)
        | f(flag_screenshot_key, OptionFlags::SCREENSHOT_KEY)
        | f(flag_close_sec, OptionFlags::CLOSE_SEC)
        | f(flag_freeze, OptionFlags::FREEZE)
        | f(flag_show_progress, OptionFlags::SHOW_PROGRESS)
        | f(flag_load_transparent, OptionFlags::LOAD_TRANSPARENT)
        | f(flag_scale_progress, OptionFlags::SCALE_PROGRESS)
        | f(flag_display_errors, OptionFlags::DISPLAY_ERRORS)
        | f(flag_write_errors, OptionFlags::WRITE_ERRORS)
        | f(flag_abort_errors, OptionFlags::ABORT_ERRORS)
        | f(flag_variable_errors, OptionFlags::VARIABLE_ERRORS)
        | f(flag_creation_event_order, OptionFlags::CREATION_EVENT_ORDER);

    Ok(Options {
        is_new_format: false,
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
    })
}

const fn f(is: bool, flag: OptionFlags) -> OptionFlags {
    if is { flag } else { OptionFlags::empty() }
}

pub fn build(builder: &mut DataBuilder, options: &Options) -> Result<()> {
    builder.write_bool32(options.flags.contains(OptionFlags::FULLSCREEN));
    builder.write_bool32(options.flags.contains(OptionFlags::INTERPOLATE_PIXELS));
    builder.write_bool32(options.flags.contains(OptionFlags::USE_NEW_AUDIO));
    builder.write_bool32(options.flags.contains(OptionFlags::NO_BORDER));
    builder.write_bool32(options.flags.contains(OptionFlags::SHOW_CURSOR));

    builder.write_i32(options.window_scale);

    builder.write_bool32(options.flags.contains(OptionFlags::SIZEABLE));
    builder.write_bool32(options.flags.contains(OptionFlags::STAY_ON_TOP));

    builder.write_u32(options.window_color);

    builder.write_bool32(options.flags.contains(OptionFlags::CHANGE_RESOLUTION));

    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);

    builder.write_bool32(options.flags.contains(OptionFlags::NO_BUTTONS));

    builder.write_i32(options.vertex_sync);

    builder.write_bool32(options.flags.contains(OptionFlags::SCREEN_KEY));
    builder.write_bool32(options.flags.contains(OptionFlags::HELP_KEY));
    builder.write_bool32(options.flags.contains(OptionFlags::QUIT_KEY));
    builder.write_bool32(options.flags.contains(OptionFlags::SAVE_KEY));
    builder.write_bool32(options.flags.contains(OptionFlags::SCREENSHOT_KEY));
    builder.write_bool32(options.flags.contains(OptionFlags::CLOSE_SEC));

    builder.write_i32(options.priority);

    builder.write_bool32(options.flags.contains(OptionFlags::FREEZE));
    builder.write_bool32(options.flags.contains(OptionFlags::SHOW_PROGRESS));

    builder.write_gm_texture(options.back_image)?;
    builder.write_gm_texture(options.front_image)?;
    builder.write_gm_texture(options.load_image)?;

    builder.write_bool32(options.flags.contains(OptionFlags::LOAD_TRANSPARENT));

    builder.write_u32(options.load_alpha);

    builder.write_bool32(options.flags.contains(OptionFlags::SCALE_PROGRESS));
    builder.write_bool32(options.flags.contains(OptionFlags::DISPLAY_ERRORS));
    builder.write_bool32(options.flags.contains(OptionFlags::WRITE_ERRORS));
    builder.write_bool32(options.flags.contains(OptionFlags::ABORT_ERRORS));
    builder.write_bool32(options.flags.contains(OptionFlags::VARIABLE_ERRORS));
    builder.write_bool32(options.flags.contains(OptionFlags::CREATION_EVENT_ORDER));

    builder.write_simple_list(&options.constants)?;
    Ok(())
}
