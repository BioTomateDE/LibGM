use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::options::{GMOptions, GMOptionsConstant, GMOptionsFlags, GMOptionsWindowColor};
use crate::deserialize::texture_page_items::GMTexturePageItem;
use crate::serialize::chunk_writing::{DataBuilder, DataPlaceholder};

pub fn build_chunk_optn(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("OPTN")?;

    if gm_data.options.is_new_format {
        build_options_new(builder, &gm_data.options)?;
    } else {
        build_options_old(builder, &gm_data.options)?;
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

fn build_options_flags_new(flags: &GMOptionsFlags) -> u64 {
    let mut raw: u64 = 0;
    if flags.fullscreen {raw |= 0x1};
    if flags.interpolate_pixels {raw |= 0x2};
    if flags.use_new_audio {raw |= 0x4};
    if flags.no_border {raw |= 0x8};
    if flags.show_cursor {raw |= 0x10};
    if flags.sizeable {raw |= 0x20};
    if flags.stay_on_top {raw |= 0x40};
    if flags.change_resolution {raw |= 0x80};
    if flags.no_buttons {raw |= 0x100};
    if flags.screen_key {raw |= 0x200};
    if flags.help_key {raw |= 0x400};
    if flags.quit_key {raw |= 0x800};
    if flags.save_key {raw |= 0x1000};
    if flags.screenshot_key {raw |= 0x2000};
    if flags.close_sec {raw |= 0x4000};
    if flags.freeze {raw |= 0x8000};
    if flags.show_progress {raw |= 0x10000};
    if flags.load_transparent {raw |= 0x20000};
    if flags.scale_progress {raw |= 0x40000};
    if flags.display_errors {raw |= 0x80000};
    if flags.write_errors {raw |= 0x100000};
    if flags.abort_errors {raw |= 0x200000};
    if flags.variable_errors {raw |= 0x400000};
    if flags.creation_event_order {raw |= 0x800000};
    if flags.use_front_touch {raw |= 0x1000000};
    if flags.use_rear_touch {raw |= 0x2000000};
    if flags.use_fast_collision {raw |= 0x4000000};
    if flags.fast_collision_compatibility {raw |= 0x8000000};
    if flags.disable_sandbox {raw |= 0x10000000};
    if flags.enable_copy_on_write {raw |= 0x20000000};
    raw
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

    build_options_window_color(builder, &options.window_color);

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

    build_options_image(builder, &options.back_image)?;
    build_options_image(builder, &options.front_image)?;
    build_options_image(builder, &options.load_image)?;

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
    builder.write_u64(build_options_flags_new(&options.flags));
    builder.write_i32(options.window_scale);
    build_options_window_color(builder, &options.window_color);
    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);
    builder.write_u32(options.vertex_sync);
    builder.write_u32(options.priority);
    build_options_image(builder, &options.back_image)?;
    build_options_image(builder, &options.front_image)?;
    build_options_image(builder, &options.load_image)?;
    builder.write_u32(options.load_alpha);
    build_constants(builder, &options.constants)?;
    Ok(())
}


fn build_options_image(builder: &mut DataBuilder, texture: &Option<GMRef<GMTexturePageItem>>) -> Result<(), String> {
    match texture {
        None => builder.write_usize(0),
        Some(reference) => builder.write_placeholder(DataPlaceholder::TexturePageItem(reference.index))?
    }
    Ok(())
}

fn build_options_window_color(builder: &mut DataBuilder, window_color: &GMOptionsWindowColor) {
    builder.write_u8(window_color.r);
    builder.write_u8(window_color.g);
    builder.write_u8(window_color.b);
    builder.write_u8(window_color.a);
}

fn build_constants(builder: &mut DataBuilder, constants: &Vec<GMOptionsConstant>) -> Result<(), String> {
    builder.write_usize(constants.len());

    for constant in constants {
        builder.write_gm_string(&constant.name)?;
        builder.write_gm_string(&constant.value)?;
    }

    Ok(())
}

