use crate::deserialize::backgrounds::GMBackground;
use crate::deserialize::embedded_textures::GMEmbeddedTexture;
use crate::deserialize::fonts::{GMFont, GMFontGlyph};
use crate::deserialize::game_objects::{GMGameObject, GMGameObjectEvent, GMGameObjectEventAction};
use crate::deserialize::general_info::{GMFunctionClassifications, GMGeneralInfo, GMGeneralInfoFlags, GMOptions, GMOptionsFlags};
use crate::deserialize::rooms::{GMRoom, GMRoomBackground, GMRoomFlags, GMRoomGameObject, GMRoomLayer, GMRoomTile, GMRoomView};
use crate::deserialize::sequence::{GMKeyframe, GMKeyframeMoment, GMSequence, GMTrack};
use crate::deserialize::sounds::{GMSound, GMSoundFlags};
use crate::deserialize::strings::GMStrings;
use crate::deserialize::texture_page_items::GMTexture;

#[allow(dead_code)]
impl GMGeneralInfo {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("General Info:");
        println!("  GMS Debugger Disabled: {}", self.is_debugger_disabled);
        println!("  Bytecode Version: {}", self.bytecode_version);
        println!("  File Name: {}", self.game_file_name.resolve(&strings.strings_by_index)?);
        println!("  Config: {}", self.config.resolve(&strings.strings_by_index)?);
        println!("  Game ID: {}", self.game_id);
        println!("  Directplay GUID: {}", self.directplay_guid);
        println!("  Game Name: {}", self.game_name.resolve(&strings.strings_by_index)?);
        println!("  Version: {}.{}.{}.{}", self.major_version, self.minor_version, self.release_version, self.stable_version);
        println!("  Default Window Size: {}x{}", self.default_window_width, self.default_window_height);
        println!("  Flags: {}", self.flags.to_string());
        println!("  License CRC32: {}", self.license_crc32);
        println!("  License MD5: {}", format_license_md5(&self.license_md5));
        println!("  Timestamp: {}", self.timestamp_created);
        println!("  Display Name: {}", self.display_name.resolve(&strings.strings_by_index)?);
        println!("  Active Targets: {}", self.active_targets);
        println!("  Function Classifications: {}", &self.function_classifications.to_string());
        println!("  Steam AppID: {}", self.steam_appid);
        println!("  Debugger Port: {:?}", self.debugger_port);
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMOptions {
    pub fn print(&self) {
        println!("Options:");
        println!("  Flags: {}", &self.flags.to_string());
        println!("  Scale: {}", self.scale);
        println!("  Window Color: {:?}", self.window_color);
        println!("  Color Depth: {}", self.color_depth);
        println!("  Resolution: {}", self.resolution);
        println!("  Frequency: {}", self.frequency);
        println!("  Vertex Sync: {}", self.vertex_sync);
        println!("  Priority: {}", self.priority);
        println!("  Load Alpha: {}", self.load_alpha);
        println!();
    }
}


#[allow(dead_code)]
impl GMGeneralInfoFlags {
    pub fn to_string(&self) -> String {
        let mut flag_strings: Vec<&str> = vec![];
        if self.borderless_window {
            flag_strings.push("Borderless Window");
        }
        if self.sync_vertex1 {
            flag_strings.push("Sync Vertex 1");
        }
        if self.sync_vertex2 {
            flag_strings.push("Sync Vertex 2");
        }
        if self.sync_vertex3 {
            flag_strings.push("Sync Vertex 3");
        }
        if self.fullscreen {
            flag_strings.push("Fullscreen");
        }
        if self.interpolate {
            flag_strings.push("Interpolate");
        }
        if self.scale {
            flag_strings.push("Scale");
        }
        if self.show_cursor {
            flag_strings.push("Show Cursor");
        }
        if self.sizeable {
            flag_strings.push("Sizeable");
        }
        if self.screen_key {
            flag_strings.push("Screen Key");
        }
        if self.studio_version_b1 {
            flag_strings.push("Studio Version B1");
        }
        if self.studio_version_b2 {
            flag_strings.push("Studio Version B2");
        }
        if self.studio_version_b3 {
            flag_strings.push("Studio Version B3");
        }
        if self.steam_enabled {
            flag_strings.push("Steam Enabled");
        }
        if self.local_data_enabled {
            flag_strings.push("Local Data Enabled");
        }
        if self.javascript_mode {
            flag_strings.push("JavaScript Mode");
        }
        if self.license_exclusions {
            flag_strings.push("Licence Exclusions");
        }
        flag_strings.join(", ")
    }
}



#[allow(dead_code)]
impl GMFunctionClassifications {
    fn to_string(&self) -> String {
        let mut function_classification_strings: Vec<&str> = vec![];

        if self.none {
            function_classification_strings.push("None");
        }
        if self.internet {
            function_classification_strings.push("Internet");
        }
        if self.joystick {
            function_classification_strings.push("Joystick");
        }
        if self.gamepad {
            function_classification_strings.push("Gamepad");
        }
        if self.immersion {
            function_classification_strings.push("Immersion");
        }
        if self.screengrab {
            function_classification_strings.push("Screen Grab");
        }
        if self.math {
            function_classification_strings.push("Math");
        }
        if self.action {
            function_classification_strings.push("Action");
        }
        if self.matrix_d3d {
            function_classification_strings.push("Matrix D3D");
        }
        if self.d3dmodel {
            function_classification_strings.push("D3D Model");
        }
        if self.data_structures {
            function_classification_strings.push("Data Structures");
        }
        if self.file {
            function_classification_strings.push("File");
        }
        if self.ini {
            function_classification_strings.push("INI");
        }
        if self.filename {
            function_classification_strings.push("Filename");
        }
        if self.directory {
            function_classification_strings.push("Directory");
        }
        if self.environment {
            function_classification_strings.push("Environment");
        }
        if self.http {
            function_classification_strings.push("HTTP");
        }
        if self.encoding {
            function_classification_strings.push("Encoding");
        }
        if self.uidialog {
            function_classification_strings.push("UI Dialog");
        }
        if self.motion_planning {
            function_classification_strings.push("Motion Planning");
        }
        if self.shape_collision {
            function_classification_strings.push("Shape Collision");
        }
        if self.instance {
            function_classification_strings.push("Instance");
        }
        if self.room {
            function_classification_strings.push("Room");
        }
        if self.game {
            function_classification_strings.push("Game");
        }
        if self.display {
            function_classification_strings.push("Display");
        }
        if self.device {
            function_classification_strings.push("Device");
        }
        if self.window {
            function_classification_strings.push("Window");
        }
        if self.draw_color {
            function_classification_strings.push("Draw Color");
        }
        if self.texture {
            function_classification_strings.push("Texture");
        }
        if self.layer {
            function_classification_strings.push("Layer");
        }
        if self.string {
            function_classification_strings.push("String");
        }
        if self.tiles {
            function_classification_strings.push("Tiles");
        }
        if self.surface {
            function_classification_strings.push("Surface");
        }
        if self.skeleton {
            function_classification_strings.push("Skeleton");
        }
        if self.io {
            function_classification_strings.push("IO");
        }
        if self.variables {
            function_classification_strings.push("Variables");
        }
        if self.array {
            function_classification_strings.push("Array");
        }
        if self.external_call {
            function_classification_strings.push("External Call");
        }
        if self.notification {
            function_classification_strings.push("Notification");
        }
        if self.date {
            function_classification_strings.push("Date");
        }
        if self.particle {
            function_classification_strings.push("Particle");
        }
        if self.sprite {
            function_classification_strings.push("Sprite");
        }
        if self.clickable {
            function_classification_strings.push("Clickable");
        }
        if self.legacy_sound {
            function_classification_strings.push("Legacy Sound");
        }
        if self.audio {
            function_classification_strings.push("Audio");
        }
        if self.event {
            function_classification_strings.push("Event");
        }
        if self.free_type {
            function_classification_strings.push("FreeType");
        }
        if self.analytics {
            function_classification_strings.push("Analytics");
        }
        if self.achievement {
            function_classification_strings.push("Achievement");
        }
        if self.cloud_saving {
            function_classification_strings.push("Cloud Saving");
        }
        if self.ads {
            function_classification_strings.push("Ads");
        }
        if self.os {
            function_classification_strings.push("OS");
        }
        if self.iap {
            function_classification_strings.push("IAP");
        }
        if self.facebook {
            function_classification_strings.push("Facebook");
        }
        if self.physics {
            function_classification_strings.push("Physics");
        }
        if self.flash_aa {
            function_classification_strings.push("Flash AA");
        }
        if self.console {
            function_classification_strings.push("Console");
        }
        if self.buffer {
            function_classification_strings.push("Buffer");
        }
        if self.steam {
            function_classification_strings.push("Steam");
        }
        if self.shaders {
            function_classification_strings.push("Shaders");
        }
        if self.vertex_buffers {
            function_classification_strings.push("Vertex Buffers");
        }

        function_classification_strings.join(", ")
    }
}


#[allow(dead_code)]
impl GMOptionsFlags {
    fn to_string(&self) -> String {
        let mut flag_strings: Vec<&str> = vec![];

        if self.fullscreen {
            flag_strings.push("Fullscreen");
        }
        if self.interpolate_pixels {
            flag_strings.push("Interpolate Pixels");
        }
        if self.use_new_audio {
            flag_strings.push("Use New Audio");
        }
        if self.no_border {
            flag_strings.push("No Border");
        }
        if self.show_cursor {
            flag_strings.push("Show Cursor");
        }
        if self.sizeable {
            flag_strings.push("Sizeable");
        }
        if self.stay_on_top {
            flag_strings.push("Stay on Top");
        }
        if self.change_resolution {
            flag_strings.push("Change Resolution");
        }
        if self.no_buttons {
            flag_strings.push("No Buttons");
        }
        if self.screen_key {
            flag_strings.push("Screen Key");
        }
        if self.help_key {
            flag_strings.push("Help Key");
        }
        if self.quit_key {
            flag_strings.push("Quit Key");
        }
        if self.save_key {
            flag_strings.push("Save Key");
        }
        if self.screenshot_key {
            flag_strings.push("Screenshot Key");
        }
        if self.close_sec {
            flag_strings.push("Close Sec");
        }
        if self.freeze {
            flag_strings.push("Freeze");
        }
        if self.show_progress {
            flag_strings.push("Show Progress");
        }
        if self.load_transparent {
            flag_strings.push("Load Transparent");
        }
        if self.scale_progress {
            flag_strings.push("Scale Progress");
        }
        if self.display_errors {
            flag_strings.push("Display Errors");
        }
        if self.write_errors {
            flag_strings.push("Write Errors");
        }
        if self.abort_errors {
            flag_strings.push("Abort Errors");
        }
        if self.variable_errors {
            flag_strings.push("Variable Errors");
        }
        if self.creation_event_order {
            flag_strings.push("Creation Event Order");
        }
        if self.use_front_touch {
            flag_strings.push("Use Front Touch");
        }
        if self.use_rear_touch {
            flag_strings.push("Use Rear Touch");
        }
        if self.use_fast_collision {
            flag_strings.push("Use Fast Collision");
        }
        if self.fast_collision_compatibility {
            flag_strings.push("Fast Collision Compatibility");
        }
        if self.disable_sandbox {
            flag_strings.push("Disable Sandbox");
        }
        if self.enable_copy_on_write {
            flag_strings.push("Enable Copy on Write");
        }

        flag_strings.join(", ")
    }
}


#[allow(dead_code)]
impl GMFont {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMFont:");
        println!("  Name: {}", self.name.resolve(&strings.strings_by_index)?);
        println!("  Display Name: {}", self.display_name.resolve(&strings.strings_by_index)?);
        println!("  EM Size: {}", self.em_size);
        println!("  Bold: {}", self.bold);
        println!("  Italic: {}", self.italic);
        println!("  Range Start: {}", self.range_start);
        println!("  Charset: {}", self.charset);
        println!("  Anti-Alias: {}", self.anti_alias);
        println!("  Range End: {}", self.range_end);
        println!("  Texture: #{}", self.texture.index);
        println!("  Scale X: {}", self.scale_x);
        println!("  Scale Y: {}", self.scale_y);
        println!("  Ascender Offset: {:?}", self.ascender_offset);
        println!("  Ascender: {:?}", self.ascender);
        println!("  SDF Spread: {:?}", self.sdf_spread);
        println!("  Line Height: {:?}", self.line_height);
        println!();
        Ok(())
    }
}


#[allow(dead_code)]
impl GMFontGlyph {
    pub fn print(&self) {
        println!("GMGlyph:");
        println!("  Character: {}", if let Some(chr) = self.character { format!("'{chr}'") } else { "None".to_string() });
        println!("  Position: ({}; {})", self.x, self.y);
        println!("  Size: {} x {}", self.width, self.height);
        println!("  Shift Modifier: {}", self.shift_modifier);
        println!("  Offset: {}", self.offset);
        println!();
    }
}


#[allow(dead_code)]
impl GMRoom {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMRoom:");
        println!("  Name: \"{}\"", self.name.resolve(&strings.strings_by_index)?);
        println!("  Caption: \"{}\"", self.caption.resolve(&strings.strings_by_index)?);
        println!("  Dimensions: {}x{}", self.width, self.height);
        println!("  Speed: {}", self.speed);
        println!("  Persistent: {}", self.persistent);
        println!("  Background Color: #{:06X}", self.background_color & 0xFFFFFF);
        println!("  Draw Background Color: {}", self.draw_background_color);
        println!("  Creation Code: {:?}", self.creation_code);
        println!("  Flags: {}", self.flags.to_string());
        println!("  Backgrounds Length: {}", self.backgrounds.len());
        // for background in self.backgrounds.clone() {
        //     background.print();
        // }
        println!("  Views Length: {}", self.views.len());
        // for view in self.views.clone() {
        //     view.print();
        // }
        println!("  Game Objects Length: {}", self.game_objects.len());
        println!("  Tiles Length: {}", self.tiles.len());
        // for tile in self.tiles.clone() {
        //     tile.print();
        // }
        println!("  World: {}", self.world);
        println!("  Bounds: ({}, {}) - ({}, {})", self.left, self.top, self.right, self.bottom);
        println!("  Gravity: ({}, {})", self.gravity_x, self.gravity_y);
        println!("  Meters Per Pixel: {}", self.meters_per_pixel);
        match &self.layers {
            Some(layers) => {
                println!("  Layers Length: {}", layers.len());
                for layer in layers {
                    layer.print(&strings)?;
                }
            },
            None => println!("  Layers: None"),
        }

        match &self.sequences {
            Some(sequences) => {
                println!("  Sequences Length: {}", sequences.len());
                // for sequences in sequences {
                //     sequences.print();
                // }
            },
            None => println!("  Sequences: None"),
        }
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMRoomLayer {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMRoomLayer:");
        println!("  Layer Name: {}", self.layer_name.resolve(&strings.strings_by_index)?);
        println!("  Layer ID: {}", self.layer_id);
        println!("  Layer Type: {:?}", self.layer_type);
        println!("  Layer Depth: {}", self.layer_depth);
        println!("  Offset: ({}, {})", self.x_offset, self.y_offset);
        println!("  Speed: ({}, {})", self.horizontal_speed, self.vertical_speed);
        println!("  Visible: {}", self.is_visible);
        Ok(())
    }
}

#[allow(dead_code)]
impl GMRoomTile {
    pub fn print(&self) {
        println!("GMRoomTile:");
        println!("  Position: ({}, {})", self.x, self.y);
        println!("  Texture: {:?}", self.texture);
        println!("  Source: ({}, {})", self.source_x, self.source_y);
        println!("  Size: {}x{}", self.width, self.height);
        println!("  Depth: {}", self.tile_depth);
        println!("  Instance ID: {}", self.instance_id);
        println!("  Scale: ({}, {})", self.scale_x, self.scale_y);
        println!("  Color: {:X}", self.color);
        println!();
    }
}

#[allow(dead_code)]
impl GMRoomFlags {
    pub fn to_string(&self) -> String {
        let mut flags: Vec<&'static str> = vec![];
        if self.enable_views { flags.push("Enable Views"); }
        if self.show_color { flags.push("Show Color"); }
        if self.dont_clear_display_buffer { flags.push("Don't Clear Display Buffer"); }
        if self.is_gms2 { flags.push("Is GMS2"); }
        if self.is_gms2_3 { flags.push("Is GMS2.3"); }
        flags.join(", ")
    }
}

#[allow(dead_code)]
impl GMRoomBackground {
    pub fn print(&self) {
        println!("GMRoomBackground:");
        println!("  Enabled: {}", self.enabled);
        println!("  Foreground: {}", self.foreground);
        // println!("  Background Definition: {}", self.background_definition);
        println!("  Position: ({}, {})", self.x, self.y);
        println!("  Tile: ({}, {})", self.tile_x, self.tile_y);
        println!("  Speed: ({}, {})", self.speed_x, self.speed_y);
        println!("  Stretch: {}", self.stretch);
        println!();
    }
}

#[allow(dead_code)]
impl GMRoomView {
    pub fn print(&self) {
        println!("GMRoomView:");
        println!("  Enabled: {}", self.enabled);
        println!("  View Position: ({}, {})", self.view_x, self.view_y);
        println!("  View Size: {}x{}", self.view_width, self.view_height);
        println!("  Port Position: ({}, {})", self.port_x, self.port_y);
        println!("  Port Size: {}x{}", self.port_width, self.port_height);
        println!("  Border: ({}, {})", self.border_x, self.border_y);
        println!("  Speed: ({}, {})", self.speed_x, self.speed_y);
        println!("  Object ID: {:?}", self.object);
        println!();
    }
}

#[allow(dead_code)]
impl GMSequence {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMSequence:");
        println!("  Name: {}", self.name.resolve(&strings.strings_by_index)?);
        println!("  Playback: {:?}", self.playback);
        println!("  Playback Speed: {} ({:?})", self.playback_speed, self.playback_speed_type);
        println!("  Length: {}", self.length);
        println!("  Origin: ({}, {})", self.origin_x, self.origin_y);
        println!("  Volume: {}", self.volume);
        println!("  Broadcast Messages: {:?}", self.broadcast_messages);
        println!("  Tracks: [{} items]", self.tracks.len());
        println!("  Function IDs: [{} items]", self.function_ids.len());
        println!("  Moments: [{} items]", self.moments.len());
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMKeyframe {
    pub fn print(&self) {
        println!("GMKeyframe:");
        println!("  Key: {}", self.key);
        println!("  Length: {}", self.length);
        println!("  Stretch: {}", self.stretch);
        println!("  Disabled: {}", self.disabled);
        println!("  Channels: {:?}", self.channels);
        println!();
    }
}

#[allow(dead_code)]
impl GMTrack {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMTrack:");
        println!("  Model Name: {}", self.model_name.resolve(&strings.strings_by_index)?);
        println!("  Name: {}", self.name.resolve(&strings.strings_by_index)?);
        println!("  Built-in Name: {:?}", self.builtin_name);
        println!("  Traits: {:?}", self.traits);
        println!("  Is Creation Track: {}", self.is_creation_track);
        println!("  Tags: {:?}", self.tags);
        println!("  Sub-Tracks: [{} items]", self.sub_tracks.len());
        println!("  Keyframes: [{} items]", self.keyframes.len());
        println!("  GM Anim Curve String: {}", self.anim_curve_string.clone().map_or("<string is unset>", |i| i.display(strings)));
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMKeyframeMoment {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMKeyframeMoment:");
        println!("  Internal Count: {}", self.internal_count);
        if let Some(event) = &self.event {
            println!("  Event: {}", event.resolve(&strings.strings_by_index)?);
        } else {
            println!("  Event: None");
        }
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMEmbeddedTexture {
    pub fn print(&self) {
        println!("GMEmbeddedTexture:");
        println!("  Scaled: {}", self.scaled);
        println!("  Generated Mips: {:?}", self.generated_mips);
        println!("  Texture Block Size: {:?}", self.texture_block_size);
        println!("  Texture Width: {:?}", self.texture_width);
        println!("  Texture Height: {:?}", self.texture_height);
        println!("  Index In Group: {:?}", self.index_in_group);
        println!("  Texture Data: <Image Data>");
        println!();
    }
}

#[allow(dead_code)]
impl GMTexture {
    pub fn print(&self) {
        println!("GMTexture:");
        println!("  Target X: {}", self.target_x);
        println!("  Target Y: {}", self.target_y);
        println!("  Target Width: {}", self.target_width);
        println!("  Target Height: {}", self.target_height);
        println!("  Bounding Width: {}", self.bounding_width);
        println!("  Bounding Height: {}", self.bounding_height);
        println!("  Image Data: <DynamicImage Data>");
        println!();
    }
}

#[allow(dead_code)]
impl GMBackground {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMBackground:");
        println!("  Name: \"{}\"", self.name.resolve(&strings.strings_by_index)?);
        println!("  Transparent: {}", self.transparent);
        println!("  Smooth: {}", self.smooth);
        println!("  Preload: {}", self.preload);
        println!("  Texture Index: {}", if let Some(ref tex) = self.texture { format!("{}", tex.index) } else { "None".to_string() });
        if let Some(ref gms2_data) = self.gms2_data {
            println!("  GMS2 Unknown Always 2: {:?}", gms2_data.unknown_always2);
            println!("  GMS2 Tile Width: {:?}", gms2_data.tile_width);
            println!("  GMS2 Tile Height: {:?}", gms2_data.tile_height);
            println!("  GMS2 Output Border X: {:?}", gms2_data.output_border_x);
            println!("  GMS2 Output Border Y: {:?}", gms2_data.output_border_y);
            println!("  GMS2 Tile Columns: {:?}", gms2_data.tile_columns);
            println!("  GMS2 Items Per Tile Count: {:?}", gms2_data.items_per_tile_count);
            println!("  GMS2 Unknown Always Zero: {:?}", gms2_data.unknown_always_zero);
            println!("  GMS2 Frame Length: {:?}", gms2_data.frame_length);
            println!("  GMS2 Tile IDs: [{} items]", gms2_data.tile_ids.len());
        }
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMSound {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMSound:");
        println!("  Name: \"{}\"", self.name.resolve(&strings.strings_by_index)?);
        println!("  Flags: {}", self.flags.to_string());
        println!("  Audio Type: \"{}\"", self.audio_type.resolve(&strings.strings_by_index)?);
        println!("  File: \"{}\"", self.file.resolve(&strings.strings_by_index)?);
        println!("  Effects: {}", self.effects);
        println!("  Volume: {}", self.volume);
        println!("  Pitch: {}", self.pitch);
        println!("  Audio File: {:?}", self.audio_file);
        println!("  Length: {} seconds", if self.audio_length.is_some() {self.audio_length.unwrap().to_string()} else {"<Unspecified>".to_string()});
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMSoundFlags {
    pub fn to_string(&self) -> String {
        let mut flags = Vec::new();
        if self.is_embedded { flags.push("Embedded"); }
        if self.is_compressed { flags.push("Compressed"); }
        if self.is_decompressed_on_load { flags.push("Decompressed On Load"); }
        if self.regular { flags.push("Regular"); }
        flags.join(", ")
    }
}

#[allow(dead_code)]
impl GMGameObject {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMGameObject:");
        println!("  Name: \"{}\"", self.name.resolve(&strings.strings_by_index)?);
        println!("  Sprite Index: {:?}", self.sprite);
        println!("  Visible: {}", self.visible);
        println!("  Managed: {:?}", self.managed);
        println!("  Solid: {}", self.solid);
        println!("  Depth: {}", self.depth);
        println!("  Persistent: {}", self.persistent);
        println!("  Parent ID: {}", self.parent_id);
        println!("  Texture Mask ID: {:?}", self.texture_mask);
        println!("  Uses Physics: {}", self.uses_physics);
        println!("  Is Sensor: {}", self.is_sensor);
        println!("  Collision Shape: {:?}", self.collision_shape);
        println!("  Density: {}", self.density);
        println!("  Restitution: {}", self.restitution);
        println!("  Group: {}", self.group);
        println!("  Linear Damping: {}", self.linear_damping);
        println!("  Angular Damping: {}", self.angular_damping);
        println!("  Physics Shape Vertex Count: {}", self.physics_shape_vertices.len());
        println!("  Friction: {}", self.friction);
        println!("  Awake: {}", self.awake);
        println!("  Kinematic: {}", self.kinematic);
        println!("  Events Count: {}", self.events.len());
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMGameObjectEvent {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMGameObjectEvent:");
        println!("  Subtype: {}", self.subtype);
        println!("  Actions: [{} items]", self.actions.len());
        for action in &self.actions {
            action.print(strings)?;
        }
        println!();
        Ok(())
    }
}

#[allow(dead_code)]
impl GMGameObjectEventAction {
    pub fn print(&self, strings: &GMStrings) -> Result<(), String> {
        println!("GMGameObjectEventAction:");
        println!("  Lib ID: {}", self.lib_id);
        println!("  ID: {}", self.id);
        println!("  Kind: {}", self.kind);
        println!("  Use Relative: {}", self.use_relative);
        println!("  Is Question: {}", self.is_question);
        println!("  Use Apply To: {}", self.use_apply_to);
        println!("  Exe Type: {}", self.exe_type);
        // println!("  Action Name: {}", self.action_name.as_ref().map(|i| i.resolve(&strings.strings_by_index))..unwrap_or_else(|| "None".to_string()));
        println!("  Code: {:?}", self.code);
        println!("  Argument Count: {}", self.argument_count);
        println!("  Who: {}", self.who);
        println!("  Relative: {}", self.relative);
        println!("  Is Not: {}", self.is_not);
        println!("  Unknown Always Zero: {}", self.unknown_always_zero);
        println!();
        Ok(())
    }
}


#[allow(dead_code)]
impl GMRoomGameObject {
    pub fn print(&self) -> Result<(), String> {
        println!("GMRoomGameObject:");
        println!("  X: {}", self.x);
        println!("  Y: {}", self.y);
        println!("  Object Definition: {:?}", self.object_definition);
        println!("  Instance ID: {}", self.instance_id);
        println!("  Creation Code: {:?}", self.creation_code);
        println!("  Scale X: {}", self.scale_x);
        println!("  Scale Y: {}", self.scale_y);
        if let Some(image_speed) = self.image_speed {
            println!("  Image Speed: {}", image_speed);
        }
        if let Some(image_index) = self.image_index {
            println!("  Image Index: {}", image_index);
        }
        println!("  Color: #{:#010x}", self.color); // Hex format for color
        println!("  Rotation: {}", self.rotation);
        println!("  Pre Create Code: {:?}", self.pre_create_code);
        println!();
        Ok(())
    }
}




fn format_license_md5(license: &[u8; 16]) -> String {
    let mut hex_bytes: Vec<String> = vec![];

    for i in license {
        hex_bytes.push(format!("{:02X}", i))
    }

    hex_bytes.join(" ")
}

pub fn hexdump(raw_data: &[u8], start: usize, end: Option<usize>) -> Result<String, String> {
    let len: usize = raw_data.len();
    let end: usize = end.unwrap_or_else(|| len);
    if end > len {
        return Err(format!("Specified end of hexdump is out ouf bounds: {} >= {} (start: {})", end, len, start));
    }
    if start > end {
        return Err(format!("Specified start of hexdump is greater or equal to specified end: {} >= {} (len: {})", start, end, len));
    }
    let len: usize = end - start;
    if len < 1 {
        return Ok("".to_string());
    }

    let mut string: String = String::with_capacity(len * 3);
    for i in start..end {
        let byte: u8 = raw_data[i];
        string.push_str(&format!("{byte:02X} "));
    }
    string.pop();  // remove trailing space
    Ok(string)
}

pub fn format_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}
