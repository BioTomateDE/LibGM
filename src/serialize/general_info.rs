use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::general_info::{GMFunctionClassifications, GMGeneralInfo, GMGeneralInfoFlags, GMOptions, GMOptionsConstant, GMOptionsFlags, GMOptionsWindowColor};
use crate::deserialize::rooms::GMRooms;
use crate::deserialize::texture_page_items::GMTexture;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_gen8(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "GEN8");
    let info: &GMGeneralInfo = &gm_data.general_info;

    builder.write_u8(if info.is_debugger_disabled {1} else {0});
    builder.write_u8(info.bytecode_version);
    builder.write_u16(info.unknown_value);
    data_builder.push_pointer_placeholder(&mut builder, GMPointer::String(info.game_file_name.index))?;
    data_builder.push_pointer_placeholder(&mut builder, GMPointer::String(info.config.index))?;
    builder.write_usize(gm_data.game_objects.game_objects_by_index.len());
    builder.write_usize(get_last_tile_id(&gm_data.rooms));
    builder.write_u32(info.game_id);
    builder.raw_data.extend(info.directplay_guid.as_bytes());
    data_builder.push_pointer_placeholder(&mut builder, GMPointer::String(info.game_name.index))?;
    builder.write_u32(info.major_version);
    builder.write_u32(info.minor_version);
    builder.write_u32(info.release_version);
    builder.write_u32(info.stable_version);
    builder.write_u32(info.default_window_width);
    builder.write_u32(info.default_window_height);
    builder.write_u32(build_general_info_flags(&info.flags));
    builder.write_u32(info.license_crc32);
    builder.raw_data.extend(info.license_md5);
    builder.write_i64(info.timestamp_created.timestamp());
    data_builder.push_pointer_placeholder(&mut builder, GMPointer::String(info.display_name.index))?;
    builder.write_u64(info.active_targets);
    builder.write_u64(build_function_classifications(&info.function_classifications));
    builder.write_i32(info.steam_appid);
    if info.bytecode_version >= 14 {
        builder.write_u32(info.debugger_port.ok_or("General info: debugger port not set")?);
    }

    builder.write_usize(info.room_order.len());
    for room_id in &info.room_order {
        builder.write_u32(*room_id);
    }

    builder.finish(data_builder)?;
    Ok(())
}


fn get_last_tile_id(rooms: &GMRooms) -> usize {
    let mut tile_id: usize = 10_000_000;
    for room in &rooms.rooms_by_index {
        tile_id += room.tiles.len();
    }
    tile_id
}


fn build_general_info_flags(flags: &GMGeneralInfoFlags) -> u32 {
    let mut raw: u32 = 0;

    if flags.fullscreen {raw |= 0x0001};
    if flags.sync_vertex1 {raw |= 0x0002};
    if flags.sync_vertex2 {raw |= 0x0004};
    if flags.sync_vertex3 {raw |= 0x0100};
    if flags.interpolate {raw |= 0x0008};
    if flags.scale {raw |= 0x0010};
    if flags.show_cursor {raw |= 0x0020};
    if flags.sizeable {raw |= 0x0040};
    if flags.screen_key {raw |= 0x0080};
    if flags.studio_version_b1 {raw |= 0x0200};
    if flags.studio_version_b2 {raw |= 0x0400};
    if flags.studio_version_b3 {raw |= 0x0800};
    if flags.steam_enabled {raw |= 0x1000};
    if flags.local_data_enabled {raw |= 0x2000};
    if flags.borderless_window {raw |= 0x4000};
    if flags.javascript_mode {raw |= 0x8000};

    raw
}

fn build_function_classifications(function_classifications: &GMFunctionClassifications) -> u64 {
    let mut raw: u64 = 0;

    if function_classifications.none {raw |= 0x0};
    if function_classifications.internet {raw |= 0x1};
    if function_classifications.joystick {raw |= 0x2};
    if function_classifications.gamepad {raw |= 0x4};
    if function_classifications.immersion {raw |= 0x8};
    if function_classifications.screengrab {raw |= 0x10};
    if function_classifications.math {raw |= 0x20};
    if function_classifications.action {raw |= 0x40};
    if function_classifications.matrix_d3d {raw |= 0x80};
    if function_classifications.d3dmodel {raw |= 0x100};
    if function_classifications.data_structures {raw |= 0x200};
    if function_classifications.file {raw |= 0x400};
    if function_classifications.ini {raw |= 0x800};
    if function_classifications.filename {raw |= 0x1000};
    if function_classifications.directory {raw |= 0x2000};
    if function_classifications.environment {raw |= 0x4000};
    if function_classifications.unused1 {raw |= 0x8000};
    if function_classifications.http {raw |= 0x10000};
    if function_classifications.encoding {raw |= 0x20000};
    if function_classifications.uidialog {raw |= 0x40000};
    if function_classifications.motion_planning {raw |= 0x80000};
    if function_classifications.shape_collision {raw |= 0x100000};
    if function_classifications.instance {raw |= 0x200000};
    if function_classifications.room {raw |= 0x400000};
    if function_classifications.game {raw |= 0x800000};
    if function_classifications.display {raw |= 0x1000000};
    if function_classifications.device {raw |= 0x2000000};
    if function_classifications.window {raw |= 0x4000000};
    if function_classifications.draw_color {raw |= 0x8000000};
    if function_classifications.texture {raw |= 0x10000000};
    if function_classifications.layer {raw |= 0x20000000};
    if function_classifications.string {raw |= 0x40000000};
    if function_classifications.tiles {raw |= 0x80000000};
    if function_classifications.surface {raw |= 0x100000000};
    if function_classifications.skeleton {raw |= 0x200000000};
    if function_classifications.io {raw |= 0x400000000};
    if function_classifications.variables {raw |= 0x800000000};
    if function_classifications.array {raw |= 0x1000000000};
    if function_classifications.external_call {raw |= 0x2000000000};
    if function_classifications.notification {raw |= 0x4000000000};
    if function_classifications.date {raw |= 0x8000000000};
    if function_classifications.particle {raw |= 0x10000000000};
    if function_classifications.sprite {raw |= 0x20000000000};
    if function_classifications.clickable {raw |= 0x40000000000};
    if function_classifications.legacy_sound {raw |= 0x80000000000};
    if function_classifications.audio {raw |= 0x100000000000};
    if function_classifications.event {raw |= 0x200000000000};
    if function_classifications.unused2 {raw |= 0x400000000000};
    if function_classifications.free_type {raw |= 0x800000000000};
    if function_classifications.analytics {raw |= 0x1000000000000};
    if function_classifications.unused3 {raw |= 0x2000000000000};
    if function_classifications.unused4 {raw |= 0x4000000000000};
    if function_classifications.achievement {raw |= 0x8000000000000};
    if function_classifications.cloud_saving {raw |= 0x10000000000000};
    if function_classifications.ads {raw |= 0x20000000000000};
    if function_classifications.os {raw |= 0x40000000000000};
    if function_classifications.iap {raw |= 0x80000000000000};
    if function_classifications.facebook {raw |= 0x100000000000000};
    if function_classifications.physics {raw |= 0x200000000000000};
    if function_classifications.flash_aa {raw |= 0x400000000000000};
    if function_classifications.console {raw |= 0x800000000000000};
    if function_classifications.buffer {raw |= 0x1000000000000000};
    if function_classifications.steam {raw |= 0x2000000000000000};
    if function_classifications.unused5 {raw |= 2310346608841064448};
    if function_classifications.shaders {raw |= 0x4000000000000000};
    if function_classifications.vertex_buffers {raw |= 9223372036854775808};

    raw
}


pub fn build_chunk_optn(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "OPTN");

    if gm_data.options.is_new_format {
        build_options_new(data_builder, &mut builder, &gm_data.options)?;
    } else {
        build_options_old(data_builder, &mut builder, &gm_data.options)?;
    }

    builder.finish(data_builder)?;
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


fn build_options_old(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, options: &GMOptions) -> Result<(), String> {
    builder.write_bool32(options.flags.fullscreen);
    builder.write_bool32(options.flags.interpolate_pixels);
    builder.write_bool32(options.flags.use_new_audio);
    builder.write_bool32(options.flags.no_border);
    builder.write_bool32(options.flags.show_cursor);

    builder.write_i32(options.scale);

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

    build_options_image(data_builder, builder, &options.back_image)?;
    build_options_image(data_builder, builder, &options.front_image)?;
    build_options_image(data_builder, builder, &options.load_image)?;

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


fn build_options_new(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, options: &GMOptions) -> Result<(), String> {
    builder.write_u32(options.unknown1);
    builder.write_u32(options.unknown2);
    builder.write_u64(build_options_flags_new(&options.flags));
    builder.write_i32(options.scale);
    build_options_window_color(builder, &options.window_color);
    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);
    builder.write_u32(options.vertex_sync);
    builder.write_u32(options.priority);
    build_options_image(data_builder, builder, &options.back_image)?;
    build_options_image(data_builder, builder, &options.front_image)?;
    build_options_image(data_builder, builder, &options.load_image)?;
    builder.write_u32(options.load_alpha);
    build_constants(data_builder, builder, &options.constants)?;
    Ok(())
}


fn build_options_image(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, texture: &Option<GMRef<GMTexture>>) -> Result<(), String> {
    match texture {
        None => builder.write_usize(0),
        Some(reference) => data_builder.push_pointer_placeholder(builder, GMPointer::Texture(reference.index))?
    }
    Ok(())
}


fn build_options_window_color(builder: &mut ChunkBuilder, window_color: &GMOptionsWindowColor) {
    builder.write_u8(window_color.r);
    builder.write_u8(window_color.g);
    builder.write_u8(window_color.b);
    builder.write_u8(window_color.a);
}


fn build_constants(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, constants: &Vec<GMOptionsConstant>) -> Result<(), String> {
    builder.write_usize(constants.len());

    for constant in constants {
        builder.write_gm_string(data_builder, &constant.name)?;
        builder.write_gm_string(data_builder, &constant.value)?;
    }

    Ok(())
}

