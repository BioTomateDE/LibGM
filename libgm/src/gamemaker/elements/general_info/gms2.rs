use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::general_info::GMGeneralInfo,
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::rng::DotnetRng,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GMS2Data {
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

impl GMGeneralInfo {
    /// Parse and verify UID
    pub(super) fn read_gms2_data(&self, reader: &mut DataReader) -> Result<GMS2Data> {
        let timestamp: i64 = self.timestamp_created.timestamp();
        let mut info_timestamp_offset: bool = true;
        let seed: i32 = (timestamp & 0xFFFF_FFFF) as i32;
        let mut rng = DotnetRng::new(seed);

        let first_expected: i64 = (i64::from(rng.next()) << 32) | i64::from(rng.next());
        let first_actual = reader.read_i64()?;
        if first_actual != first_expected {
            bail!("Unexpected random UID #1: expected {first_expected}; got {first_actual}");
        }

        let info_location: i32 = self.get_info_location(timestamp);
        let mut random_uid = [0_i64; 4];

        for i in 0i32..4 {
            if i == info_location {
                let curr = reader.read_i64()?;
                random_uid[i as usize] = curr;

                if curr != self.get_info_number(first_expected, true) {
                    if curr != self.get_info_number(first_expected, false) {
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

                random_uid[i as usize] = (i64::from(second_actual) << 32) | i64::from(third_actual);
            }
        }
        let fps = reader.read_f32()?;
        let allow_statistics = reader.read_bool32()?;
        let game_guid: [u8; 16] = reader
            .read_bytes_const::<16>()
            .cloned()
            .context("reading Game GUID")?;

        Ok(GMS2Data {
            random_uid,
            fps,
            allow_statistics,
            game_guid,
            info_timestamp_offset,
        })
    }

    /// Write UID
    pub(super) fn write_gms2_data(&self, builder: &mut DataBuilder) -> Result<()> {
        let gms2_info: &GMS2Data = self
            .gms2_data
            .as_ref()
            .ok_or("GMS2 Data not set in General Info")?;
        let timestamp: i64 = self.timestamp_created.timestamp();
        let seed: i32 = (timestamp & 0xFFFF_FFFF) as i32;
        let mut rng = DotnetRng::new(seed);
        let first_random: i64 = (i64::from(rng.next()) << 32) | i64::from(rng.next());
        let info_number = self.get_info_number(first_random, gms2_info.info_timestamp_offset);
        let info_location: i32 = self.get_info_location(timestamp);
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
        Ok(())
    }

    const fn get_info_location(&self, timestamp: i64) -> i32 {
        ((timestamp & 0xFFFF) as i32 / 7
            + self.game_id.wrapping_sub(self.default_window_width) as i32
            + self.room_order.len() as i32)
            .abs()
            % 4
    }

    fn get_info_number(&self, first_random: i64, info_timestamp_offset: bool) -> i64 {
        let flags_raw: u32 = self.flags.build();
        let mut info_number: i64 = self.timestamp_created.timestamp();
        if info_timestamp_offset {
            info_number -= 1000;
        }
        info_number = Self::uid_bitmush(info_number);
        info_number ^= first_random;
        info_number = !info_number;
        info_number ^= (i64::from(self.game_id) << 32) | i64::from(self.game_id);
        info_number ^= (i64::from(self.default_window_width) + i64::from(flags_raw)) << 48
            | (i64::from(self.default_window_height) + i64::from(flags_raw)) << 32
            | (i64::from(self.default_window_height) + i64::from(flags_raw)) << 16
            | (i64::from(self.default_window_width) + i64::from(flags_raw));
        info_number ^= i64::from(self.wad_version);
        info_number
    }

    const fn uid_bitmush(info_number: i64) -> i64 {
        let mut temp: u64 = info_number as u64;
        temp = (temp << 56 & 0xFF00_0000_0000_0000)
            | (temp >> 8 & 0x00FF_0000_0000_0000)
            | (temp << 32 & 0x0000_FF00_0000_0000)
            | (temp >> 16 & 0x0000_00FF_0000_0000)
            | (temp << 8 & 0x0000_0000_FF00_0000)
            | (temp >> 24 & 0x0000_0000_00FF_0000)
            | (temp >> 16 & 0x0000_0000_0000_FF00)
            | (temp >> 32 & 0x0000_0000_0000_00FF);
        temp as i64
    }
}
