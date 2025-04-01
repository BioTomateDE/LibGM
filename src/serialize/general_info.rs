use crate::deserialize::all::UTData;
use crate::deserialize::general_info::{UTFunctionClassifications, UTGeneralInfoFlags, UTOptionsFlags};
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::ChunkBuilder;

#[allow(non_snake_case)]
pub fn build_chunk_GEN8(data_builder: &mut DataBuilder, ut_data: &UTData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "GEN8" };

    builder.write_bool(ut_data.general_info.is_debugger_disabled)?;
    builder.write_u8(ut_data.general_info.bytecode_version)?;
    builder.write_u16(ut_data.general_info.unknown_value)?;
    builder.write_string(&ut_data.general_info.game_file_name.resolve(&ut_data.strings)?)?;
    builder.write_string(&ut_data.general_info.config.resolve(&ut_data.strings)?)?;
    builder.write_u32(ut_data.general_info.last_object_id)?;
    builder.write_u32(ut_data.general_info.last_tile_id)?;
    builder.write_u32(ut_data.general_info.game_id)?;
    builder.write_string(&ut_data.general_info.directplay_guid.hyphenated().to_string())?;
    builder.write_string(&ut_data.general_info.game_name.resolve(&ut_data.strings)?)?;
    builder.write_u32(ut_data.general_info.major_version)?;
    builder.write_u32(ut_data.general_info.minor_version)?;
    builder.write_u32(ut_data.general_info.release_version)?;
    builder.write_u32(ut_data.general_info.stable_version)?;
    builder.write_u32(ut_data.general_info.default_window_width)?;
    builder.write_u32(ut_data.general_info.default_window_height)?;
    builder.write_u64(build_general_info_flags(&ut_data.general_info.flags))?;
    builder.raw_data.extend(ut_data.general_info.license);
    builder.write_i64(ut_data.general_info.timestamp_created.timestamp())?;
    builder.write_string(&ut_data.general_info.display_name.resolve(&ut_data.strings)?)?;
    builder.write_u64(ut_data.general_info.active_targets)?;    // scuffed offsets
    builder.write_u64(build_function_classifications(&ut_data.general_info.function_classifications))?;
    builder.write_i32(-(ut_data.general_info.steam_appid as i32))?;
    builder.write_u32(ut_data.general_info.debugger_port as u32)?;
    builder.write_usize(ut_data.general_info.room_order.len())?;
    for room_id in &ut_data.general_info.room_order {
        builder.write_u32(*room_id)?;
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}


fn build_general_info_flags(flags: &UTGeneralInfoFlags) -> u64 {
    let mut raw: u64 = 0;

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

fn build_function_classifications(function_classifications: &UTFunctionClassifications) -> u64 {
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
    if function_classifications._unused1 {raw |= 0x8000};
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
    if function_classifications._unused2 {raw |= 0x400000000000};
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
    if function_classifications._unused3 {raw |= 2310346608841064448};
    if function_classifications.shaders {raw |= 0x4000000000000000};
    if function_classifications.vertex_buffers {raw |= 9223372036854775808};

    raw
}


#[allow(non_snake_case)]
pub fn build_chunk_OPTN(data_builder: &mut DataBuilder, ut_data: &UTData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "OPTN" };

    builder.write_u32(ut_data.options._unused1)?;
    builder.write_u32(ut_data.options._unused2)?;
    builder.write_u64(build_options_flags(&ut_data.options.flags))?;
    builder.write_i32(ut_data.options.scale)?;
    builder.write_u8(ut_data.options.window_color_r)?;
    builder.write_u8(ut_data.options.window_color_g)?;
    builder.write_u8(ut_data.options.window_color_b)?;
    builder.write_u8(ut_data.options.window_color_a)?;
    builder.write_u32(ut_data.options.color_depth)?;
    builder.write_u32(ut_data.options.resolution)?;
    builder.write_u32(ut_data.options.frequency)?;
    builder.write_u32(ut_data.options.vertex_sync)?;
    builder.write_u32(ut_data.options.priority)?;
    // CHANGE TYPES TO `texture page item` WHEN SUPPORTED
    builder.write_u32(ut_data.options.back_image)?;
    builder.write_u32(ut_data.options.front_image)?;
    builder.write_u32(ut_data.options.load_image)?;
    // ^
    builder.write_u32(ut_data.options.load_alpha)?;

    build_chunk(data_builder, builder)?;
    Ok(())
}


fn build_options_flags(flags: &UTOptionsFlags) -> u64 {
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
    if flags.screen_shot_key {raw |= 0x2000};
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

