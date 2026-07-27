// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::options::Constant;
use crate::wad::elem::options::OptionFlags;
use crate::wad::elem::options::Options;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

pub fn deserialize(reader: &mut DataReader) -> Result<Options> {
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

pub fn serialize(writer: &mut DataWriter, options: &Options) -> Result<()> {
    writer.write_bool32(options.flags.contains(OptionFlags::FULLSCREEN));
    writer.write_bool32(options.flags.contains(OptionFlags::INTERPOLATE_PIXELS));
    writer.write_bool32(options.flags.contains(OptionFlags::USE_NEW_AUDIO));
    writer.write_bool32(options.flags.contains(OptionFlags::NO_BORDER));
    writer.write_bool32(options.flags.contains(OptionFlags::SHOW_CURSOR));

    writer.write_i32(options.window_scale);

    writer.write_bool32(options.flags.contains(OptionFlags::SIZEABLE));
    writer.write_bool32(options.flags.contains(OptionFlags::STAY_ON_TOP));

    writer.write_u32(options.window_color);

    writer.write_bool32(options.flags.contains(OptionFlags::CHANGE_RESOLUTION));

    writer.write_u32(options.color_depth);
    writer.write_u32(options.resolution);
    writer.write_u32(options.frequency);

    writer.write_bool32(options.flags.contains(OptionFlags::NO_BUTTONS));

    writer.write_i32(options.vertex_sync);

    writer.write_bool32(options.flags.contains(OptionFlags::SCREEN_KEY));
    writer.write_bool32(options.flags.contains(OptionFlags::HELP_KEY));
    writer.write_bool32(options.flags.contains(OptionFlags::QUIT_KEY));
    writer.write_bool32(options.flags.contains(OptionFlags::SAVE_KEY));
    writer.write_bool32(options.flags.contains(OptionFlags::SCREENSHOT_KEY));
    writer.write_bool32(options.flags.contains(OptionFlags::CLOSE_SEC));

    writer.write_i32(options.priority);

    writer.write_bool32(options.flags.contains(OptionFlags::FREEZE));
    writer.write_bool32(options.flags.contains(OptionFlags::SHOW_PROGRESS));

    writer.write_gm_texture(options.back_image)?;
    writer.write_gm_texture(options.front_image)?;
    writer.write_gm_texture(options.load_image)?;

    writer.write_bool32(options.flags.contains(OptionFlags::LOAD_TRANSPARENT));

    writer.write_u32(options.load_alpha);

    writer.write_bool32(options.flags.contains(OptionFlags::SCALE_PROGRESS));
    writer.write_bool32(options.flags.contains(OptionFlags::DISPLAY_ERRORS));
    writer.write_bool32(options.flags.contains(OptionFlags::WRITE_ERRORS));
    writer.write_bool32(options.flags.contains(OptionFlags::ABORT_ERRORS));
    writer.write_bool32(options.flags.contains(OptionFlags::VARIABLE_ERRORS));
    writer.write_bool32(options.flags.contains(OptionFlags::CREATION_EVENT_ORDER));

    writer.write_simple_list(&options.constants)?;
    Ok(())
}
