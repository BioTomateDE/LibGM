use crate::structs::*;

pub fn print_general_info(general_info: &UTGeneralInfo) {
    println!("\nGeneral Info:");
    println!(
        "  GMS Debugger Disabled: {}",
        general_info.is_debugger_disabled
    );
    println!("  Bytecode Version: {}", general_info.bytecode_version);
    println!("  File Name: {}", general_info.game_file_name);
    println!("  Config: {}", general_info.config);
    println!("  Last object ID: {}", general_info.last_object_id);
    println!("  Last tile ID: {}", general_info.last_tile_id);
    println!("  Game ID: {}", general_info.game_id);
    println!("  Directplay GUID: {}", general_info.directplay_guid);
    println!("  Game Name: {}", general_info.game_name);
    println!(
        "  Version: {}.{}.{}.{}",
        general_info.major_version,
        general_info.minor_version,
        general_info.release_version,
        general_info.stable_version
    );
    println!(
        "  Default Window Size: {}x{}",
        general_info.default_window_width, general_info.default_window_height
    );
    println!("  Flags: {}", format_flags(&general_info.flags));
    println!("  License: {}", format_license_md5(&general_info.license));
    println!("  Timestamp: {}", general_info.timestamp_created);
    println!("  Display Name: {}", general_info.display_name);
    println!("  Active Targets: {}", general_info.active_targets);
    println!("  Function Classifications: {}", format_function_classifications(&general_info.function_classifications));
    println!("  Steam AppID: {}", general_info.steam_appid);
    println!("  Debugger Port: {}", general_info.debugger_port);
    // println!("  Room Order: {:?}", general_info.room_order);
    println!();
}

fn format_flags(flags: &UTGeneralInfoFlags) -> String {
    let mut flag_strings: Vec<&str> = vec![];
    if flags.borderless_window {
        flag_strings.push("Borderless Window");
    }
    if flags.sync_vertex1 {
        flag_strings.push("Sync Vertex 1");
    }
    if flags.sync_vertex2 {
        flag_strings.push("Sync Vertex 2");
    }
    if flags.sync_vertex3 {
        flag_strings.push("Sync Vertex 3");
    }
    if flags.fullscreen {
        flag_strings.push("Fullscreen");
    }
    if flags.interpolate {
        flag_strings.push("Interpolate");
    }
    if flags.scale {
        flag_strings.push("Scale");
    }
    if flags.show_cursor {
        flag_strings.push("Show Cursor");
    }
    if flags.sizeable {
        flag_strings.push("Sizeable");
    }
    if flags.screen_key {
        flag_strings.push("Screen Key");
    }
    if flags.studio_version_b1 {
        flag_strings.push("Studio Version B1");
    }
    if flags.studio_version_b2 {
        flag_strings.push("Studio Version B2");
    }
    if flags.studio_version_b3 {
        flag_strings.push("Studio Version B3");
    }
    if flags.steam_enabled {
        flag_strings.push("Steam Enabled");
    }
    if flags.local_data_enabled {
        flag_strings.push("Local Data Enabled");
    }
    if flags.javascript_mode {
        flag_strings.push("JavaScript Mode");
    }
    flag_strings.join(", ")
}

fn format_function_classifications(
    function_classifications: &UTFunctionClassifications,
) -> String {
    let mut function_classification_strings: Vec<&str> = vec![];

    if function_classifications.none {
        function_classification_strings.push("None");
    }
    if function_classifications.internet {
        function_classification_strings.push("Internet");
    }
    if function_classifications.joystick {
        function_classification_strings.push("Joystick");
    }
    if function_classifications.gamepad {
        function_classification_strings.push("Gamepad");
    }
    if function_classifications.immersion {
        function_classification_strings.push("Immersion");
    }
    if function_classifications.screengrab {
        function_classification_strings.push("Screen Grab");
    }
    if function_classifications.math {
        function_classification_strings.push("Math");
    }
    if function_classifications.action {
        function_classification_strings.push("Action");
    }
    if function_classifications.matrix_d3d {
        function_classification_strings.push("Matrix D3D");
    }
    if function_classifications.d3dmodel {
        function_classification_strings.push("D3D Model");
    }
    if function_classifications.data_structures {
        function_classification_strings.push("Data Structures");
    }
    if function_classifications.file {
        function_classification_strings.push("File");
    }
    if function_classifications.ini {
        function_classification_strings.push("INI");
    }
    if function_classifications.filename {
        function_classification_strings.push("Filename");
    }
    if function_classifications.directory {
        function_classification_strings.push("Directory");
    }
    if function_classifications.environment {
        function_classification_strings.push("Environment");
    }
    if function_classifications.http {
        function_classification_strings.push("HTTP");
    }
    if function_classifications.encoding {
        function_classification_strings.push("Encoding");
    }
    if function_classifications.uidialog {
        function_classification_strings.push("UI Dialog");
    }
    if function_classifications.motion_planning {
        function_classification_strings.push("Motion Planning");
    }
    if function_classifications.shape_collision {
        function_classification_strings.push("Shape Collision");
    }
    if function_classifications.instance {
        function_classification_strings.push("Instance");
    }
    if function_classifications.room {
        function_classification_strings.push("Room");
    }
    if function_classifications.game {
        function_classification_strings.push("Game");
    }
    if function_classifications.display {
        function_classification_strings.push("Display");
    }
    if function_classifications.device {
        function_classification_strings.push("Device");
    }
    if function_classifications.window {
        function_classification_strings.push("Window");
    }
    if function_classifications.draw_color {
        function_classification_strings.push("Draw Color");
    }
    if function_classifications.texture {
        function_classification_strings.push("Texture");
    }
    if function_classifications.layer {
        function_classification_strings.push("Layer");
    }
    if function_classifications.string {
        function_classification_strings.push("String");
    }
    if function_classifications.tiles {
        function_classification_strings.push("Tiles");
    }
    if function_classifications.surface {
        function_classification_strings.push("Surface");
    }
    if function_classifications.skeleton {
        function_classification_strings.push("Skeleton");
    }
    if function_classifications.io {
        function_classification_strings.push("IO");
    }
    if function_classifications.variables {
        function_classification_strings.push("Variables");
    }
    if function_classifications.array {
        function_classification_strings.push("Array");
    }
    if function_classifications.external_call {
        function_classification_strings.push("External Call");
    }
    if function_classifications.notification {
        function_classification_strings.push("Notification");
    }
    if function_classifications.date {
        function_classification_strings.push("Date");
    }
    if function_classifications.particle {
        function_classification_strings.push("Particle");
    }
    if function_classifications.sprite {
        function_classification_strings.push("Sprite");
    }
    if function_classifications.clickable {
        function_classification_strings.push("Clickable");
    }
    if function_classifications.legacy_sound {
        function_classification_strings.push("Legacy Sound");
    }
    if function_classifications.audio {
        function_classification_strings.push("Audio");
    }
    if function_classifications.event {
        function_classification_strings.push("Event");
    }
    if function_classifications.free_type {
        function_classification_strings.push("FreeType");
    }
    if function_classifications.analytics {
        function_classification_strings.push("Analytics");
    }
    if function_classifications.achievement {
        function_classification_strings.push("Achievement");
    }
    if function_classifications.cloud_saving {
        function_classification_strings.push("Cloud Saving");
    }
    if function_classifications.ads {
        function_classification_strings.push("Ads");
    }
    if function_classifications.os {
        function_classification_strings.push("OS");
    }
    if function_classifications.iap {
        function_classification_strings.push("IAP");
    }
    if function_classifications.facebook {
        function_classification_strings.push("Facebook");
    }
    if function_classifications.physics {
        function_classification_strings.push("Physics");
    }
    if function_classifications.flash_aa {
        function_classification_strings.push("Flash AA");
    }
    if function_classifications.console {
        function_classification_strings.push("Console");
    }
    if function_classifications.buffer {
        function_classification_strings.push("Buffer");
    }
    if function_classifications.steam {
        function_classification_strings.push("Steam");
    }
    if function_classifications.shaders {
        function_classification_strings.push("Shaders");
    }
    if function_classifications.vertex_buffers {
        function_classification_strings.push("Vertex Buffers");
    }

    function_classification_strings.join(", ")
}


fn format_license_md5(license: &[u8; 16]) -> String {
    let mut hex_bytes: Vec<String> = vec![];

    for i in license {
        hex_bytes.push(format!("{:02X}", i))
    }

    hex_bytes.join(" ")
}

