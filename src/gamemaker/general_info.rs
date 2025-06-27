use std::fmt::Formatter;
use chrono::{DateTime, Utc};
use crate::csharp_rng::CSharpRng;
use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::rooms::GMRoom;
use crate::gm_serialize::{DataBuilder, GMSerializeIfVersion};

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
    pub active_targets: u64,
    pub function_classifications: GMFunctionClassifications,
    pub steam_appid: i32,
    pub debugger_port: Option<u32>,
    pub room_order: Vec<GMRef<GMRoom>>,
    pub gms2_info: Option<GMGeneralInfoGMS2>,
    pub exists: bool,
}
impl GMChunkElement for GMGeneralInfo {
    /// Should only be used as a small stub in GMReader because Rust doesn't have nullables (options are too ugly for this).
    /// ___
    /// **THIS VALUE SHOULD NEVER BE USED! IMMEDIATELY REPLACE IT WITH ACTUAL GEN8 WHEN PARSED.**
    fn empty() -> Self {
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
                build: 69420,
                branch: LTSBranch::Post2022_0,
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
            license_md5: [6, 9, 4, 2, 0, 6, 9, 4, 2, 0, 6, 9, 4, 2, 0, 0],
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
            gms2_info: None,
            exists: false,
        }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMGeneralInfo {
    pub fn is_version_at_least<V: Into<GMVersionReq>>(&self, version_req: V) -> bool {
        self.version.is_version_at_least(version_req)
    }
    pub fn set_version_at_least<V: Into<GMVersionReq>>(&mut self, version_req: V) -> Result<(), String> {
        self.version.set_version_at_least(version_req)
    }
}

impl GMElement for GMGeneralInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let is_debugger_disabled: bool = match reader.read_u8()? {
            0 => false,
            1 => true,
            other => return Err(format!("Invalid u8 bool {other} while reading general info \"is debugger disabled\"")),
        };
        let bytecode_version: u8 = reader.read_u8()?;
        let unknown_value: u16 = reader.read_u16()?;
        let game_file_name: GMRef<String> = reader.read_gm_string()?;
        let config: GMRef<String> = reader.read_gm_string()?;
        let last_object_id: u32 = reader.read_u32()?;
        let last_tile_id: u32 = reader.read_u32()?;
        let game_id: u32 = reader.read_u32()?;

        let directplay_guid: [u8; 16] = *reader.read_bytes_const()
            .map_err(|e| format!("Trying to read GUID {e}"))?;
        let directplay_guid: uuid::Uuid = uuid::Builder::from_bytes_le(directplay_guid).into_uuid();

        let game_name: GMRef<String> = reader.read_gm_string()?;
        let version = GMVersion::deserialize(reader)?;
        let default_window_width: u32 = reader.read_u32()?;
        let default_window_height: u32 = reader.read_u32()?;
        let flags_raw: u32 = reader.read_u32()?;
        let flags = GMGeneralInfoFlags::parse(flags_raw);
        let license_crc32: u32 = reader.read_u32()?;

        let license_md5: [u8; 16] = *reader.read_bytes_const()
            .map_err(|e| format!("Trying to read license (MD5) {e}"))?;

        let timestamp_created: i64 = reader.read_i64()?;
        let timestamp_created: DateTime<Utc> = DateTime::from_timestamp(timestamp_created, 0)
            .ok_or_else(|| format!(
                "Invalid Creation Timestamp 0x{:016X} in chunk 'GEN8' at position {}",
                timestamp_created, reader.cur_pos,
            ))?;

        let display_name: GMRef<String> = reader.read_gm_string()?;
        let active_targets: u64 = reader.read_u64()?;
        let function_classifications = GMFunctionClassifications::deserialize(reader)?;
        let steam_appid: i32 = reader.read_i32()?;
        let debugger_port: Option<u32> = if bytecode_version >= 14 { Some(reader.read_u32()?) } else { None };
        let room_order: Vec<GMRef<GMRoom>> = reader.read_simple_list_of_resource_ids()?;

        let mut gms2_info: Option<GMGeneralInfoGMS2> = None;
        if version.major >= 2 {
            // Parse and verify UUID
            let timestamp: i64 = timestamp_created.timestamp();
            let mut info_timestamp_offset: bool = true;
            let seed: i32 = (timestamp & 0xFFFFFFFF) as i32;
            let mut rng = CSharpRng::new(seed);

            let first_expected: i64 = ((rng.next() as i64) << 32) | (rng.next() as i64);
            let first_actual: i64 = reader.read_i64()?;
            if first_actual != first_expected {
                return Err(format!("Unexpected random UID #1: expected {first_expected}; got {first_actual}"));
            }

            let info_location: i32 = ((timestamp & 0xFFFF) as i32 / 7
                + game_id.wrapping_sub(default_window_width) as i32
                + room_order.len() as i32).abs() % 4;
            let mut random_uid: Vec<i64> = Vec::with_capacity(4);

            let get_info_number = |first_random: i64, info_timestamp_offset: bool| -> i64 {
                let mut info_number: i64 = timestamp;
                if info_timestamp_offset {
                    info_number -= 1000;
                }
                info_number = Self::uid_bitmush(info_number);
                info_number ^= first_random;
                info_number = !info_number;
                info_number ^= ((game_id as i64) << 32) | (game_id as i64);
                info_number ^= (default_window_width as i64 + flags_raw as i64) << 48 |
                    (default_window_height as i64 + flags_raw as i64) << 32 |
                    (default_window_height as i64 + flags_raw as i64) << 16 |
                    (default_window_width as i64 + flags_raw as i64);
                info_number ^= bytecode_version as i64;
                info_number
            };

            for i in 0..4 {
                if i == info_location {
                    let curr: i64 = reader.read_i64()?;
                    random_uid.push(curr);

                    if curr != get_info_number(first_expected, true) {
                        if curr != get_info_number(first_expected, false) {
                            return Err("Unexpected random UID info".to_string());
                        } else {
                            info_timestamp_offset = false;
                        }
                    }
                } else {
                    let second_actual: u32 = reader.read_u32()?;
                    let third_actual: u32 = reader.read_u32()?;
                    let second_expected: u32 = rng.next() as u32;
                    let third_expected: u32 = rng.next() as u32;
                    if second_actual != second_expected {
                        return Err(format!("Unexpected random UID #2: expected {second_expected}; got {second_actual}"));
                    }
                    if third_actual != third_expected {
                        return Err(format!("Unexpected random UID #3: expected {third_expected}; got {third_actual}"));
                    }

                    random_uid.push(((second_actual as i64) << 32) | (third_actual as i64));
                }
            }
            let fps: f32 = reader.read_f32()?;
            let allow_statistics: bool = reader.read_bool32()?;
            let game_guid: [u8; 16] = reader.read_bytes_const::<16>()?.to_owned();
            gms2_info = Some(GMGeneralInfoGMS2 {
                random_uid,
                fps,
                allow_statistics,
                game_guid,
                info_timestamp_offset,
            })
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
            gms2_info,
            exists: true,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists {
            return Err("General info was never deserialized".to_string())
        }
        builder.write_u8(if self.is_debugger_disabled {1} else {0});
        builder.write_u8(self.bytecode_version);
        builder.write_u16(self.unknown_value);
        builder.write_gm_string(&self.game_file_name)?;
        builder.write_gm_string(&self.config)?;
        builder.write_u32(self.last_object_id);
        builder.write_u32(self.last_tile_id);
        builder.write_u32(self.game_id);
        builder.write_bytes(self.directplay_guid.to_bytes_le().as_slice());
        builder.write_gm_string(&self.game_name)?;
        if self.version.major == 1 {
            self.version.serialize(builder)?;
        } else {    // yoyogames moment
            builder.write_u32(2);
            builder.write_u32(0);
            builder.write_u32(0);
            builder.write_u32(0);
        }
        builder.write_u32(self.default_window_width);
        builder.write_u32(self.default_window_height);
        self.flags.serialize(builder)?;
        builder.write_u32(self.license_crc32);
        builder.write_bytes(&self.license_md5);
        builder.write_i64(self.timestamp_created.timestamp());
        builder.write_gm_string(&self.display_name)?;
        builder.write_u64(self.active_targets);
        self.function_classifications.serialize(builder)?;
        builder.write_i32(self.steam_appid);
        self.debugger_port.serialize_if_bytecode_ver(builder, "Debugger Port", 14)?;
        builder.write_usize(self.room_order.len())?;
        for room_ref in &self.room_order {
            builder.write_resource_id(room_ref);
        }
        if builder.is_gm_version_at_least((2, 0)) {
            // Write random UID
            let gms2_info: &GMGeneralInfoGMS2 = self.gms2_info.as_ref().ok_or("GMS2 Data not set in General Info")?;
            let timestamp: i64 = self.timestamp_created.timestamp();
            let seed: i32 = (timestamp & 0xFFFFFFFF) as i32;
            let mut rng = CSharpRng::new(seed);
            let first_random: i64 = ((rng.next() as i64) << 32) | rng.next() as i64;
            let info_number = self.get_info_number(first_random, gms2_info.info_timestamp_offset);
            let info_location: i32 = ((timestamp & 0xFFFF) as i32 / 7 
                + self.game_id.wrapping_sub(self.default_window_width) as i32 
                + self.room_order.len() as i32).abs() % 4;
            builder.write_i64(first_random);
            for i in 0..4 {
                if i == info_location {
                    builder.write_i64(info_number);
                } else {
                    let first: u32 = rng.next() as u32;
                    let second: u32 = rng.next() as u32;
                    builder.write_u32(first);
                    builder.write_u32(second);
                }
            }

            builder.write_f32(gms2_info.fps);
            builder.write_bool32(gms2_info.allow_statistics);
            builder.write_bytes(&gms2_info.game_guid);
        }
        Ok(())
    }
}
impl GMGeneralInfo {
    fn get_info_number(&self, first_random: i64, info_timestamp_offset: bool) -> i64 {
        let flags_raw: u32 = self.flags.build();
        let mut info_number: i64 = self.timestamp_created.timestamp();
        if info_timestamp_offset {
            info_number -= 1000;
        }
        info_number = Self::uid_bitmush(info_number);
        info_number ^= first_random;
        info_number = !info_number;
        info_number ^= ((self.game_id as i64) << 32) | (self.game_id as i64);
        info_number ^= (self.default_window_width as i64 + flags_raw as i64) << 48 |
            (self.default_window_height as i64 + flags_raw as i64) << 32 |
            (self.default_window_height as i64 + flags_raw as i64) << 16 |
            (self.default_window_width as i64 + flags_raw as i64);
        info_number ^= self.bytecode_version as i64;
        info_number
    }

    fn uid_bitmush(info_number: i64) -> i64 {
        let mut temp: u64 = info_number as u64;
        temp = (temp << 56 & 0xFF00_0000_0000_0000) |
            (temp >> 08 & 0x00FF_0000_0000_0000) |
            (temp << 32 & 0x0000_FF00_0000_0000) |
            (temp >> 16 & 0x0000_00FF_0000_0000) |
            (temp << 08 & 0x0000_0000_FF00_0000) |
            (temp >> 24 & 0x0000_0000_00FF_0000) |
            (temp >> 16 & 0x0000_0000_0000_FF00) |
            (temp >> 32 & 0x0000_0000_0000_00FF);
        temp as i64
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMGeneralInfoGMS2 {
    pub random_uid: Vec<i64>,
    pub fps: f32,
    pub allow_statistics: bool,
    pub game_guid: [u8; 16],
    pub info_timestamp_offset: bool,
}


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LTSBranch {
    Pre2022_0,
    LTS2022_0,
    Post2022_0,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVersion {
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub build: u32,
    pub branch: LTSBranch,
}
impl GMVersion {
    pub fn new(major: u32, minor: u32, release: u32, build: u32, branch: LTSBranch) -> Self {
        Self { major, minor, release, build, branch }
    }
}
impl std::fmt::Display for GMVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let branch_str = match self.branch {
            LTSBranch::Pre2022_0 => "pre2022_0",
            LTSBranch::LTS2022_0 => "lts2022_0",
            LTSBranch::Post2022_0 => "post2022_0",
        };
        write!(f, "{}.{}.{}.{} ({branch_str})", self.major, self.minor, self.release, self.build)
    }
}
impl GMElement for GMVersion {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let major: u32 = reader.read_u32()?;
        let minor: u32 = reader.read_u32()?;
        let release: u32 = reader.read_u32()?;
        let build: u32 = reader.read_u32()?;
        // since gen8 gm version is stuck on maximum 2.0.0.0; LTS will (initially) always be Pre2022_0
        Ok(GMVersion::new(major, minor, release, build, LTSBranch::Pre2022_0))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.major);
        builder.write_u32(self.minor);
        builder.write_u32(self.release);
        builder.write_u32(self.build);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVersionReq {
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub build: u32,
    pub non_lts: bool,
}
impl GMVersionReq {
    pub fn none() -> Self {
        Self {
            major: 0,
            minor: 0,
            release: 0,
            build: 0,
            non_lts: false,
        }
    }
}
impl From<(u32, u32)> for GMVersionReq {
    fn from((major, minor): (u32, u32)) -> Self {
        Self {
            major,
            minor,
            release: 0,
            build: 0,
            non_lts: false,
        }
    }
}
impl From<(u32, u32, u32)> for GMVersionReq {
    fn from((major, minor, release): (u32, u32, u32)) -> Self {
        Self {
            major,
            minor,
            release,
            build: 0,
            non_lts: false,
        }
    }
}
impl From<(u32, u32, u32, u32)> for GMVersionReq {
    fn from((major, minor, release, build): (u32, u32, u32, u32)) -> Self {
        Self {
            major,
            minor,
            release,
            build,
            non_lts: false,
        }
    }
}
impl From<(u32, u32, LTSBranch)> for GMVersionReq {
    fn from((major, minor, lts): (u32, u32, LTSBranch)) -> Self {
        Self {
            major,
            minor,
            release: 0,
            build: 0,
            non_lts: matches!(lts, LTSBranch::Post2022_0),
        }
    }
}
impl From<(u32, u32, u32, LTSBranch)> for GMVersionReq {
    fn from((major, minor, release, lts): (u32, u32, u32, LTSBranch)) -> Self {
        Self {
            major,
            minor,
            release,
            build: 0,
            non_lts: matches!(lts, LTSBranch::Post2022_0),
        }
    }
}
impl From<(u32, u32, u32, u32, LTSBranch)> for GMVersionReq {
    fn from((major, minor, release, build, lts): (u32, u32, u32, u32, LTSBranch)) -> Self {
        Self {
            major,
            minor,
            release,
            build,
            non_lts: matches!(lts, LTSBranch::Post2022_0),
        }
    }
}
impl std::fmt::Display for GMVersionReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lts_str = if self.non_lts { " (Non LTS)" } else { "" };
        write!(f, "{}.{}.{}.{}{lts_str}", self.major, self.minor, self.release, self.build)
    }
}
impl GMVersion {
    pub fn is_version_at_least<V: Into<GMVersionReq>>(&self, version_req: V) -> bool {
        let ver: GMVersionReq = version_req.into();
        if ver.non_lts && self.branch < LTSBranch::Post2022_0 {
            return false
        }
        if self.major != ver.major {
            return self.major > ver.major;
        }
        if self.minor != ver.minor {
            return self.minor > ver.minor;
        }
        if self.release != ver.release {
            return self.release > ver.release;
        }
        if self.build != ver.build {
            return self.build > ver.build;
        }
        true   // The version is exactly what was supplied.
    }

    pub fn set_version_at_least<V: Into<GMVersionReq>>(&mut self, version_req: V) -> Result<(), String> {
        let new_ver: GMVersionReq = version_req.into();
        if !matches!(new_ver.major, 2|2022|2023|2024) {
            return Err(format!(
                "Tried to set GameMaker Version to {} which is not allowed for original GameMaker Version {}",
                new_ver, self,
            ))
        }
        if self.is_version_at_least(new_ver.clone()) {
            return Ok(())   // only override version if new version is higher
        }
        self.major = new_ver.major;
        self.minor = new_ver.minor;
        self.release = new_ver.release;
        self.build = new_ver.build;
        if new_ver.non_lts {
            self.branch = LTSBranch::Post2022_0;
        }
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMGeneralInfoFlags {
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
impl GMElement for GMGeneralInfoFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let raw: u32 = reader.read_u32()?;
        Ok(Self::parse(raw))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.build());
        Ok(())
    }
}

impl GMGeneralInfoFlags {
    fn parse(raw: u32) -> Self {
        Self {
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
    fn build(&self) -> u32 {
        let mut raw: u32 = 0;
        if self.fullscreen {raw |= 0x0001};
        if self.sync_vertex1 {raw |= 0x0002};
        if self.sync_vertex2 {raw |= 0x0004};
        if self.sync_vertex3 {raw |= 0x0100};
        if self.interpolate {raw |= 0x0008};
        if self.scale {raw |= 0x0010};
        if self.show_cursor {raw |= 0x0020};
        if self.sizeable {raw |= 0x0040};
        if self.screen_key {raw |= 0x0080};
        if self.studio_version_b1 {raw |= 0x0200};
        if self.studio_version_b2 {raw |= 0x0400};
        if self.studio_version_b3 {raw |= 0x0800};
        if self.steam_enabled {raw |= 0x1000};
        if self.local_data_enabled {raw |= 0x2000};
        if self.borderless_window {raw |= 0x4000};
        if self.javascript_mode {raw |= 0x8000};
        raw
    }
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
impl GMElement for GMFunctionClassifications {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let raw: u64 = reader.read_u64()?;
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
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let mut raw: u64 = 0;
        if self.none {raw |= 0x0};
        if self.internet {raw |= 0x1};
        if self.joystick {raw |= 0x2};
        if self.gamepad {raw |= 0x4};
        if self.immersion {raw |= 0x8};
        if self.screengrab {raw |= 0x10};
        if self.math {raw |= 0x20};
        if self.action {raw |= 0x40};
        if self.matrix_d3d {raw |= 0x80};
        if self.d3dmodel {raw |= 0x100};
        if self.data_structures {raw |= 0x200};
        if self.file {raw |= 0x400};
        if self.ini {raw |= 0x800};
        if self.filename {raw |= 0x1000};
        if self.directory {raw |= 0x2000};
        if self.environment {raw |= 0x4000};
        if self.unused1 {raw |= 0x8000};
        if self.http {raw |= 0x10000};
        if self.encoding {raw |= 0x20000};
        if self.uidialog {raw |= 0x40000};
        if self.motion_planning {raw |= 0x80000};
        if self.shape_collision {raw |= 0x100000};
        if self.instance {raw |= 0x200000};
        if self.room {raw |= 0x400000};
        if self.game {raw |= 0x800000};
        if self.display {raw |= 0x1000000};
        if self.device {raw |= 0x2000000};
        if self.window {raw |= 0x4000000};
        if self.draw_color {raw |= 0x8000000};
        if self.texture {raw |= 0x10000000};
        if self.layer {raw |= 0x20000000};
        if self.string {raw |= 0x40000000};
        if self.tiles {raw |= 0x80000000};
        if self.surface {raw |= 0x100000000};
        if self.skeleton {raw |= 0x200000000};
        if self.io {raw |= 0x400000000};
        if self.variables {raw |= 0x800000000};
        if self.array {raw |= 0x1000000000};
        if self.external_call {raw |= 0x2000000000};
        if self.notification {raw |= 0x4000000000};
        if self.date {raw |= 0x8000000000};
        if self.particle {raw |= 0x10000000000};
        if self.sprite {raw |= 0x20000000000};
        if self.clickable {raw |= 0x40000000000};
        if self.legacy_sound {raw |= 0x80000000000};
        if self.audio {raw |= 0x100000000000};
        if self.event {raw |= 0x200000000000};
        if self.unused2 {raw |= 0x400000000000};
        if self.free_type {raw |= 0x800000000000};
        if self.analytics {raw |= 0x1000000000000};
        if self.unused3 {raw |= 0x2000000000000};
        if self.unused4 {raw |= 0x4000000000000};
        if self.achievement {raw |= 0x8000000000000};
        if self.cloud_saving {raw |= 0x10000000000000};
        if self.ads {raw |= 0x20000000000000};
        if self.os {raw |= 0x40000000000000};
        if self.iap {raw |= 0x80000000000000};
        if self.facebook {raw |= 0x100000000000000};
        if self.physics {raw |= 0x200000000000000};
        if self.flash_aa {raw |= 0x400000000000000};
        if self.console {raw |= 0x800000000000000};
        if self.buffer {raw |= 0x1000000000000000};
        if self.steam {raw |= 0x2000000000000000};
        if self.unused5 {raw |= 2310346608841064448};
        if self.shaders {raw |= 0x4000000000000000};
        if self.vertex_buffers {raw |= 9223372036854775808};
        builder.write_u64(raw);
        Ok(())
    }
}

