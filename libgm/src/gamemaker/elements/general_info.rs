use crate::gamemaker::data::Endianness;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::rooms::GMRoom;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::{GMVersion, GMVersionReq};
use crate::gamemaker::reference::GMRef;
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::assert::assert_int;
use crate::util::bitfield::bitfield_struct;
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
    pub game_file_name: String,

    /// Which `GameMaker` configuration the data file was compiled with.
    pub config: String,

    /// The last object id of the data file.
    pub last_object_id: u32,

    /// The last tile id of the data file.
    pub last_tile_id: u32,

    /// The game id of the data file.
    pub game_id: u32,

    /// The DirectPlay GUID of the data file.
    /// This is always empty in `GameMaker` Studio.
    pub directplay_guid: uuid::Uuid,

    /// The name of the game.
    pub game_name: String,

    /// The version of the data file. For `GameMaker` 2 games, this will be specified as 2.0.0.0,
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
    pub display_name: String,

    /// The function classifications of this data file.
    pub function_classifications: GMFunctionClassifications,

    /// The Steam app id of the game.
    pub steam_appid: i32,

    /// The port the data file exposes for the debugger.
    pub debugger_port: Option<u32>,

    /// The room order of the data file.
    pub room_order: Vec<GMRef<GMRoom>>,

    /// Set in `GameMaker` 2+ data files.
    pub gms2_info: Option<GMGeneralInfoGMS2>,

    pub exists: bool,
}

impl Default for GMGeneralInfo {
    /// Should only be used as a small stub in `DataReader` because
    /// Rust doesn't have nullables ([`Option`]s are too ugly for this).
    /// ___________
    /// **This value should never be used!** Immediately replace it with actual `GEN8` when parsed.
    fn default() -> Self {
        Self {
            is_debugger_disabled: true,
            bytecode_version: 67,
            unknown_value: 0,
            game_file_name: "".to_string(),
            config: "".to_string(),
            last_object_id: 100_000,
            last_tile_id: 10_000_000,
            game_id: 1337,
            directplay_guid: Default::default(),
            game_name: "".to_string(),
            version: GMVersion::stub(),
            default_window_width: 1337,
            default_window_height: 1337,
            flags: GMGeneralInfoFlags::default(),
            license_crc32: 1337,
            license_md5: [0; 16],
            timestamp_created: Default::default(),
            display_name: "".to_string(),
            function_classifications: GMFunctionClassifications::default(),
            steam_appid: 0,
            debugger_port: None,
            room_order: vec![],
            gms2_info: None,
            exists: false,
        }
    }
}

impl GMChunkElement for GMGeneralInfo {
    const NAME: &'static str = "GEN8";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMGeneralInfo {
    pub fn is_version_at_least<V: Into<GMVersionReq>>(
        &self,
        version_req: V,
    ) -> bool {
        self.version.is_version_at_least(version_req)
    }
    pub fn set_version_at_least<V: Into<GMVersionReq>>(
        &mut self,
        version_req: V,
    ) -> Result<()> {
        self.version.set_version_at_least(version_req)
    }
}

impl GMElement for GMGeneralInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let is_debugger_disabled: bool = match reader.read_u8()? {
            0 => false,
            1 => true,
            other => {
                bail!(
                    "Invalid u8 bool {other} while reading general info \"is debugger disabled\""
                )
            }
        };
        let bytecode_version = reader.read_u8()?;
        let unknown_value = reader.read_u16()?;
        let game_file_name: String = reader.read_gm_string()?;
        let config: String = reader.read_gm_string()?;
        let last_object_id = reader.read_u32()?;
        let last_tile_id = reader.read_u32()?;
        let game_id = reader.read_u32()?;

        let directplay_guid: [u8; 16] =
            *reader.read_bytes_const().context("reading GUID")?;
        let uuid_parser = match reader.endianness {
            Endianness::Little => uuid::Builder::from_bytes_le,
            Endianness::Big => uuid::Builder::from_bytes, // unconfirmed
        };
        let directplay_guid: uuid::Uuid =
            uuid_parser(directplay_guid).into_uuid();

        let game_name: String = reader.read_gm_string()?;
        let version = GMVersion::deserialize(reader)?;
        let default_window_width = reader.read_u32()?;
        let default_window_height = reader.read_u32()?;
        let flags_raw = reader.read_u32()?;
        let flags = GMGeneralInfoFlags::parse(flags_raw);
        let license_crc32 = reader.read_u32()?;
        let license_md5: [u8; 16] =
            *reader.read_bytes_const().context("reading license (MD5)")?;

        let timestamp_created = reader.read_i64()?;
        let timestamp_created: DateTime<Utc> =
            DateTime::from_timestamp(timestamp_created, 0).ok_or_else(
                || format!("Invalid Creation Timestamp {timestamp_created}"),
            )?;

        let display_name: String = reader.read_gm_string()?;
        let active_targets = reader.read_u64()?;
        assert_int("Active Targets", 0, active_targets)?;
        let function_classifications =
            GMFunctionClassifications::deserialize(reader)?;
        let steam_appid = reader.read_i32()?;
        let debugger_port: Option<u32> =
            reader.deserialize_if_bytecode_version(14)?;
        let room_order: Vec<GMRef<GMRoom>> =
            reader.read_simple_list_of_resource_ids()?;

        let mut gms2_info: Option<GMGeneralInfoGMS2> = None;
        if version.major >= 2 {
            // Parse and verify UUID
            let timestamp: i64 = timestamp_created.timestamp();
            let mut info_timestamp_offset: bool = true;
            let seed: i32 = (timestamp & 0xFFFFFFFF) as i32;
            let mut rng = CSharpRng::new(seed);

            let first_expected: i64 =
                ((rng.next() as i64) << 32) | (rng.next() as i64);
            let first_actual = reader.read_i64()?;
            if first_actual != first_expected {
                bail!(
                    "Unexpected random UID #1: expected {first_expected}; got {first_actual}"
                );
            }

            let info_location: i32 = ((timestamp & 0xFFFF) as i32 / 7
                + game_id.wrapping_sub(default_window_width) as i32
                + room_order.len() as i32)
                .abs()
                % 4;
            let mut random_uid = [0_i64; 4];

            let get_info_number = |first_random: i64,
                                   info_timestamp_offset: bool|
             -> i64 {
                let mut info_number: i64 = timestamp;
                if info_timestamp_offset {
                    info_number -= 1000;
                }
                info_number = Self::uid_bitmush(info_number);
                info_number ^= first_random;
                info_number = !info_number;
                info_number ^= ((game_id as i64) << 32) | (game_id as i64);
                info_number ^= (default_window_width as i64 + flags_raw as i64)
                    << 48
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
                        }
                        info_timestamp_offset = false;
                    }
                } else {
                    let second_actual = reader.read_u32()?;
                    let third_actual = reader.read_u32()?;
                    let second_expected: u32 = rng.next() as u32;
                    let third_expected: u32 = rng.next() as u32;
                    if second_actual != second_expected {
                        bail!(
                            "Unexpected random UID #2: expected {second_expected}; got {second_actual}"
                        );
                    }
                    if third_actual != third_expected {
                        bail!(
                            "Unexpected random UID #3: expected {third_expected}; got {third_actual}"
                        );
                    }

                    random_uid[i as usize] =
                        ((second_actual as i64) << 32) | (third_actual as i64);
                }
            }
            let fps = reader.read_f32()?;
            let allow_statistics = reader.read_bool32()?;
            let game_guid: [u8; 16] = reader
                .read_bytes_const::<16>()
                .cloned()
                .context("reading Game GUID")?;
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
        builder.write_gm_string(&self.game_file_name);
        builder.write_gm_string(&self.config);
        builder.write_u32(self.last_object_id);
        builder.write_u32(self.last_tile_id);
        builder.write_u32(self.game_id);
        builder.write_bytes(self.directplay_guid.to_bytes_le().as_slice());
        builder.write_gm_string(&self.game_name);
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
        builder.write_gm_string(&self.display_name);
        builder.write_u64(0); // "Active targets"
        self.function_classifications.serialize(builder)?;
        builder.write_i32(self.steam_appid);
        self.debugger_port.serialize_if_bytecode_ver(
            builder,
            "Debugger Port",
            14,
        )?;
        builder.write_usize(self.room_order.len())?;
        for room_ref in &self.room_order {
            builder.write_resource_id(*room_ref);
        }
        if builder.is_gm_version_at_least((2, 0)) {
            // Write random UID
            let gms2_info: &GMGeneralInfoGMS2 = self
                .gms2_info
                .as_ref()
                .ok_or("GMS2 Data not set in General Info")?;
            let timestamp: i64 = self.timestamp_created.timestamp();
            let seed: i32 = (timestamp & 0xFFFFFFFF) as i32;
            let mut rng = CSharpRng::new(seed);
            let first_random: i64 =
                ((rng.next() as i64) << 32) | rng.next() as i64;
            let info_number = self
                .get_info_number(first_random, gms2_info.info_timestamp_offset);
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
    fn get_info_number(
        &self,
        first_random: i64,
        info_timestamp_offset: bool,
    ) -> i64 {
        let flags_raw: u32 = self.flags.build();
        let mut info_number: i64 = self.timestamp_created.timestamp();
        if info_timestamp_offset {
            info_number -= 1000;
        }
        info_number = Self::uid_bitmush(info_number);
        info_number ^= first_random;
        info_number = !info_number;
        info_number ^= ((self.game_id as i64) << 32) | (self.game_id as i64);
        info_number ^= (self.default_window_width as i64 + flags_raw as i64)
            << 48
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

    /// If enabled, the game runner may send requests to a `GameMaker` player count statistics server.
    pub allow_statistics: bool,

    /// Unknown, some sort of checksum.
    pub game_guid: [u8; 16],

    /// Whether the random UID's timestamp was initially offset.
    pub info_timestamp_offset: bool,
}

bitfield_struct! {
    /// Contains general information flags for `GameMaker` games.
    GMGeneralInfoFlags : u32 {
        /// Start the game as fullscreen.
        fullscreen: 0,

        /// Use synchronization to avoid tearing.
        sync_vertex1: 1,

        /// Use synchronization to avoid tearing. (???)
        sync_vertex2: 2,

        /// Use synchronization to avoid tearing. (???)
        sync_vertex3: 8,

        /// Interpolate colours between pixels.
        interpolate: 3,

        /// Keep aspect ratio during scaling.
        scale: 4,

        /// Display mouse cursor.
        show_cursor: 5,

        /// Allows window to be resized.
        sizeable: 6,

        /// Allows fullscreen switching. (???)
        screen_key: 7,

        studio_version_b1: 9,

        studio_version_b2: 10,

        studio_version_b3: 11,

        steam_enabled: 12,

        local_data_enabled: 13,

        /// Whether the game supports borderless window
        borderless_window: 14,

        /// Tells the runner to run Javascript code
        javascript_mode: 15,

        license_exclusions: 16,
    }
}

bitfield_struct! {
    GMFunctionClassifications : u64 {
        internet: 0,
        joystick: 1,
        gamepad: 2,
        immersion: 3,
        screengrab: 4,
        math: 5,
        action: 6,
        matrix_d3d: 7,
        d3d_model: 8,
        data_structures: 9,
        file: 10,
        ini: 11,
        filename: 12,
        directory: 13,
        environment: 14,
        http: 16,
        encoding: 17,
        ui_dialog: 18,
        motion_planning: 19,
        shape_collision: 20,
        instance: 21,
        room: 22,
        game: 23,
        display: 24,
        device: 25,
        window: 26,
        draw_color: 27,
        texture: 28,
        layer: 29,
        string: 30,
        tiles: 31,
        surface: 32,
        skeleton: 33,
        io: 34,
        variables: 35,
        array: 36,
        external_call: 37,
        notification: 38,
        date: 39,
        particle: 40,
        sprite: 41,
        clickable: 42,
        legacy_sound: 43,
        audio: 44,
        event: 45,
        free_type: 47,
        analytics: 48,
        achievement: 51,
        cloud_saving: 52,
        ads: 53,
        os: 54,
        in_app_purchases: 55,
        facebook: 56,
        physics: 57,
        flash_aa: 58,
        console: 59,
        buffer: 60,
        steam: 61,
        shaders: 62,
        vertex_buffers: 63,
    }
}
