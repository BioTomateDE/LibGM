use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        gm_version::{GMVersionReq, LTSBranch::PostLTS},
    },
    prelude::*,
};

pub fn check_2022_9(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    if reader.general_info.is_version_at_least((2023, 1, PostLTS)) {
        return Ok(None);
    }

    reader.read_gms2_chunk_version("TGIN Version")?;
    let tgin_count = reader.read_u32()?;
    if tgin_count < 1 {
        return Ok(None);
    }
    let pointer1 = reader.read_u32()?;
    let pointer2 = if tgin_count >= 2 {
        reader.read_u32()?
    } else {
        reader.chunk.end_pos
    };
    reader.cur_pos = pointer1 + 4;

    // Check to see if the pointer located at this address points within this object
    // If not, then we know we're using a new format!
    let ptr = reader.read_u32()?;
    if ptr < pointer1 || ptr >= pointer2 {
        return Ok(Some((2022, 9).into()));
    }

    Ok(None)
}

pub fn check_2023_1(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    if reader.general_info.is_version_at_least((2023, 1, PostLTS)) {
        return Ok(None);
    }

    reader.read_gms2_chunk_version("TGIN Version")?;

    let tgin_count = reader.read_u32()?;
    if tgin_count < 1 {
        return Ok(None);
    }
    let pointer1 = reader.read_u32()?;

    // Go to the 4th list pointer of the first TGIN entry.
    // (either to "Fonts" or "SpineTextures" depending on the version)
    reader.cur_pos = pointer1 + 16 + 4 * 3;
    let pointer4 = reader.read_u32()?;

    // If there's a "TexturePages" count instead of the 5th list pointer.
    // The count can't be greater than the pointer.
    // (the list could be either "Tilesets" or "Fonts").
    if reader.read_u32()? <= pointer4 {
        return Ok(Some((2023, 1, PostLTS).into()));
    }

    Ok(None)
}
