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
use crate::wad::chunk::ChunkName;
use crate::wad::elem::GMElement;
use crate::wad::elem::room::Room;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;
use crate::wad::version::GMVersion;
use crate::wad::version::IdeVersion;

#[derive(Debug, Clone, PartialEq)]
pub struct GeneralInfo {
    /// Indicates whether debugging support via an external GameMaker debugger is enabled.
    /// The game will hang if this is enabled and there is no debugger.
    /// (It prints "Waiting for debugger to connect...")
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
    /// This value is probably ignored by the runner.
    ///
    /// See [`GMVersion`] for the real format version.
    pub wad_version: u8,

    /// The file name of the runner (on Windows).
    ///
    /// This is probably ignored by the runner.
    pub runner_file_name: GMRef<String>,

    /// Which GameMaker configuration the data file was compiled with.
    ///
    /// This is probably ignored by the runner.
    pub config: GMRef<String>,

    /// The last game object instance ID of the data file.
    ///
    /// It's important to increment this when a new game object instance is added.
    /// Otherwise, instances dynamically created at runtime (using `instance_create`)
    /// will have the same IDs as some statically placed instances in rooms.
    /// Instance IDs becoming non-unique leads to lots of weird behavior.
    ///
    /// This number is usually above 100000, since that's the offset instance IDs start at.
    /// If it is lower, then the runner can confuse it for a game object asset ID,
    /// since it has no way to tell the difference.
    pub last_instance_id: u32,

    /// The last tile ID of the data file.
    ///
    /// See above.
    /// Honestly nobody cares about tiles so this is probably fine to set to 69420.
    pub last_tile_id: u32,

    /// The game id of the data file, whatever that may mean.
    ///
    /// This is visible in the GameMaker IDE. You can press a button to re-generate it.
    pub game_id: u32,

    /// The `DirectPlay` GUID of the data file.
    ///
    /// This is always empty in GameMaker Studio.
    pub directplay_guid: Blob<[u8; 16]>,

    /// The name of the game.
    ///
    /// This is the name of the directory game save data gets written to.
    pub game_name: GMRef<String>,

    /// The GameMaker version of the IDE this game was created in.
    ///
    /// This version struct is not updated by YoYo Games since GMS 2 and is
    /// is stuck on `2.0.0.0` since GameMaker Studio: 2 (released November 2016).
    /// If you need the format version of the data file, check out [`GMVersion`].
    ///
    /// This value is probably ignored by the runner.
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
        let word = reader.read_u32()?;
        let debugger_enabled: bool = match word & 0xFF {
            1 => false,
            0 => true,
            other => bail!("Invalid 'Is Debugger Disabled' u8 bool {other}"),
        };
        let wad_version: u8 = (word >> 8) as u8;
        reader.assert_int(word >> 16, 0, "Remaining bytes in first GEN8 word")?;

        let runner_file_name: GMRef<String> = reader.read_gm_string()?;
        let config: GMRef<String> = reader.read_gm_string()?;
        let last_instance_id = reader.read_u32()?;
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
        let debugger_port: u32 =
            if reader.version >= GMVersion::Wad14 { reader.read_u32()? } else { 0 };
        let room_order: Vec<GMRef<Room>> = reader.read_simple_list()?;

        let mut general_info = Self {
            debugger_enabled,
            wad_version,
            runner_file_name,
            config,
            last_instance_id,
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

        if reader.version >= GMVersion::GMS2 {
            let gms2 = general_info.read_gms2_data(reader)?;
            general_info.gms2_data = Some(gms2);
        }

        Ok(general_info)
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        let word = (!self.debugger_enabled as u32) | (self.wad_version as u32) << 8;
        writer.write_u32(word);
        writer.write_gm_string(self.runner_file_name)?;
        writer.write_gm_string(self.config)?;
        writer.write_u32(self.last_instance_id);
        writer.write_u32(self.last_tile_id);
        writer.write_u32(self.game_id);
        writer.write_bytes(&*self.directplay_guid);
        writer.write_gm_string(self.game_name)?;
        writer.write_elem(&self.ide_version)?;
        writer.write_u32(self.window_width);
        writer.write_u32(self.window_height);
        writer.write_u32(self.flags.bits());
        writer.write_u32(self.license_crc32);
        writer.write_bytes(&*self.license_md5);
        writer.write_i64(self.creation_timestamp.timestamp());
        writer.write_gm_string(self.display_name)?;
        writer.write_u64(0); // "Active targets"
        writer.write_u64(self.function_classifications.bits());
        writer.write_i32(self.steam_appid);
        if writer.version() >= GMVersion::Wad14 {
            writer.write_u32(self.debugger_port);
        }
        writer.write_simple_list(&self.room_order)?;

        if writer.version() >= GMVersion::GMS2 {
            self.write_gms2_data(writer)?;
        }
        Ok(())
    }
}

impl Default for GeneralInfo {
    fn default() -> Self {
        Self {
            debugger_enabled: false,
            wad_version: 17,
            runner_file_name: GMRef::none(),
            config: GMRef::none(), // should be "Default"
            last_instance_id: 100_000,
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
