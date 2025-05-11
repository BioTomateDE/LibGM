use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use chrono::{DateTime, Utc};
use crate::deserialize::strings::GMStrings;
use crate::deserialize::texture_page_items::{GMTexture, GMTextures};

#[derive(Debug, Clone)]
pub struct GMOptions {
    pub is_new_format: bool,
    pub unknown1: u32,
    pub unknown2: u32,
    pub flags: GMOptionsFlags,
    pub scale: i32,
    pub window_color: GMOptionsWindowColor,
    pub color_depth: u32,
    pub resolution: u32,
    pub frequency: u32,
    pub vertex_sync: u32,
    pub priority: u32,
    pub back_image: Option<GMRef<GMTexture>>,
    pub front_image: Option<GMRef<GMTexture>>,
    pub load_image: Option<GMRef<GMTexture>>,
    pub load_alpha: u32,
    pub constants: Vec<GMOptionsConstant>,
}

#[derive(Debug, Clone)]
pub struct GMGeneralInfo {
    pub is_debugger_disabled: bool,
    pub bytecode_version: u8,
    pub unknown_value: u16,
    pub game_file_name: GMRef<String>,
    pub config: GMRef<String>,
    pub game_id: u32,
    pub directplay_guid: uuid::Uuid,
    pub game_name: GMRef<String>,
    pub major_version: u32,
    pub minor_version: u32,
    pub release_version: u32,
    pub stable_version: u32,
    pub default_window_width: u32,
    pub default_window_height: u32,
    pub flags: GMGeneralInfoFlags,
    pub license_crc32: u32,
    pub license_md5: [u8; 16],
    pub timestamp_created: DateTime<Utc>,
    pub display_name: GMRef<String>,
    pub active_targets: u64,
    pub function_classifications: GMFunctionClassifications,
    pub steam_appid: i32,
    pub debugger_port: Option<u32>,
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
    pub steam_enabled: bool,
    pub local_data_enabled: bool,
    pub borderless_window: bool,
    pub javascript_mode: bool,
    pub license_exclusions: bool,
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
    pub unused1: bool,
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
    pub unused2: bool,
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
    pub unused5: bool,
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

#[derive(Debug, Clone)]
pub struct GMOptionsWindowColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone)]
pub struct GMOptionsConstant {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
}


pub fn parse_chunk_gen8(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMGeneralInfo, String> {
    chunk.cur_pos = 0;
    let is_debugger_disabled: bool = chunk.read_u8()? != 0;
    let bytecode_version: u8 = chunk.read_u8()?;
    let unknown_value: u16 = chunk.read_u16()?;
    let game_file_name: GMRef<String> = chunk.read_gm_string(strings)?;
    let config: GMRef<String> = chunk.read_gm_string(strings)?;
    let _last_object_id: u32 = chunk.read_u32()?;
    let _last_tile_id: u32 = chunk.read_u32()?;
    let game_id: u32 = chunk.read_u32()?;

    let directplay_guid: [u8; 16] = chunk.data.get(chunk.cur_pos..chunk.cur_pos + 16)
        .ok_or_else(|| format!(
            "Trying to read GUID out of bounds in chunk 'GEN8' at position {}: {} > {}.",
            chunk.cur_pos,
            chunk.cur_pos + 16,
            chunk.data.len(),
        ))?.try_into().expect("GUID length somehow not 16");
    chunk.cur_pos += 16;
    let directplay_guid: uuid::Uuid = uuid::Builder::from_bytes_le(directplay_guid).into_uuid();

    let game_name: GMRef<String> = chunk.read_gm_string(strings)?;
    let major_version: u32 = chunk.read_u32()?;
    let minor_version: u32 = chunk.read_u32()?;
    let release_version: u32 = chunk.read_u32()?;
    let stable_version: u32 = chunk.read_u32()?;
    let default_window_width: u32 = chunk.read_u32()?;
    let default_window_height: u32 = chunk.read_u32()?;
    let flags: GMGeneralInfoFlags = parse_general_info_flags(chunk.read_u32()?);
    let license_crc32: u32 = chunk.read_u32()?;

    let license_md5: [u8; 16] = chunk.data.get(chunk.cur_pos .. chunk.cur_pos + 16)
        .ok_or_else(|| format!(
            "Trying to read license (MD5) out of bounds in chunk 'GEN8' at position {}: {} > {}.",
            chunk.cur_pos,
            chunk.cur_pos + 16,
            chunk.data.len(),
        ))?.try_into().expect("GUID length somehow not 16");
    chunk.cur_pos += 16;

    let timestamp_created: i64 = chunk.read_i64()?;
    let timestamp_created: DateTime<Utc> = DateTime::from_timestamp(timestamp_created, 0)
        .ok_or_else(|| format!(
            "Invalid Creation Timestamp 0x{:016X} in chunk 'GEN8' at position {}.",
            timestamp_created,
            chunk.cur_pos
        ))?;

    let display_name: GMRef<String> = chunk.read_gm_string(strings)?;
    let active_targets: u64 = chunk.read_u64()?;
    let function_classifications: GMFunctionClassifications = parse_function_classifications(chunk.read_u64()?);
    let steam_appid: i32 = chunk.read_i32()?;
    let debugger_port: Option<u32> = if bytecode_version >= 14 { Some(chunk.read_u32()?) } else { None };

    let room_count: usize = chunk.read_usize()?;
    let mut room_order: Vec<u32> = Vec::with_capacity(room_count);
    for _ in 0..room_count {
        let room_id: u32 = chunk.read_u32()?;
        room_order.push(room_id);
    }

    Ok(GMGeneralInfo {
        is_debugger_disabled,
        bytecode_version,
        unknown_value,
        game_file_name,
        config,
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
        license_crc32,
        license_md5,
        timestamp_created,
        display_name,
        active_targets,
        function_classifications,
        steam_appid,
        debugger_port,
        room_order,
    })
}

fn parse_general_info_flags(raw: u32) -> GMGeneralInfoFlags{
    GMGeneralInfoFlags {
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
        license_exclusions: 0 != raw & 0x10000,
    }
}

fn parse_function_classifications(raw: u64) -> GMFunctionClassifications {
    GMFunctionClassifications {
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
        unused1: 0 != raw & 0x8000,
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
        unused2: 0 != raw & 0x400000000000,
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
        unused5: 0 != raw & 2310346608841064448,
        shaders: 0 != raw & 0x4000000000000000,
        vertex_buffers: 0 != raw & 9223372036854775808,
    }
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


pub fn parse_chunk_optn(chunk: &mut GMChunk, strings: &GMStrings, textures: &GMTextures) -> Result<GMOptions, String> {
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


fn parse_options_new(chunk: &mut GMChunk, strings: &GMStrings, textures: &GMTextures) -> Result<GMOptions, String> {
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
    let back_image: Option<GMRef<GMTexture>> = parse_options_image(chunk, textures)?;
    let front_image: Option<GMRef<GMTexture>> = parse_options_image(chunk, textures)?;
    let load_image: Option<GMRef<GMTexture>> = parse_options_image(chunk, textures)?;
    let load_alpha: u32 = chunk.read_u32()?;
    let constants: Vec<GMOptionsConstant> = parse_constants(chunk, strings)?;

    Ok(GMOptions {
        is_new_format: true,
        unknown1,
        unknown2,
        flags,
        scale,
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

fn parse_options_old(chunk: &mut GMChunk, strings: &GMStrings, textures: &GMTextures) -> Result<GMOptions, String> {
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

    let back_image: Option<GMRef<GMTexture>> = parse_options_image(chunk, textures)?;
    let front_image: Option<GMRef<GMTexture>> = parse_options_image(chunk, textures)?;
    let load_image: Option<GMRef<GMTexture>> = parse_options_image(chunk, textures)?;

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
        scale,
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


fn parse_constants(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMOptionsConstant>, String> {
    let constants_count: usize = chunk.read_usize()?;
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

fn parse_options_image(chunk: &mut GMChunk, textures: &GMTextures) -> Result<Option<GMRef<GMTexture>>, String> {
    let absolute_position: usize = chunk.read_usize()?;
    if absolute_position == 0 {
        return Ok(None)
    }

    let texture: GMRef<GMTexture> = textures.abs_pos_to_ref.get(&absolute_position)
        .ok_or_else(|| format!("Could not get Options image with absolute texture position {absolute_position}."))?
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

