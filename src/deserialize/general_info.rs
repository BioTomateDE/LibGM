use std::fmt::Formatter;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use chrono::{DateTime, Utc};
use crate::deserialize::rooms::GMRoom;
use crate::deserialize::strings::GMStrings;


#[derive(Debug, Clone)]
pub struct GMGeneralInfo {
    pub is_debugger_disabled: bool,
    pub bytecode_version: u8,
    pub unknown_value: u16,
    pub game_file_name: GMRef<String>,
    pub config: GMRef<String>,
    pub last_object_id: u32,
    pub last_tile_id: u32,
    pub game_id: u32,
    pub directplay_guid: uuid::Uuid,
    pub game_name: GMRef<String>,
    pub version: GMVersion,
    pub default_window_width: u32,
    pub default_window_height: u32,
    pub flags: GMGeneralInfoFlags,
    pub license_crc32: u32,
    pub license_md5: [u8; 16],
    pub timestamp_created: DateTime<Utc>,
    pub display_name: GMRef<String>,
    pub active_targets: u64,        // TODO make a flags struct for this
    pub function_classifications: GMFunctionClassifications,
    pub steam_appid: i32,
    pub debugger_port: Option<u32>,
    pub room_order: Vec<GMRef<GMRoom>>,
}
impl GMGeneralInfo {
    pub fn is_version_at_least(&self, major: u32, minor: u32, release: u32, build: u32) -> bool {
        if self.version.major != major {
            return self.version.major > major;
        }
        if self.version.minor != minor {
            return self.version.minor > minor;
        }
        if self.version.release != release {
            return self.version.release > release;
        }
        if self.version.stable != build {
            return self.version.stable > build;
        }
        true   // The version is exactly what was supplied.
    }
    
    /// Should only be used as a small stub in GMReader because Rust doesn't have nullables (options are too ugly for this).
    /// ___
    /// **THIS VALUE SHOULD NEVER BE USED! IMMEDIATELY REPLACE IT WITH ACTUAL GEN8 WHEN PARSED.**
    pub fn stub() -> Self {
        Self {
            is_debugger_disabled: true,
            bytecode_version: 187,
            unknown_value: 187,
            game_file_name: GMRef::new(69420),
            config: GMRef::new(69420),
            last_object_id: 69420,
            last_tile_id: 69420,
            game_id: 69420,
            directplay_guid: Default::default(),
            game_name: GMRef::new(69420),
            version: GMVersion {
                major: 69420,
                minor: 69420,
                release: 69420,
                stable: 69420,
            },
            default_window_width: 69420,
            default_window_height: 69420,
            flags: GMGeneralInfoFlags {
                fullscreen: false,
                sync_vertex1: false,
                sync_vertex2: false,
                sync_vertex3: false,
                interpolate: false,
                scale: false,
                show_cursor: false,
                sizeable: false,
                screen_key: false,
                studio_version_b1: false,
                studio_version_b2: false,
                studio_version_b3: false,
                steam_enabled: false,
                local_data_enabled: false,
                borderless_window: false,
                javascript_mode: false,
                license_exclusions: false,
            },
            license_crc32: 69420,
            license_md5: [6,9,4,2,0, 6,9,4,2,0, 6,9,4,2,0, 0],
            timestamp_created: Default::default(),
            display_name: GMRef::new(69420),
            active_targets: 69420,
            function_classifications: GMFunctionClassifications {
                none: false,
                internet: false,
                joystick: false,
                gamepad: false,
                immersion: false,
                screengrab: false,
                math: false,
                action: false,
                matrix_d3d: false,
                d3dmodel: false,
                data_structures: false,
                file: false,
                ini: false,
                filename: false,
                directory: false,
                environment: false,
                unused1: false,
                http: false,
                encoding: false,
                uidialog: false,
                motion_planning: false,
                shape_collision: false,
                instance: false,
                room: false,
                game: false,
                display: false,
                device: false,
                window: false,
                draw_color: false,
                texture: false,
                layer: false,
                string: false,
                tiles: false,
                surface: false,
                skeleton: false,
                io: false,
                variables: false,
                array: false,
                external_call: false,
                notification: false,
                date: false,
                particle: false,
                sprite: false,
                clickable: false,
                legacy_sound: false,
                audio: false,
                event: false,
                unused2: false,
                free_type: false,
                analytics: false,
                unused3: false,
                unused4: false,
                achievement: false,
                cloud_saving: false,
                ads: false,
                os: false,
                iap: false,
                facebook: false,
                physics: false,
                flash_aa: false,
                console: false,
                buffer: false,
                steam: false,
                unused5: false,
                shaders: false,
                vertex_buffers: false,
            },
            steam_appid: 69420,
            debugger_port: None,
            room_order: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVersion {
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub stable: u32,
}
impl std::fmt::Display for GMVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.major, self.minor, self.release, self.stable)
    }
}
impl GMVersion {
    pub fn new(major: u32, minor: u32, release: u32, stable: u32) -> Self {
        Self { major, minor, release, stable }
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


pub fn parse_chunk_gen8(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMGeneralInfo, String> {
    chunk.cur_pos = 0;
    let is_debugger_disabled: bool = chunk.read_u8()? != 0;
    let bytecode_version: u8 = chunk.read_u8()?;
    let unknown_value: u16 = chunk.read_u16()?;
    let game_file_name: GMRef<String> = chunk.read_gm_string(strings)?;
    let config: GMRef<String> = chunk.read_gm_string(strings)?;
    let last_object_id: u32 = chunk.read_u32()?;
    let last_tile_id: u32 = chunk.read_u32()?;
    let game_id: u32 = chunk.read_u32()?;

    let directplay_guid: [u8; 16] = *chunk.read_bytes_const()
        .map_err(|e| format!("Trying to read GUID {e}"))?;
    let directplay_guid: uuid::Uuid = uuid::Builder::from_bytes_le(directplay_guid).into_uuid();

    let game_name: GMRef<String> = chunk.read_gm_string(strings)?;
    let version: GMVersion = parse_version(chunk)?;
    let default_window_width: u32 = chunk.read_u32()?;
    let default_window_height: u32 = chunk.read_u32()?;
    let flags: GMGeneralInfoFlags = parse_general_info_flags(chunk.read_u32()?);
    let license_crc32: u32 = chunk.read_u32()?;

    let license_md5: [u8; 16] = *chunk.read_bytes_const()
        .map_err(|e| format!("Trying to read license (MD5) {e}"))?;

    let timestamp_created: i64 = chunk.read_i64()?;
    let timestamp_created: DateTime<Utc> = DateTime::from_timestamp(timestamp_created, 0)
        .ok_or_else(|| format!(
            "Invalid Creation Timestamp 0x{:016X} in chunk 'GEN8' at position {}",
            timestamp_created, chunk.cur_pos,
        ))?;

    let display_name: GMRef<String> = chunk.read_gm_string(strings)?;
    let active_targets: u64 = chunk.read_u64()?;
    let function_classifications: GMFunctionClassifications = parse_function_classifications(chunk.read_u64()?);
    let steam_appid: i32 = chunk.read_i32()?;
    let debugger_port: Option<u32> = if bytecode_version >= 14 { Some(chunk.read_u32()?) } else { None };

    let room_count: usize = chunk.read_usize_count()?;
    let mut room_order: Vec<GMRef<GMRoom>> = Vec::with_capacity(room_count);
    for _ in 0..room_count {
        let room_id: usize = chunk.read_usize_count()?;
        room_order.push(GMRef::new(room_id));
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
        version,
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


fn parse_version(chunk: &mut GMChunk) -> Result<GMVersion, String> {
    let major: u32 = chunk.read_u32()?;
    let minor: u32 = chunk.read_u32()?;
    let release: u32 = chunk.read_u32()?;
    let stable: u32 = chunk.read_u32()?;
    Ok(GMVersion::new(major, minor, release, stable))
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

