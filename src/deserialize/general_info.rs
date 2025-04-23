use crate::deserialize::chunk_reading::GMChunk;
use chrono::{DateTime, Utc};
use crate::deserialize::strings::{GMStringRef, GMStrings};

#[derive(Debug, Clone)]
pub struct GMOptions {
    pub _unused1: u32,
    pub _unused2: u32,
    pub flags: GMOptionsFlags,
    pub scale: i32,
    pub window_color_r: u8,
    pub window_color_g: u8,
    pub window_color_b: u8,
    pub window_color_a: u8,
    pub color_depth: u32,
    pub resolution: u32,
    pub frequency: u32,
    pub vertex_sync: u32,
    pub priority: u32,
    pub back_image: u32,         // CHANGE TYPE TO `texture page item` WHEN SUPPORTED
    pub front_image: u32,        // CHANGE TYPE TO `texture page item` WHEN SUPPORTED
    pub load_image: u32,         // CHANGE TYPE TO `texture page item` WHEN SUPPORTED
    pub load_alpha: u32,
}

#[derive(Debug, Clone)]
pub struct GMGeneralInfo {
    pub is_debugger_disabled: bool,
    pub bytecode_version: u8,
    pub unknown_value: u16,
    pub game_file_name: GMStringRef,
    pub config: GMStringRef,
    pub last_object_id: u32,
    pub last_tile_id: u32,
    pub game_id: u32,
    pub directplay_guid: uuid::Uuid,
    pub game_name: GMStringRef,
    pub major_version: u32,
    pub minor_version: u32,
    pub release_version: u32,
    pub stable_version: u32,
    pub default_window_width: u32,
    pub default_window_height: u32,
    pub flags: GMGeneralInfoFlags,
    pub license: [u8; 16],
    pub timestamp_created: DateTime<Utc>,
    pub display_name: GMStringRef,
    pub active_targets: u64,
    pub function_classifications: GMFunctionClassifications,
    pub steam_appid: u32,
    pub debugger_port: u16,
    pub room_order: Vec<u32>,
}

impl GMGeneralInfo {
    pub fn is_version_at_least(&self, major: u32, minor: u32, release: u32, build: u32) -> bool {

        if self.major_version != major {
            return self.major_version > major;
        }
        if self.minor_version != minor {
            return self.minor_version > minor;
        }
        if self.release_version != release {
            return self.release_version > release;
        }
        if self.stable_version != build {
            return self.stable_version > build;
        }

        true   // The version is exactly what was supplied.
    }
}

#[derive(Debug, Clone)]
pub struct GMGeneralInfoFlags {
    // taken from https://github.com/UnderminersTeam/UndertaleModTool/blob/master/UndertaleModLib/Models/UndertaleGeneralInfo.cs
    pub fullscreen: bool,
    pub sync_vertex1: bool,
    pub sync_vertex2: bool,
    pub sync_vertex3: bool,
    pub interpolate: bool,
    pub scale: bool,
    pub show_cursor: bool,
    pub sizeable: bool,
    pub screen_key: bool,
    pub studio_version_b1: bool,
    pub studio_version_b2: bool,
    pub studio_version_b3: bool,
    // studio_version_mask
    pub steam_enabled: bool,
    pub local_data_enabled: bool,
    pub borderless_window: bool,
    pub javascript_mode: bool,
    // license_exclusions: bool,
}

#[derive(Debug, Clone)]
pub struct GMFunctionClassifications {
    pub none: bool,
    pub internet: bool,
    pub joystick: bool,
    pub gamepad: bool,
    pub immersion: bool,
    pub screengrab: bool,
    pub math: bool,
    pub action: bool,
    pub matrix_d3d: bool,
    pub d3dmodel: bool,
    pub data_structures: bool,
    pub file: bool,
    pub ini: bool,
    pub filename: bool,
    pub directory: bool,
    pub environment: bool,
    pub _unused1: bool,
    pub http: bool,
    pub encoding: bool,
    pub uidialog: bool,
    pub motion_planning: bool,
    pub shape_collision: bool,
    pub instance: bool,
    pub room: bool,
    pub game: bool,
    pub display: bool,
    pub device: bool,
    pub window: bool,
    pub draw_color: bool,
    pub texture: bool,
    pub layer: bool,
    pub string: bool,
    pub tiles: bool,
    pub surface: bool,
    pub skeleton: bool,
    pub io: bool,
    pub variables: bool,
    pub array: bool,
    pub external_call: bool,
    pub notification: bool,
    pub date: bool,
    pub particle: bool,
    pub sprite: bool,
    pub clickable: bool,
    pub legacy_sound: bool,
    pub audio: bool,
    pub event: bool,
    pub _unused2: bool,
    pub free_type: bool,
    pub analytics: bool,
    pub unused3: bool,
    pub unused4: bool,
    pub achievement: bool,
    pub cloud_saving: bool,
    pub ads: bool,
    pub os: bool,
    pub iap: bool,
    pub facebook: bool,
    pub physics: bool,
    pub flash_aa: bool,
    pub console: bool,
    pub buffer: bool,
    pub steam: bool,
    pub _unused3: bool,
    pub shaders: bool,
    pub vertex_buffers: bool,
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
    pub screen_shot_key: bool,
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

#[allow(non_snake_case)]
pub fn parse_chunk_GEN8(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMGeneralInfo, String> {
    chunk.file_index = 0;
    let is_debugger_disabled: bool = chunk.read_u8()? != 0;
    let bytecode_version: u8 = chunk.read_u8()?;
    let unknown_value: u16 = chunk.read_u16()?;
    let game_file_name: GMStringRef = chunk.read_gm_string(strings)?;
    let config: GMStringRef = chunk.read_gm_string(strings)?;
    let last_object_id: u32 = chunk.read_u32()?;
    let last_tile_id: u32 = chunk.read_u32()?;
    let game_id: u32 = chunk.read_u32()?;

    let directplay_guid: [u8; 16] = match chunk.data[chunk.file_index..chunk.file_index + 16].try_into() {
        Ok(data) => data,
        Err(_) => return Err(format!(
            "Trying to read GUID out of bounds in chunk 'GEN8' at position {}: {} > {}.",
            chunk.file_index,
            chunk.file_index + 16,
            chunk.data.len(),
        )),
    };
    chunk.file_index += 16;
    let directplay_guid: uuid::Uuid = uuid::Builder::from_bytes_le(directplay_guid).into_uuid();
    // ^ perhaps not `_le` but idk bc it's usually just null

    let game_name: GMStringRef = chunk.read_gm_string(strings)?;
    let major_version: u32 = chunk.read_u32()?;
    let minor_version: u32 = chunk.read_u32()?;
    let release_version: u32 = chunk.read_u32()?;
    let stable_version: u32 = chunk.read_u32()?;
    let default_window_width: u32 = chunk.read_u32()?;
    let default_window_height: u32 = chunk.read_u32()?;
    let flags: GMGeneralInfoFlags = parse_flags(chunk)?;

    let license: [u8; 16] = match chunk.data[chunk.file_index..chunk.file_index+16].try_into() {
        Ok(data) => data,
        Err(_) => return Err(format!(
            "Trying to read license out of bounds in chunk 'GEN8' at position {}: {} > {}.",
            chunk.file_index,
            chunk.file_index + 16,
            chunk.data.len(),
        )),
    };
    chunk.file_index += 16;

    let timestamp_created: i64 = chunk.read_i64()?;
    let timestamp_created: DateTime<Utc> = match DateTime::from_timestamp(timestamp_created, 0) {
        Some(timestamp) => timestamp,
        None => return Err(format!(
            "Invalid Timestamp {:016X} in chunk 'GEN8' at position {}.",
            timestamp_created,
            chunk.file_index
        )),
    };

    let display_name: GMStringRef = chunk.read_gm_string(strings)?;
    // probably not actually u64 (rather u32) but it's zero and there's null bytes surrounding it so idk
    let active_targets: u64 = chunk.read_u64()?;
    let function_classifications: GMFunctionClassifications = parse_function_classifications(chunk)?;
    let steam_appid: u32 = (-chunk.read_i32()?) as u32;
    let debugger_port: u16 = chunk.read_u32()? as u16;

    let end: usize = chunk.read_usize()? * 4 + 4;
    let mut room_order: Vec<u32> = vec![];

    while chunk.file_index < end {
        let room_id: u32 = chunk.read_u32()?;
        room_order.push(room_id);
    }

    Ok(GMGeneralInfo {
        is_debugger_disabled,
        bytecode_version,
        unknown_value,
        game_file_name,
        config,
        last_object_id,
        last_tile_id,
        game_id,
        directplay_guid,
        game_name,
        major_version,
        minor_version,
        release_version,
        stable_version,
        default_window_width,
        default_window_height,
        flags,
        license,
        timestamp_created,
        display_name,
        active_targets,
        function_classifications,
        steam_appid,
        debugger_port,
        room_order,
    })
}

fn parse_flags(chunk: &mut GMChunk) -> Result<GMGeneralInfoFlags, String> {
    let raw: u64 = chunk.read_u64()?;
    Ok(GMGeneralInfoFlags {
        fullscreen: 0 != raw & 0x0001,
        sync_vertex1: 0 != raw & 0x0002,
        sync_vertex2: 0 != raw & 0x0004,
        sync_vertex3: 0 != raw & 0x0100,
        interpolate: 0 != raw & 0x0008,
        scale: 0 != raw & 0x0010,
        show_cursor: 0 != raw & 0x0020,
        sizeable: 0 != raw & 0x0040,
        screen_key: 0 != raw & 0x0080,
        studio_version_b1: 0 != raw & 0x0200,
        studio_version_b2: 0 != raw & 0x0400,
        studio_version_b3: 0 != raw & 0x0800,
        steam_enabled: 0 != raw & 0x1000,
        local_data_enabled: 0 != raw & 0x2000,
        borderless_window: 0 != raw & 0x4000,
        javascript_mode: 0 != raw & 0x8000,
    })
}

fn parse_function_classifications(chunk: &mut GMChunk) -> Result<GMFunctionClassifications, String> {
    let raw: u64 = chunk.read_u64()?;
    Ok(GMFunctionClassifications {
        none: 0 != raw & 0x0,
        internet: 0 != raw & 0x1,
        joystick: 0 != raw & 0x2,
        gamepad: 0 != raw & 0x4,
        immersion: 0 != raw & 0x8,
        screengrab: 0 != raw & 0x10,
        math: 0 != raw & 0x20,
        action: 0 != raw & 0x40,
        matrix_d3d: 0 != raw & 0x80,
        d3dmodel: 0 != raw & 0x100,
        data_structures: 0 != raw & 0x200,
        file: 0 != raw & 0x400,
        ini: 0 != raw & 0x800,
        filename: 0 != raw & 0x1000,
        directory: 0 != raw & 0x2000,
        environment: 0 != raw & 0x4000,
        _unused1: 0 != raw & 0x8000,
        http: 0 != raw & 0x10000,
        encoding: 0 != raw & 0x20000,
        uidialog: 0 != raw & 0x40000,
        motion_planning: 0 != raw & 0x80000,
        shape_collision: 0 != raw & 0x100000,
        instance: 0 != raw & 0x200000,
        room: 0 != raw & 0x400000,
        game: 0 != raw & 0x800000,
        display: 0 != raw & 0x1000000,
        device: 0 != raw & 0x2000000,
        window: 0 != raw & 0x4000000,
        draw_color: 0 != raw & 0x8000000,
        texture: 0 != raw & 0x10000000,
        layer: 0 != raw & 0x20000000,
        string: 0 != raw & 0x40000000,
        tiles: 0 != raw & 0x80000000,
        surface: 0 != raw & 0x100000000,
        skeleton: 0 != raw & 0x200000000,
        io: 0 != raw & 0x400000000,
        variables: 0 != raw & 0x800000000,
        array: 0 != raw & 0x1000000000,
        external_call: 0 != raw & 0x2000000000,
        notification: 0 != raw & 0x4000000000,
        date: 0 != raw & 0x8000000000,
        particle: 0 != raw & 0x10000000000,
        sprite: 0 != raw & 0x20000000000,
        clickable: 0 != raw & 0x40000000000,
        legacy_sound: 0 != raw & 0x80000000000,
        audio: 0 != raw & 0x100000000000,
        event: 0 != raw & 0x200000000000,
        _unused2: 0 != raw & 0x400000000000,
        free_type: 0 != raw & 0x800000000000,
        analytics: 0 != raw & 0x1000000000000,
        unused3: 0 != raw & 0x2000000000000,
        unused4: 0 != raw & 0x4000000000000,
        achievement: 0 != raw & 0x8000000000000,
        cloud_saving: 0 != raw & 0x10000000000000,
        ads: 0 != raw & 0x20000000000000,
        os: 0 != raw & 0x40000000000000,
        iap: 0 != raw & 0x80000000000000,
        facebook: 0 != raw & 0x100000000000000,
        physics: 0 != raw & 0x200000000000000,
        flash_aa: 0 != raw & 0x400000000000000,
        console: 0 != raw & 0x800000000000000,
        buffer: 0 != raw & 0x1000000000000000,
        steam: 0 != raw & 0x2000000000000000,
        _unused3: 0 != raw & 2310346608841064448,
        shaders: 0 != raw & 0x4000000000000000,
        vertex_buffers: 0 != raw & 9223372036854775808,
    })
}

fn parse_options_flags(chunk: &mut GMChunk) -> Result<GMOptionsFlags, String> {
    let raw: u64 = chunk.read_u64()?;
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
        screen_shot_key: 0 != raw & 0x2000,
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


#[allow(non_snake_case)]
pub fn parse_chunk_OPTN(chunk: &mut GMChunk) -> Result<GMOptions, String> {
    chunk.file_index = 0;
    let _unused1: u32 = chunk.read_u32()?;
    let _unused2: u32 = chunk.read_u32()?;
    let flags: GMOptionsFlags = parse_options_flags(chunk)?;
    let scale: i32 = chunk.read_i32()?;
    let window_color_r: u8 = chunk.read_u8()?;
    let window_color_g: u8 = chunk.read_u8()?;
    let window_color_b: u8 = chunk.read_u8()?;
    let window_color_a: u8 = chunk.read_u8()?;
    let color_depth: u32 = chunk.read_u32()?;
    let resolution: u32 = chunk.read_u32()?;
    let frequency: u32 = chunk.read_u32()?;
    let vertex_sync: u32 = chunk.read_u32()?;
    let priority: u32 = chunk.read_u32()?;
    // CHANGE TYPES TO `texture page item` WHEN SUPPORTED
    let back_image: u32 = chunk.read_u32()?;
    let front_image: u32 = chunk.read_u32()?;
    let load_image: u32 = chunk.read_u32()?;
    // ^
    let load_alpha: u32 = chunk.read_u32()?;

    // constants missing

    Ok(GMOptions {
        _unused1,
        _unused2,
        flags,
        scale,
        window_color_r,
        window_color_g,
        window_color_b,
        window_color_a,
        color_depth,
        resolution,
        frequency,
        vertex_sync,
        priority,
        back_image,
        front_image,
        load_image,
        load_alpha,
    })
}

