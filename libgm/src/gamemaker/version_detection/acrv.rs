use crate::{
    gamemaker::{deserialize::reader::DataReader, gm_version::GMVersionReq},
    prelude::*,
};

pub fn check_2_3_1(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let count = reader.read_u32()?;
    if count < 1 {
        return Ok(None);
    }

    // Go to the first "point"
    reader.cur_pos = reader.read_u32()? + 8;
    for _ in 0..2 {
        if reader.read_u32()? != 0 {
            // In 2.3 an int with the value of 0 would be set here,
            // it cannot be version 2.3 if this value isn't 0.
            return Ok(Some((2, 3, 1).into()));
        }
    }

    Ok(None)
}
