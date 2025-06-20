use crate::deserialize::all::GMData;
use crate::deserialize::general_info::{GMFunctionClassifications, GMGeneralInfo, GMGeneralInfoFlags, GMVersion};
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_gen8(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("GEN8")?;
    let info: &GMGeneralInfo = &gm_data.general_info;

    builder.write_u8(if info.is_debugger_disabled {1} else {0});
    builder.write_u8(info.bytecode_version);
    builder.write_u16(info.unknown_value);
    builder.write_placeholder(GMPointer::String(info.game_file_name.index))?;
    builder.write_placeholder(GMPointer::String(info.config.index))?;
    builder.write_u32(info.last_object_id);     // these have to be incremented when mods add objects/tiles!
    builder.write_u32(info.last_tile_id);       // ^
    builder.write_u32(info.game_id);
    builder.raw_data.extend(info.directplay_guid.as_bytes());
    builder.write_placeholder(GMPointer::String(info.game_name.index))?;
    build_version(builder, &info.version);
    builder.write_u32(info.default_window_width);
    builder.write_u32(info.default_window_height);
    builder.write_u32(build_general_info_flags(&info.flags));
    builder.write_u32(info.license_crc32);
    builder.raw_data.extend(info.license_md5);
    builder.write_i64(info.timestamp_created.timestamp());
    builder.write_placeholder(GMPointer::String(info.display_name.index))?;
    builder.write_u64(info.active_targets);
    builder.write_u64(build_function_classifications(&info.function_classifications));
    builder.write_i32(info.steam_appid);
    if info.bytecode_version >= 14 {
        builder.write_u32(info.debugger_port.ok_or("General info: debugger port not set")?);
    }

    builder.write_usize(info.room_order.len());
    for room_ref in &info.room_order {
        builder.write_usize(room_ref.index);
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


fn build_version(chunk: &mut DataBuilder, version: &GMVersion) {
    chunk.write_u32(version.major);
    chunk.write_u32(version.minor);
    chunk.write_u32(version.release);
    chunk.write_u32(version.stable);
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

