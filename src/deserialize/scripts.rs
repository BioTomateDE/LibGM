use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::GMCode;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMScript {
    pub name: GMRef<String>,
    pub is_constructor: bool,
    pub code: Option<GMRef<GMCode>>,
}


#[derive(Debug, Clone)]
pub struct GMScripts {
    pub scripts_by_index: Vec<GMScript>,
    pub abs_pos_to_index: HashMap<usize, usize>,
}

pub fn parse_chunk_scpt(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMScripts, String> {
    chunk.cur_pos = 0;
    let script_count: usize = chunk.read_usize_count()?;

    let mut abs_start_positions: Vec<usize> = Vec::with_capacity(script_count);
    for _ in 0..script_count {
        abs_start_positions.push(chunk.read_usize_pos()?);
    }

    let mut scripts_by_index: Vec<GMScript> = Vec::with_capacity(script_count);
    let mut abs_pos_to_index: HashMap<usize, usize> = HashMap::with_capacity(script_count);

    for (i, abs_start_position) in abs_start_positions.iter().enumerate() {
        chunk.cur_pos = *abs_start_position - chunk.abs_pos;
        let name: GMRef<String> = chunk.read_gm_string(&strings)?;

        let mut code_id: i32 = chunk.read_i32()?;
        let is_constructor: bool = if code_id < -1 {
            code_id = (code_id as u32 & 0x7FFFFFFF) as i32;
            true
        } else {
            false
        };

        let code: Option<GMRef<GMCode>> = if code_id == -1 {
            None
        } else {
            let code_id: usize = usize::try_from(code_id).map_err(|e| format!(
                "Could not convert Code ID {code_id} (0x{code_id:08X}) to usize for Script \"{}\": {e}", name.display(strings),
            ))?;
            Some(GMRef::new(code_id))
        };

        scripts_by_index.push(GMScript { name, is_constructor, code });
        abs_pos_to_index.insert(*abs_start_position, i);
    }


    Ok(GMScripts { scripts_by_index, abs_pos_to_index })
}

