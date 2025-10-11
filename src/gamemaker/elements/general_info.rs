use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::rooms::GMRoom;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::{GMVersion, GMVersionReq, LTSBranch};
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::rng::CSharpRng;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct GMGeneralInfo {
    /// Indicates whether debugging support is disabled.
    pub is_debugger_disabled: bool,

    /// The bytecode version of the data file.
    pub bytecode_version: u8,

    pub unknown_value: u16,

    /// The file name of the runner.
    pub game_file_name: GMRef<String>,

    /// Which GameMaker configuration the data file was compiled with.
    pub config: GMRef<String>,

    /// The last object id of the data file.
    pub last_object_id: u32,

    /// The last tile id of the data file.
    pub last_tile_id: u32,

    /// The game id of the data file.
    pub game_id: u32,

    /// The DirectPlay GUID of the data file.
    /// This is always empty in GameMaker Studio.
    pub directplay_guid: uuid::Uuid,

    /// The name of the game.
    pub game_name: GMRef<String>,

    /// The version of the data file. For GameMaker 2 games, this will be specified as 2.0.0.0,
    /// but `detect_version.rs` will detect the actual version later.
    pub version: GMVersion,

    /// The default window width of the game window.
    pub default_window_width: u32,

    /// The default window height of the game window.
    pub default_window_height: u32,

    /// The info flags of the data file.
    pub flags: GMGeneralInfoFlags,

    /// The CRC32 of the license used to compile the game.
    pub license_crc32: u32,

    /// The MD5 of the license used to compile the game.
    pub license_md5: [u8; 16],

    /// The UNIX timestamp the game was compiled.
    pub timestamp_created: DateTime<Utc>,

    /// The name that gets displayed in the window.
    pub display_name: GMRef<String>,

    /// Unknown/unused.
    pub active_targets: u64,

    /// The function classifications of this data file.
    pub function_classifications: GMFunctionClassifications,

    /// The Steam app id of the game.
    pub steam_appid: i32,

    /// The port the data file exposes for the debugger.
    pub debugger_port: Option<u32>,

    /// The room order of the data file.
    pub room_order: Vec<GMRef<GMRoom>>,

    /// Set in GameMaker 2+ data files.
    pub gms2_info: Option<GMGeneralInfoGMS2>,

    pub exists: bool,
}

impl GMChunkElement for GMGeneralInfo {
    /// Should only be used as a small stub in GMReader because Rust doesn't have nullables (options are too ugly for this).
    /// ___
    /// **THIS VALUE SHOULD NEVER BE USED! IMMEDIATELY REPLACE IT WITH ACTUAL GEN8 WHEN PARSED.**
    fn stub() -> Self {
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
                branch: LTSBranch::PostLTS,
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
    pub fn set_version_at_least<V: Into<GMVersionReq>>(&mut self, version_req: V) -> Result<()> {
        self.version.set_version_at_least(version_req)
    }
}

impl GMElement for GMGeneralInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let is_debugger_disabled: bool = match reader.read_u8()? {
            0 => false,
            1 => true,
            other => {
                bail!("Invalid u8 bool {other} while reading general info \"is debugger disabled\"")
            }
        };
        let bytecode_version = reader.read_u8()?;
        let unknown_value = reader.read_u16()?;
        let game_file_name: GMRef<String> = reader.read_gm_string()?;
        let config: GMRef<String> = reader.read_gm_string()?;
        let last_object_id = reader.read_u32()?;
        let last_tile_id = reader.read_u32()?;
        let game_id = reader.read_u32()?;

        let directplay_guid: [u8; 16] = *reader
            .read_bytes_const()
            .map_err(|e| format!("Trying to read GUID {e}"))?;
        let directplay_guid: uuid::Uuid = uuid::Builder::from_bytes_le(directplay_guid).into_uuid();

        let game_name: GMRef<String> = reader.read_gm_string()?;
        let version = GMVersion::deserialize(reader)?;
        let default_window_width = reader.read_u32()?;
        let default_window_height = reader.read_u32()?;
        let flags_raw = reader.read_u32()?;
        let flags = GMGeneralInfoFlags::parse(flags_raw);
        let license_crc32 = reader.read_u32()?;

        let license_md5: [u8; 16] = *reader
            .read_bytes_const()
            .map_err(|e| format!("Trying to read license (MD5) {e}"))?;

        let timestamp_created = reader.read_i64()?;
        let timestamp_created: DateTime<Utc> = DateTime::from_timestamp(timestamp_created, 0).with_context(|| {
            format!(
                "Invalid Creation Timestamp 0x{:016X} in chunk 'GEN8' at position {}",
                timestamp_created, reader.cur_pos,
            )
        })?;

        let display_name: GMRef<String> = reader.read_gm_string()?;
        let active_targets = reader.read_u64()?;
        let function_classifications = GMFunctionClassifications::deserialize(reader)?;
        let steam_appid = reader.read_i32()?;
        let debugger_port: Option<u32> = if bytecode_version >= 14 {
            Some(reader.read_u32()?)
        } else {
            None
        };
        let room_order: Vec<GMRef<GMRoom>> = reader.read_simple_list_of_resource_ids()?;

        let mut gms2_info: Option<GMGeneralInfoGMS2> = None;
        if version.major >= 2 {
            // Parse and verify UUID
            let timestamp: i64 = timestamp_created.timestamp();
            let mut info_timestamp_offset: bool = true;
            let seed: i32 = (timestamp & 0xFFFFFFFF) as i32;
            let mut rng = CSharpRng::new(seed);

            let first_expected: i64 = ((rng.next() as i64) << 32) | (rng.next() as i64);
            let first_actual = reader.read_i64()?;
            if first_actual != first_expected {
                bail!("Unexpected random UID #1: expected {first_expected}; got {first_actual}");
            }

            let info_location: i32 = ((timestamp & 0xFFFF) as i32 / 7
                + game_id.wrapping_sub(default_window_width) as i32
                + room_order.len() as i32)
                .abs()
                % 4;
            let mut random_uid = [0_i64; 4];

            let get_info_number = |first_random: i64, info_timestamp_offset: bool| -> i64 {
                let mut info_number: i64 = timestamp;
                if info_timestamp_offset {
                    info_number -= 1000;
                }
                info_number = Self::uid_bitmush(info_number);
                info_number ^= first_random;
                info_number = !info_number;
                info_number ^= ((game_id as i64) << 32) | (game_id as i64);
                info_number ^= (default_window_width as i64 + flags_raw as i64) << 48
                    | (default_window_height as i64 + flags_raw as i64) << 32
                    | (default_window_height as i64 + flags_raw as i64) << 16
                    | (default_window_width as i64 + flags_raw as i64);
                info_number ^= bytecode_version as i64;
                info_number
            };

            for i in 0..4 {
                if i == info_location {
                    let curr = reader.read_i64()?;
                    random_uid[i as usize] = curr;

                    if curr != get_info_number(first_expected, true) {
                        if curr != get_info_number(first_expected, false) {
                            bail!("Unexpected random UID info");
                        } else {
                            info_timestamp_offset = false;
                        }
                    }
                } else {
                    let second_actual = reader.read_u32()?;
                    let third_actual = reader.read_u32()?;
                    let second_expected: u32 = rng.next() as u32;
                    let third_expected: u32 = rng.next() as u32;
                    if second_actual != second_expected {
                        bail!("Unexpected random UID #2: expected {second_expected}; got {second_actual}");
                    }
                    if third_actual != third_expected {
                        bail!("Unexpected random UID #3: expected {third_expected}; got {third_actual}");
                    }

                    random_uid[i as usize] = ((second_actual as i64) << 32) | (third_actual as i64);
                }
            }
            let fps = reader.read_f32()?;
            let allow_statistics = reader.read_bool32()?;
            let game_guid: [u8; 16] = reader
                .read_bytes_const::<16>()
                .cloned()
                .map_err(|e| format!("Trying to read Game GUID {e}"))?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if !self.exists {
            bail!("General info was never deserialized");
        }
        builder.write_u8(if self.is_debugger_disabled { 1 } else { 0 });
        builder.write_u8(self.bytecode_version);
        builder.write_u16(self.unknown_value);
        builder.write_gm_string(&self.game_file_name)?;
        builder.write_gm_string(&self.config)?;
        builder.write_u32(self.last_object_id);
        builder.write_u32(self.last_tile_id);
        builder.write_u32(self.game_id);
        builder.write_bytes(self.directplay_guid.to_bytes_le().as_slice());
        builder.write_gm_string(&self.game_name)?;
        self.version.serialize(builder)?; // Technically incorrect but idc
        // if self.version.major == 1 {
        //     self.version.serialize(builder)?;
        // } else {    // Yoyogames moment
        //     builder.write_u32(2);
        //     builder.write_u32(0);
        //     builder.write_u32(0);
        //     builder.write_u32(0);
        // }
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
        self.debugger_port
            .serialize_if_bytecode_ver(builder, "Debugger Port", 14)?;
        builder.write_usize(self.room_order.len())?;
        for room_ref in &self.room_order {
            builder.write_resource_id(room_ref);
        }
        if builder.is_gm_version_at_least((2, 0)) {
            // Write random UID
            let gms2_info: &GMGeneralInfoGMS2 = self.gms2_info.as_ref().context("GMS2 Data not set in General Info")?;
            let timestamp: i64 = self.timestamp_created.timestamp();
            let seed: i32 = (timestamp & 0xFFFFFFFF) as i32;
            let mut rng = CSharpRng::new(seed);
            let first_random: i64 = ((rng.next() as i64) << 32) | rng.next() as i64;
            let info_number = self.get_info_number(first_random, gms2_info.info_timestamp_offset);
            let info_location: i32 = ((timestamp & 0xFFFF) as i32 / 7
                + self.game_id.wrapping_sub(self.default_window_width) as i32
                + self.room_order.len() as i32)
                .abs()
                % 4;
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
        info_number ^= (self.default_window_width as i64 + flags_raw as i64) << 48
            | (self.default_window_height as i64 + flags_raw as i64) << 32
            | (self.default_window_height as i64 + flags_raw as i64) << 16
            | (self.default_window_width as i64 + flags_raw as i64);
        info_number ^= self.bytecode_version as i64;
        info_number
    }

    fn uid_bitmush(info_number: i64) -> i64 {
        let mut temp: u64 = info_number as u64;
        temp = (temp << 56 & 0xFF00_0000_0000_0000)
            | (temp >> 08 & 0x00FF_0000_0000_0000)
            | (temp << 32 & 0x0000_FF00_0000_0000)
            | (temp >> 16 & 0x0000_00FF_0000_0000)
            | (temp << 08 & 0x0000_0000_FF00_0000)
            | (temp >> 24 & 0x0000_0000_00FF_0000)
            | (temp >> 16 & 0x0000_0000_0000_FF00)
            | (temp >> 32 & 0x0000_0000_0000_00FF);
        temp as i64
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMGeneralInfoGMS2 {
    /// Unknown, some sort of checksum.
    pub random_uid: [i64; 4],

    /// The FPS of the game.
    pub fps: f32,

    /// If enabled, the game runner may send requests to a GameMaker player count statistics server.
    pub allow_statistics: bool,

    /// Unknown, some sort of checksum.
    pub game_guid: [u8; 16],

    /// Whether the random UID's timestamp was initially offset.
    pub info_timestamp_offset: bool,
}

#[derive(Debug, Clone)]
pub struct GMGeneralInfoFlags {
    /// Start the game as fullscreen.
    pub fullscreen: bool,

    /// Use synchronization to avoid tearing.
    pub sync_vertex1: bool,

    /// Use synchronization to avoid tearing. (???)
    pub sync_vertex2: bool,

    /// Use synchronization to avoid tearing. (???)
    pub sync_vertex3: bool,

    /// Interpolate colours between pixels.
    pub interpolate: bool,

    /// Keep aspect ratio during scaling.
    pub scale: bool,

    /// Display mouse cursor.
    pub show_cursor: bool,

    /// Allows window to be resized.
    pub sizeable: bool,

    /// Allows fullscreen switching. (???)
    pub screen_key: bool,

    pub studio_version_b1: bool,

    pub studio_version_b2: bool,

    pub studio_version_b3: bool,

    pub steam_enabled: bool,

    pub local_data_enabled: bool,

    /// Whether the game supports borderless window
    pub borderless_window: bool,

    /// Tells the runner to run Javascript code
    pub javascript_mode: bool,

    pub license_exclusions: bool,
}

impl GMElement for GMGeneralInfoFlags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw = reader.read_u32()?;
        Ok(Self::parse(raw))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
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
        raw |= self.fullscreen as u32 * 0x0001;
        raw |= self.sync_vertex1 as u32 * 0x0002;
        raw |= self.sync_vertex2 as u32 * 0x0004;
        raw |= self.sync_vertex3 as u32 * 0x0100;
        raw |= self.interpolate as u32 * 0x0008;
        raw |= self.scale as u32 * 0x0010;
        raw |= self.show_cursor as u32 * 0x0020;
        raw |= self.sizeable as u32 * 0x0040;
        raw |= self.screen_key as u32 * 0x0080;
        raw |= self.studio_version_b1 as u32 * 0x0200;
        raw |= self.studio_version_b2 as u32 * 0x0400;
        raw |= self.studio_version_b3 as u32 * 0x0800;
        raw |= self.steam_enabled as u32 * 0x1000;
        raw |= self.local_data_enabled as u32 * 0x2000;
        raw |= self.borderless_window as u32 * 0x4000;
        raw |= self.javascript_mode as u32 * 0x8000;
        raw |= self.license_exclusions as u32 * 0x10000;
        raw
    }
}

#[derive(Debug, Clone)]
pub struct GMFunctionClassifications {
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
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw = reader.read_u64()?;
        Ok(GMFunctionClassifications {
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
            unused5: 0 != raw & 0x2010000000000000,
            shaders: 0 != raw & 0x4000000000000000,
            vertex_buffers: 0 != raw & 0x8000000000000000,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut raw: u64 = 0;
        raw |= self.internet as u64 * 0x1;
        raw |= self.joystick as u64 * 0x2;
        raw |= self.gamepad as u64 * 0x4;
        raw |= self.immersion as u64 * 0x8;
        raw |= self.screengrab as u64 * 0x10;
        raw |= self.math as u64 * 0x20;
        raw |= self.action as u64 * 0x40;
        raw |= self.matrix_d3d as u64 * 0x80;
        raw |= self.d3dmodel as u64 * 0x100;
        raw |= self.data_structures as u64 * 0x200;
        raw |= self.file as u64 * 0x400;
        raw |= self.ini as u64 * 0x800;
        raw |= self.filename as u64 * 0x1000;
        raw |= self.directory as u64 * 0x2000;
        raw |= self.environment as u64 * 0x4000;
        raw |= self.unused1 as u64 * 0x8000;
        raw |= self.http as u64 * 0x10000;
        raw |= self.encoding as u64 * 0x20000;
        raw |= self.uidialog as u64 * 0x40000;
        raw |= self.motion_planning as u64 * 0x80000;
        raw |= self.shape_collision as u64 * 0x100000;
        raw |= self.instance as u64 * 0x200000;
        raw |= self.room as u64 * 0x400000;
        raw |= self.game as u64 * 0x800000;
        raw |= self.display as u64 * 0x1000000;
        raw |= self.device as u64 * 0x2000000;
        raw |= self.window as u64 * 0x4000000;
        raw |= self.draw_color as u64 * 0x8000000;
        raw |= self.texture as u64 * 0x10000000;
        raw |= self.layer as u64 * 0x20000000;
        raw |= self.string as u64 * 0x40000000;
        raw |= self.tiles as u64 * 0x80000000;
        raw |= self.surface as u64 * 0x100000000;
        raw |= self.skeleton as u64 * 0x200000000;
        raw |= self.io as u64 * 0x400000000;
        raw |= self.variables as u64 * 0x800000000;
        raw |= self.array as u64 * 0x1000000000;
        raw |= self.external_call as u64 * 0x2000000000;
        raw |= self.notification as u64 * 0x4000000000;
        raw |= self.date as u64 * 0x8000000000;
        raw |= self.particle as u64 * 0x10000000000;
        raw |= self.sprite as u64 * 0x20000000000;
        raw |= self.clickable as u64 * 0x40000000000;
        raw |= self.legacy_sound as u64 * 0x80000000000;
        raw |= self.audio as u64 * 0x100000000000;
        raw |= self.event as u64 * 0x200000000000;
        raw |= self.unused2 as u64 * 0x400000000000;
        raw |= self.free_type as u64 * 0x800000000000;
        raw |= self.analytics as u64 * 0x1000000000000;
        raw |= self.unused3 as u64 * 0x2000000000000;
        raw |= self.unused4 as u64 * 0x4000000000000;
        raw |= self.achievement as u64 * 0x8000000000000;
        raw |= self.cloud_saving as u64 * 0x10000000000000;
        raw |= self.ads as u64 * 0x20000000000000;
        raw |= self.os as u64 * 0x40000000000000;
        raw |= self.iap as u64 * 0x80000000000000;
        raw |= self.facebook as u64 * 0x100000000000000;
        raw |= self.physics as u64 * 0x200000000000000;
        raw |= self.flash_aa as u64 * 0x400000000000000;
        raw |= self.console as u64 * 0x800000000000000;
        raw |= self.buffer as u64 * 0x1000000000000000;
        raw |= self.steam as u64 * 0x2000000000000000;
        raw |= self.unused5 as u64 * 0x2010000000000000;
        raw |= self.shaders as u64 * 0x4000000000000000;
        raw |= self.vertex_buffers as u64 * 0x8000000000000000;
        builder.write_u64(raw);
        Ok(())
    }
}
