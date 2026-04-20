use super::target_version;
use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::version::GMVersion;

pub fn check_2023_x(reader: &mut DataReader) -> Result<Option<GMVersion>> {
    let mut ver = Ok(None);
    reader.align(4)?;
    reader.read_gms2_chunk_version("PSEM Version")?;
    let count = reader.read_u32()?;
    if count < 11 {
        // 2023.2 automatically adds eleven, later versions don't
        ver = target_version!(2023, 4);
    }
    if count == 0 {
        return ver; // Nothing more to detect
    }
    if count == 1 {
        match reader.chunk.end_pos - reader.chunk.start_pos {
            0xF8 => ver = target_version!(2023, 8),
            0xD8 => ver = target_version!(2023, 6),
            0xC8 => ver = target_version!(2023, 4),
            elem_size => bail!("Unrecognized PSEM size {elem_size} with only one element"),
        }
    } else {
        let pointer1 = reader.read_u32()?;
        let pointer2 = reader.read_u32()?;
        match pointer2 - pointer1 {
            0xEC => ver = target_version!(2023, 8),
            0xC0 => ver = target_version!(2023, 6),
            0xBC => ver = target_version!(2023, 4),
            0xB0 => {} // 2023.2
            elem_size => bail!("Unrecognized PSEM size {elem_size} with {count} elements"),
        }
    }
    ver
}
