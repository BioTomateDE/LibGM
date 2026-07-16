// SPDX-License-Identifier: GPL-3.0-only
mod flags;
mod function_classifications;
mod gms2;

use chrono::DateTime;
use chrono::Utc;

pub use self::flags::Flags;
pub use self::function_classifications::FunctionClassifications;
pub use self::gms2::GMS2Data;
use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::ChunkName;
use crate::wad::elem::GMElement;
use crate::wad::elem::room::Room;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::version::GMVersion;
use crate::wad::version::IdeVersion;

#[derive(Debug, Clone, PartialEq)]
pub struct GeneralInfo {
    /// Indicates whether debugging support via an external GameMaker debugger is enabled.
    /// The game will crash (?) if this is enabled and there is no debugger.
    ///
    /// This bool is stored as an inverted [`u8`] in the data file.
    pub debugger_enabled: bool,

    /// The WAD version of the data file (aka bytecode version).
    ///
    /// WAD stands for "Where's All the Data".
    ///
    /// Technically, this is the version of the data file format.
    /// However, since YoYo Games does not update this version
    /// specification anymore, it has become worthless past ~16.
    ///
    /// See [`GMVersion`] for the real format version.
    pub wad_version: u8,

    /// Who knows. Probably redundant in GMS2.
    pub unknown_value: u16,

    /// The file name of the runner.
    pub game_file_name: GMRef<String>,

    /// Which GameMaker configuration the data file was compiled with.
    pub config: GMRef<String>,

    /// The last game object ID of the data file.
    pub last_object_id: u32,

    /// The last tile ID of the data file.
    pub last_tile_id: u32,

    /// The game id of the data file, whatever that may mean.
    pub game_id: u32,

    /// The `DirectPlay` GUID of the data file.
    /// This is always empty in GameMaker Studio.
    pub directplay_guid: Blob<[u8; 16]>,

    /// The name of the game.
    pub game_name: GMRef<String>,

    /// The GameMaker version of the IDE this game was created in.
    ///
    /// This version struct is not updated by YoYo Games since GMS 2 and is
    /// is stuck on `2.0.0.0` since GameMaker Studio: 2 (released November 2016).
    /// If you need the format version of the data file, check out [`GMVersion`].
    pub ide_version: IdeVersion,

    /// When the game window is created, its width will be set to this value.
    ///
    /// This can still be overridden in GML code via `window_set_width` or `window_set_size`.
    pub window_width: u32,

    /// When the game window is created, its height will be set to this value.
    ///
    /// This can still be overridden in GML code via `window_set_height` or `window_set_size`.
    pub window_height: u32,

    /// The info flags of the data file.
    pub flags: Flags,

    /// The CRC32 of the license used to compile the game.
    pub license_crc32: u32,

    /// The MD5 of the license used to compile the game.
    pub license_md5: Blob<[u8; 16]>,

    /// The timestamp the game was compiled at.
    pub creation_timestamp: DateTime<Utc>,

    /// The name that gets displayed in the window title.
    pub display_name: GMRef<String>,

    /// The function classifications of this data file.
    pub function_classifications: FunctionClassifications,

    /// The Steam app ID of the game.
    /// This may be zero.
    pub steam_appid: i32,

    /// The port the data file exposes for the debugger.
    ///
    /// This may be set (non-zero) in WAD Version 14 and higher.
    pub debugger_port: u32,

    /// The room order of the data file.
    pub room_order: Vec<GMRef<Room>>,

    /// Set in GameMaker 2+ data files.
    pub gms2_data: Option<GMS2Data>,
}

impl GMChunk for GeneralInfo {
    const NAME: ChunkName = ChunkName::GEN8;
}

impl GMElement for GeneralInfo {
    // be sure not to use `reader.general_info` here, as it is not initialized yet!
    // this includes methods like `deserialize_if_gm_version`.
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let debugger_enabled: bool = match reader.read_u8()? {
            1 => false,
            0 => true,
            other => bail!("Invalid 'Is Debugger Disabled' u8 bool {other}"),
        };
        let wad_version = reader.read_u8()?;
        let unknown_value = reader.read_u16()?;
        let game_file_name: GMRef<String> = reader.read_gm_string()?;
        let config: GMRef<String> = reader.read_gm_string()?;
        let last_object_id = reader.read_u32()?;
        let last_tile_id = reader.read_u32()?;
        let game_id = reader.read_u32()?;
        let directplay_guid: [u8; 16] = *reader.read_bytes_const().ctx("reading GUID")?;
        let game_name: GMRef<String> = reader.read_gm_string()?;
        let ide_version = IdeVersion::deserialize(reader)?;
        let window_width = reader.read_u32()?;
        let window_height = reader.read_u32()?;
        let flags_raw = reader.read_u32()?;
        let flags = Flags::from_bits(flags_raw)
            .ok_or_else(|| format!("Invalid GEN8 Flags {flags_raw:08X}"))?;
        let license_crc32 = reader.read_u32()?;
        let license_md5: [u8; 16] = *reader.read_bytes_const().ctx("reading license (MD5)")?;

        let creation_timestamp = reader.read_i64()?;
        let creation_timestamp: DateTime<Utc> =
            DateTime::from_timestamp_secs(creation_timestamp)
                .ok_or_else(|| format!("Invalid Creation Timestamp {creation_timestamp}"))?;

        let display_name: GMRef<String> = reader.read_gm_string()?;
        let active_targets = reader.read_u64()?;
        reader.assert_int(active_targets, 0, "Active Targets")?;
        let fclass = reader.read_u64()?;
        let function_classifications = FunctionClassifications::from_bits(fclass)
            .ok_or_else(|| format!("Invalid GEN8 Function Classifications {fclass:016X}"))?;
        let steam_appid = reader.read_i32()?;
        let debugger_port: u32 = if wad_version >= 14 {
            reader.read_u32()?
        } else {
            0
        };
        let room_order: Vec<GMRef<Room>> = reader.read_simple_list()?;

        let mut general_info = Self {
            debugger_enabled,
            wad_version,
            unknown_value,
            game_file_name,
            config,
            last_object_id,
            last_tile_id,
            game_id,
            directplay_guid: Blob(directplay_guid),
            game_name,
            ide_version,
            window_width,
            window_height,
            flags,
            license_crc32,
            license_md5: Blob(license_md5),
            creation_timestamp,
            display_name,
            function_classifications,
            steam_appid,
            debugger_port,
            room_order,
            gms2_data: None,
        };

        if ide_version.major >= 2 {
            let gms2 = general_info.read_gms2_data(reader)?;
            general_info.gms2_data = Some(gms2);
        }

        Ok(general_info)
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u8(!self.debugger_enabled as u8);
        builder.write_u8(self.wad_version);
        builder.write_u16(self.unknown_value);
        builder.write_gm_string(self.game_file_name)?;
        builder.write_gm_string(self.config)?;
        builder.write_u32(self.last_object_id);
        builder.write_u32(self.last_tile_id);
        builder.write_u32(self.game_id);
        builder.write_bytes(&*self.directplay_guid);
        builder.write_gm_string(self.game_name)?;
        self.ide_version.serialize(builder)?;
        builder.write_u32(self.window_width);
        builder.write_u32(self.window_height);
        builder.write_u32(self.flags.bits());
        builder.write_u32(self.license_crc32);
        builder.write_bytes(&*self.license_md5);
        builder.write_i64(self.creation_timestamp.timestamp());
        builder.write_gm_string(self.display_name)?;
        builder.write_u64(0); // "Active targets"
        builder.write_u64(self.function_classifications.bits());
        builder.write_i32(self.steam_appid);
        if builder.version() >= GMVersion::Wad14 {
            builder.write_u32(self.debugger_port);
        }
        builder.write_simple_list(&self.room_order)?;

        if builder.version() >= GMVersion::GMS2 {
            self.write_gms2_data(builder)?;
        }
        Ok(())
    }
}

impl Default for GeneralInfo {
    fn default() -> Self {
        Self {
            debugger_enabled: false,
            wad_version: 17,
            unknown_value: 0,
            game_file_name: GMRef::none(),
            config: GMRef::none(), // should be "Default"
            last_object_id: 100_000,
            last_tile_id: 10_000_000,
            game_id: 1337,
            directplay_guid: Blob([0u8; 16]),
            game_name: GMRef::none(),
            ide_version: IdeVersion::GMS2,
            window_width: 1337,
            window_height: 1337,
            flags: Flags::empty(),
            license_crc32: 69420,
            license_md5: Blob([69; 16]),
            creation_timestamp: DateTime::default(),
            display_name: GMRef::none(),
            function_classifications: FunctionClassifications::empty(),
            steam_appid: 0,
            debugger_port: 0,
            room_order: vec![],
            gms2_data: Some(GMS2Data::default()),
        }
    }
}
