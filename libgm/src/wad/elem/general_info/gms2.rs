// SPDX-License-Identifier: GPL-3.0-only
use dotnet_rng::DotnetRng;

use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::elem::general_info::GeneralInfo;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct GMS2Data {
    /// Anti-piracy checksum.
    pub checksum: Blob<[u64; 4]>,

    /// Whether the game was compiled as a release build ("Create Executable")
    /// rather than a test build ("Run" or "Debug").
    ///
    /// This field is derived from the checksum and not directly stored in the data file.
    ///
    /// If this is `false`, then the runner (at least old runners) close the
    /// game after 1 hour (DOCME: is this correct) as an anti-piracy measure.
    pub release_build: bool,

    /// How many frames per second this game runs as.
    ///
    /// Can be adjusted in code using the
    /// [`game_set_speed`](https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/General_Game_Control/game_set_speed.htm)
    /// function.
    pub fps: f32,

    /// If enabled, the game runner may send requests to a GameMaker player
    /// count statistics server or straight to Israel.
    pub allow_statistics: bool,

    /// DOCME: Unknown, some sort of checksum.
    pub game_guid: Blob<[u8; 16]>,
}

impl Default for GMS2Data {
    fn default() -> Self {
        Self {
            checksum: Blob([69; 4]),
            fps: 60.0,
            allow_statistics: false,
            game_guid: Blob([69; 16]),
            release_build: true,
        }
    }
}

impl GeneralInfo {
    pub(super) fn read_gms2_data(&self, reader: &mut DataReader) -> Result<GMS2Data> {
        let mut release_build: bool = true;
        let seed: i32 = self.creation_timestamp as i32;
        let mut rng = DotnetRng::new(seed);

        let first_expected: u64 = ((rng.next() as u64) << 32) | rng.next() as u64;
        let first_actual = reader.read_u64()?;
        if first_actual != first_expected {
            bail!("Unexpected checksum part 1: expected {first_expected}; got {first_actual}");
        }

        let info_location: i32 = self.get_info_location(self.creation_timestamp);
        let mut checksum = [0u64; 4];

        for i in 0..4 {
            if i == info_location {
                let curr = reader.read_u64()?;
                checksum[i as usize] = curr;

                if curr != self.get_info_number(first_expected, true) {
                    let expected = self.get_info_number(first_expected, false);
                    if curr != expected {
                        bail!("Unexpected checksum info: expected {expected}, got {curr}");
                    }
                    log::debug!("Game originates from a test build");
                    release_build = false;
                }
            } else {
                let actual2 = reader.read_u32()?;
                let actual3 = reader.read_u32()?;
                let expected2: u32 = rng.next() as u32;
                let expected3: u32 = rng.next() as u32;
                if actual2 != expected2 {
                    bail!("Unexpected checksum part 2: expected {expected2}, got {actual2}");
                }
                if actual3 != expected3 {
                    bail!("Unexpected random UID #3: expected {expected3}, got {actual3}");
                }

                checksum[i as usize] = (actual2 as u64) << 32 | actual3 as u64;
            }
        }

        let fps = reader.read_f32()?;
        let allow_statistics = reader.read_bool32()?;
        let game_guid: [u8; 16] = *reader.read_bytes_const().ctx("reading Game GUID")?;

        Ok(GMS2Data {
            checksum: Blob(checksum),
            fps,
            allow_statistics,
            game_guid: Blob(game_guid),
            release_build,
        })
    }

    pub(super) fn write_gms2_data(&self, writer: &mut DataWriter) -> Result<()> {
        let gms2_info: &GMS2Data =
            self.gms2_data.as_ref().ok_or("GMS2 Data not set in General Info")?;

        let seed: i32 = self.creation_timestamp as i32;
        let mut rng = DotnetRng::new(seed);
        let first_random: u64 = ((rng.next() as u64) << 32) | rng.next() as u64;
        let info_number = self.get_info_number(first_random, gms2_info.release_build);
        let info_location: i32 = self.get_info_location(self.creation_timestamp);
        writer.write_u64(first_random);
        for i in 0..4 {
            if i == info_location {
                writer.write_u64(info_number);
            } else {
                let first = rng.next();
                let second = rng.next();
                writer.write_i32(first);
                writer.write_i32(second);
            }
        }

        writer.write_f32(gms2_info.fps);
        writer.write_bool32(gms2_info.allow_statistics);
        writer.write_bytes(&*gms2_info.game_guid);
        Ok(())
    }

    // random bullshit go!
    const fn get_info_location(&self, timestamp: u64) -> i32 {
        let t = (timestamp as u16 / 7) as i32;
        let g = self.game_id.wrapping_sub(self.window_width) as i32;
        let r = self.room_order.len() as i32;
        (t + g + r).abs() % 4
    }

    const fn get_info_number(&self, first_random: u64, offset: bool) -> u64 {
        let flags = self.flags.bits() as u64;
        let gid = self.game_id as u64;
        let ww = self.window_width as u64;
        let wh = self.window_height as u64;

        let mut info_number: u64 = self.creation_timestamp;
        if offset {
            info_number = (info_number as i64 - 1000) as u64;
        }
        info_number = Self::uid_bitmush(info_number);
        info_number ^= first_random;
        info_number = !info_number;
        info_number ^= (gid << 32) | gid;
        info_number ^= (ww + flags) << 48 | (wh + flags) << 32 | (wh + flags) << 16 | (ww + flags);
        info_number ^= self.wad_version as u64;
        info_number
    }

    const fn uid_bitmush(num: u64) -> u64 {
        #[expect(clippy::needless_late_init)]
        let new; // nice alignment
        new = (0xFF00_0000_0000_0000 & num << 56)
            | (0x00FF_0000_0000_0000 & num >> 8)
            | (0x0000_FF00_0000_0000 & num << 32)
            | (0x0000_00FF_0000_0000 & num >> 16)
            | (0x0000_0000_FF00_0000 & num << 8)
            | (0x0000_0000_00FF_0000 & num >> 24)
            | (0x0000_0000_0000_FF00 & num >> 16)
            | (0x0000_0000_0000_00FF & num >> 32);
        new
    }
}
