// SPDX-License-Identifier: GPL-3.0-only
use super::target_version;
use crate::prelude::*;
use crate::wad::parse::reader::DataReader;
use crate::wad::version::GMVersion;

pub fn check_2023_x(reader: &mut DataReader) -> Result<Option<GMVersion>> {
    let mut ver = Ok(None);
    reader.align(4)?;
    reader.read_gms2_chunk_version("PSEM Version")?;
    let count = reader.read_u32()?;
    if count < 11 {
        // 2023.2 automatically adds eleven, later versions don't
        ver = target_version!(GM2023_4);
    }
    if count == 0 {
        return ver; // Nothing more to detect
    }
    // TODO: this used to be 2023.6
    if count == 1 {
        match reader.chunk.end_pos - reader.chunk.start_pos {
            248 => ver = target_version!(GM2023_8),
            216 => ver = target_version!(Lts2022),
            200 => ver = target_version!(GM2023_4),
            elem_size => bail!("Unrecognized PSEM size {elem_size} with only one element"),
        }
    } else {
        let pointer1 = reader.read_u32()?;
        let pointer2 = reader.read_u32()?;
        match pointer2 - pointer1 {
            236 => ver = target_version!(GM2023_8),
            192 => ver = target_version!(Lts2022),
            188 => ver = target_version!(GM2023_4),
            176 => {} // 2023.2
            elem_size => bail!("Unrecognized PSEM element size {elem_size} with {count} elements"),
        }
    }
    ver
}
