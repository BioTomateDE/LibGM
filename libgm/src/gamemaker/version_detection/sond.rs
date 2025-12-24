use crate::{
    gamemaker::{deserialize::reader::DataReader, version::GMVersionReq},
    prelude::*,
};

pub fn check_2024_6(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    let target_ver = Ok(Some((2024, 6).into()));
    let possible_sound_count = reader.read_u32()?;
    let mut sound_pointers: Vec<u32> = Vec::with_capacity(2);

    for _ in 0..possible_sound_count {
        let pointer = reader.read_u32()?;
        if pointer == 0 {
            continue;
        }
        sound_pointers.push(pointer);
        if sound_pointers.len() >= 2 {
            break;
        }
    }

    if sound_pointers.len() >= 2 {
        // If first sound's theoretical (old) end offset is below the start offset of
        // The next sound by exactly 4 bytes, then this is 2024.6.
        if sound_pointers[0] + 4 * 9 == sound_pointers[1] - 4 {
            return target_ver;
        } else if sound_pointers.len() == 1 {
            // If there's a nonzero value where padding should be at the
            // End of the sound, then this is 2024.6.
            let abs_pos: u32 = sound_pointers[0] + 4 * 9;
            if abs_pos % 16 != 4 {
                bail!("Expected to be on specific alignment at this point");
            }
            reader.cur_pos = abs_pos;
            if reader.read_u32()? != 0 {
                return target_ver;
            }
        }
    }
    Ok(None)
}
