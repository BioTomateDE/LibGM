use crate::{
    gamemaker::{deserialize::reader::DataReader, version::GMVersionReq},
    gml::{instruction::DataType, opcodes},
    prelude::*,
    util::init::vec_with_capacity,
};

pub fn check_2023_8_and_2024_4(reader: &mut DataReader) -> Result<Option<GMVersionReq>> {
    fn get_chunk_elem_count(
        reader: &mut DataReader,
        chunk_name: &'static str,
        gms2: bool,
    ) -> Result<u32> {
        let Some(chunk) = reader.chunks.get(chunk_name) else {
            return Ok(0);
        };

        reader.chunk = chunk.clone();
        reader.cur_pos = chunk.start_pos;
        if gms2 {
            reader.align(4)?;
            reader.read_u32()?; // GMS2 chunk version (always 1)
        }
        let count = reader.read_u32()?;
        Ok(count)
    }

    let chunk_code = reader.chunk.clone();
    let background_count = get_chunk_elem_count(reader, "BGND", false)?;
    let path_count = get_chunk_elem_count(reader, "PATH", false)?;
    let script_count = get_chunk_elem_count(reader, "SCPT", false)?;
    let font_count = get_chunk_elem_count(reader, "FONT", false)?;
    let timeline_count = get_chunk_elem_count(reader, "TMLN", false)?;
    let shader_count = get_chunk_elem_count(reader, "SHDR", false)?;
    let sequence_count = get_chunk_elem_count(reader, "SEQN", true)?;
    let particle_system_count = get_chunk_elem_count(reader, "SEQN", true)?;

    let is_asset_type_2024_4 = |int_argument: u32| -> bool {
        let resource_id = int_argument & 0xFF_FFFF;
        let resource_type = (int_argument >> 24) as u8;
        match resource_type {
            4 => resource_id >= background_count,
            5 => resource_id >= path_count,
            6 => resource_id >= script_count,
            7 => resource_id >= font_count,
            8 => resource_id >= timeline_count,
            9 => true, // Used to be unused, now are sequences
            10 => resource_id >= shader_count,
            11 => resource_id >= sequence_count,
            // case 12 used to be animcurves, but now is unused (so would actually mean earlier than 2024.4)
            13 => resource_id >= particle_system_count,
            _ => false,
        }
    };

    reader.cur_pos = chunk_code.start_pos;
    reader.chunk = chunk_code;

    let code_count = reader.read_u32()?;
    let mut code_pointers = vec_with_capacity(code_count)?;
    for _ in 0..code_count {
        let ptr = reader.read_u32()?;
        if ptr != 0 {
            code_pointers.push(ptr);
        }
    }
    let mut detected_2023_8: bool = false;

    for code_ptr in code_pointers {
        reader.cur_pos = code_ptr + 4; // Skip name
        let instructions_length = reader.read_u32()?;
        reader.cur_pos += 4; // Skip locals and arguments count
        let instructions_start_relative = reader.read_i32()?;
        let instructions_start = (reader.cur_pos as i32 - 4 + instructions_start_relative) as u32;
        let instructions_end = instructions_start + instructions_length;
        reader.cur_pos = instructions_start;

        while reader.cur_pos < instructions_end {
            let word = reader.read_u32()?;
            let opcode = (word >> 24) as u8;
            let type1 = ((word & 0x00FF_0000) >> 16) as u8;

            if matches!(opcode, opcodes::POP | opcodes::CALL) {
                reader.cur_pos += 4;
            }

            if matches!(opcode, 0xC0..0xC4) {
                // Push variants; account for int16
                if type1 != DataType::Int16.into() {
                    reader.cur_pos += 4;
                }
                continue;
            }

            if opcode != opcodes::EXTENDED {
                continue;
            }

            if type1 == DataType::Int32.into() {
                let int_argument = reader.read_u32()?;
                if is_asset_type_2024_4(int_argument) {
                    // Return immediately if highest detectable version (2024.4) is found
                    return Ok(Some((2024, 4).into()));
                }
                detected_2023_8 = true;
            }
        }
    }

    if detected_2023_8 {
        Ok(Some((2023, 8).into()))
    } else {
        Ok(None)
    }
}
