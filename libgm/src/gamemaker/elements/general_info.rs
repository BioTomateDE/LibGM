mod flags;
mod function_classifications;
mod gms2;

use chrono::{DateTime, Utc};
pub use flags::Flags;
pub use function_classifications::FunctionClassifications;
pub use gms2::GMS2Data;
use uuid::Uuid;

use crate::{
    gamemaker::{
        chunk::ChunkName,
        data::Endianness,
        deserialize::reader::DataReader,
        elements::{GMChunk, GMElement, room::GMRoom},
        gm_version::{GMVersion, GMVersionReq, LTSBranch},
        reference::GMRef,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GMGeneralInfo {
    /// Indicates whether debugging support is disabled.
    pub is_debugger_disabled: bool,

    /// The WAD version of the data file.
    /// WAD stands for "Where's All the Data".
    ///
    /// This field is also known as "**bytecode** version".
    ///
    /// Technically, this is the version of the data file format.
    /// However, since YoYoGames does not update this version
    /// specification anymore, it has become worthless past ~16.
    /// Since they also don't update the GameMaker Studio version
    /// (`version`), GameMaker unpacking tools have to resort to
    /// version detection (which sucks).
    /// This can detect the approximate Studio version this
    /// game's data file was made in. Since studio versions
    /// are something official that does get incremented (finally!),
    /// this [`GMVersion`] is used in place of the WAD version
    /// in modern version/feature detection.
    ///
    /// ___
    /// See `version` for less information.
    pub wad_version: u8,

    /// Who knows. Probably redundant in GMS2.
    pub unknown_value: u16,

    /// The file name of the runner.
    pub game_file_name: String,

    /// Which GameMaker configuration the data file was compiled with.
    pub config: String,

    /// The last game object ID of the data file.
    pub last_object_id: u32,

    /// The last tile ID of the data file.
    pub last_tile_id: u32,

    /// The game id of the data file, whatever that may mean.
    pub game_id: u32,

    /// The `DirectPlay` GUID of the data file.
    /// This is always empty in GameMaker Studio.
    pub directplay_guid: Uuid,

    /// The name of the game.
    pub game_name: String,

    /// The GameMaker Studio Version this game's data file was made in.
    /// For GameMaker 2 games, this will be specified as 2.0.0.0,
    /// but `detect_version.rs` will detect the actual version later.
    ///
    /// Technically, this is the studio version; not the
    /// data file version. However, YoYoGames. *YoYoGames........*
    ///
    /// Note that this does not have to correspond to the actual studio version.
    /// This can be due to multiple reasons:
    /// * The data file does not use a specific newer feature,
    ///   resulting in a **lower** detected version.
    /// * There is a bug in the version detection logic (oopsies),
    ///   resulting in a **higher** detected version (false positive).
    /// * Fucking LTS.
    ///   For some reason, they added a BREAKING FEATURE
    ///   to a LONG TERM SUPPORT branch.
    ///   This means that tools like this have to differentiate
    ///   between LTS-pre-this-feature and LTS-post-this-feature.
    ///   As a result, some games made in 2022.0 LTS may be shown
    ///   as 2023.6 instead.
    ///
    /// ___
    /// See `wad_version` for more information.
    pub version: GMVersion,

    /// The default window width of the game window.
    pub default_window_width: u32,

    /// The default window height of the game window.
    pub default_window_height: u32,

    /// The info flags of the data file.
    pub flags: Flags,

    /// The CRC32 of the license used to compile the game.
    pub license_crc32: u32,

    /// The MD5 of the license used to compile the game.
    pub license_md5: [u8; 16],

    /// The timestamp the game was compiled at.
    pub timestamp_created: DateTime<Utc>,

    /// The name that gets displayed in the window title.
    pub display_name: String,

    /// The function classifications of this data file.
    pub function_classifications: FunctionClassifications,

    /// The Steam app ID of the game.
    /// This may be zero.
    pub steam_appid: i32,

    /// The port the data file exposes for the debugger.
    /// Only set in WAD14+.
    pub debugger_port: Option<u32>,

    /// The room order of the data file.
    pub room_order: Vec<GMRef<GMRoom>>,

    /// Set in GameMaker 2+ data files.
    pub gms2_data: Option<GMS2Data>,

    pub exists: bool,
}

impl Default for GMGeneralInfo {
    /// Should only be used as a small stub in `DataReader` because
    /// Rust doesn't have nullables ([`Option`]s are too ugly for this).
    /// ___________
    /// **This value should never be used!**
    /// Immediately replace it with actual `GEN8` when parsed.
    fn default() -> Self {
        Self {
            is_debugger_disabled: true,
            wad_version: 67,
            unknown_value: 0,
            game_file_name: String::new(),
            config: String::new(),
            last_object_id: 100_000,
            last_tile_id: 10_000_000,
            game_id: 1337,
            directplay_guid: Uuid::default(),
            game_name: String::new(),
            version: GMVersion::stub(),
            default_window_width: 1337,
            default_window_height: 1337,
            flags: Flags::default(),
            license_crc32: 1337,
            license_md5: [0; 16],
            timestamp_created: DateTime::default(),
            display_name: String::new(),
            function_classifications: FunctionClassifications::default(),
            steam_appid: 0,
            debugger_port: None,
            room_order: vec![],
            gms2_data: None,
            exists: false,
        }
    }
}

impl GMChunk for GMGeneralInfo {
    const NAME: ChunkName = ChunkName::new("GEN8");
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
            },
        };
        let wad_version = reader.read_u8()?;
        let unknown_value = reader.read_u16()?;
        let game_file_name: String = reader.read_gm_string()?;
        let config: String = reader.read_gm_string()?;
        let last_object_id = reader.read_u32()?;
        let last_tile_id = reader.read_u32()?;
        let game_id = reader.read_u32()?;

        let directplay_guid: [u8; 16] = *reader.read_bytes_const().context("reading GUID")?;
        let uuid_parser = match reader.endianness {
            Endianness::Little => uuid::Builder::from_bytes_le,
            Endianness::Big => uuid::Builder::from_bytes, // unconfirmed
        };
        let directplay_guid: Uuid = uuid_parser(directplay_guid).into_uuid();

        let game_name: String = reader.read_gm_string()?;
        let version = GMVersion::deserialize(reader)?;
        let default_window_width = reader.read_u32()?;
        let default_window_height = reader.read_u32()?;
        let flags_raw = reader.read_u32()?;
        let flags = Flags::parse(flags_raw);
        let license_crc32 = reader.read_u32()?;
        let license_md5: [u8; 16] = *reader.read_bytes_const().context("reading license (MD5)")?;

        let timestamp_created = reader.read_i64()?;
        let timestamp_created: DateTime<Utc> = DateTime::from_timestamp(timestamp_created, 0)
            .ok_or_else(|| format!("Invalid Creation Timestamp {timestamp_created}"))?;

        let display_name: String = reader.read_gm_string()?;
        let active_targets = reader.read_u64()?;
        reader.assert_int(active_targets, 0, "Active Targets")?;
        let function_classifications = FunctionClassifications::deserialize(reader)?;
        let steam_appid = reader.read_i32()?;
        let debugger_port: Option<u32> = reader.deserialize_if_wad_version(14)?;
        let room_order: Vec<GMRef<GMRoom>> = reader.read_simple_list()?;

        let mut general_info = Self {
            is_debugger_disabled,
            wad_version,
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
            gms2_data: None,
            exists: true,
        };

        if general_info.version.major >= 2 {
            let gms2 = general_info.read_gms2_data(reader)?;
            general_info.gms2_data = Some(gms2);
        }

        Ok(general_info)
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if !self.exists {
            bail!("General info is a required chunk");
        }

        builder.write_u8(self.is_debugger_disabled.into());
        builder.write_u8(self.wad_version);
        builder.write_u16(self.unknown_value);
        builder.write_gm_string(&self.game_file_name);
        builder.write_gm_string(&self.config);
        builder.write_u32(self.last_object_id);
        builder.write_u32(self.last_tile_id);
        builder.write_u32(self.game_id);
        builder.write_bytes(self.directplay_guid.to_bytes_le().as_slice());
        builder.write_gm_string(&self.game_name);

        let version = if self.version.major == 1 {
            &self.version
        } else {
            &GMVersion::new(2, 0, 0, 0, LTSBranch::PreLTS)
        };
        version.serialize(builder)?;

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
        self.debugger_port
            .serialize_if_wad_ver(builder, "Debugger Port", 14)?;

        builder.write_simple_list(&self.room_order)?;

        if builder.is_gm_version_at_least((2, 0)) {
            self.write_gms2_data(builder)?;
        }
        Ok(())
    }
}
