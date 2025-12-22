use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::embedded_texture::MAGIC_BZ2_QOI_HEADER,
        gm_version::GMVersionReq,
    },
    prelude::*,
};

pub fn check_2022_3(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2022, 3).into()));
    let texture_count = reader.read_u32()?;
    if texture_count < 1 {
        return Ok(None); // Can't detect if there are no texture pages
    }
    if texture_count == 1 {
        reader.cur_pos += 16; // Jump to either padding or length, depending on version
        if reader.read_u32()? > 0 {
            // Check whether it's padding or length
            return target_ver;
        }
    } else {
        let pointer1 = reader.read_u32()?;
        let pointer2 = reader.read_u32()?;
        if pointer1 + 16 == pointer2 {
            return target_ver;
        }
    }

    Ok(None)
}

pub fn check_2022_5(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2022, 5).into()));
    let texture_count = reader.read_u32()?;
    for i in 0..texture_count {
        // Go to each texture, and then to each texture's data
        reader.cur_pos = 4 * i + 4;
        reader.cur_pos = reader.read_u32()? + 12; // Go to texture; at an offset
        reader.cur_pos = reader.read_u32()?; // Go to texture data
        let header: &[u8; 4] = reader.read_bytes_const()?;
        if header != MAGIC_BZ2_QOI_HEADER {
            continue; // Nothing useful, check the next texture
        }
        reader.cur_pos += 4; // Skip width/height
        // Now check actual bz2 headers
        if reader.read_bytes_const::<3>()? != b"BZh" {
            return target_ver;
        }
        reader.cur_pos += 1;
        if *reader.read_bytes_const::<6>()? != [0x31, 0x41, 0x59, 0x26, 0x53, 0x59] {
            // Digits of pi (block header)
            return target_ver;
        }
        return Ok(None); // If first bzip2+qoi texture page version check was unsuccessful, don't bother with other ones
    }

    Ok(None)
}

pub fn check_2_0_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let texture_count = reader.read_u32()?;
    if texture_count < 1 {
        return Ok(None);
    }
    if texture_count == 1 {
        // Go to the first texture pointer (+ minimal texture entry size)
        reader.cur_pos = reader.read_u32()? + 8;
        if reader.read_u32()? == 0 {
            return Ok(None); // If there is a zero instead of texture data pointer; it's not 2.0.6
        }
    }
    if texture_count >= 2 {
        let pointer1 = reader.read_u32()?;
        let pointer2 = reader.read_u32()?;
        if pointer2 - pointer1 == 8 {
            // "Scaled" + "_textureData" -> 8
            return Ok(None);
        }
    }

    Ok(Some((2, 0, 6).into()))
}
