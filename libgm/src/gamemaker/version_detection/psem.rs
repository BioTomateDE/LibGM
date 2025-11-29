use crate::{
    gamemaker::{deserialize::reader::DataReader, gm_version::GMVersionReq},
    prelude::*,
    util::assert::assert_int,
};

pub fn check_2023_x(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let mut target_ver = None;
    reader.align(4)?;
    assert_int("PSEM Version", 1, reader.read_u32()?)?;
    let count = reader.read_u32()?;
    if count < 11 {
        // 2023.2 automatically adds eleven, later versions don't
        target_ver = Some((2023, 4).into());
    }
    if count == 0 {
        return Ok(target_ver); // Nothing more to detect
    }
    if count == 1 {
        match reader.chunk.end_pos - reader.chunk.start_pos {
            0xF8 => target_ver = Some((2023, 8).into()),
            0xD8 => target_ver = Some((2023, 6).into()),
            0xC8 => target_ver = Some((2023, 4).into()),
            elem_size => bail!("Unrecognized PSEM size {elem_size} with only one element"),
        }
    } else {
        let pointer1 = reader.read_u32()?;
        let pointer2 = reader.read_u32()?;
        match pointer2 - pointer1 {
            0xEC => target_ver = Some((2023, 8).into()),
            0xC0 => target_ver = Some((2023, 6).into()),
            0xBC => target_ver = Some((2023, 4).into()),
            0xB0 => {}, // 2023.2
            elem_size => bail!("Unrecognized PSEM size {elem_size} with {count} elements"),
        }
    }
    Ok(target_ver)
}
