use std::collections::HashMap;
use crate::chunk_reading::UTChunk;

pub struct UTScript {
    pub script_id: u32,
    pub name: String,
}

pub fn parse_chunk_SCPT(mut chunk: UTChunk, strings: &HashMap<u32, String>) -> Result<Vec<UTScript>, String> {
    let scripts_length: usize = chunk.read_usize()?;

    let mut script_ids: Vec<u32> = Vec::with_capacity(scripts_length);
    for _ in 0..scripts_length {
        let script_id: u32 = chunk.read_u32()?;
        script_ids.push(script_id);
    }

    let mut script_names: Vec<String> = Vec::with_capacity(scripts_length);
    for _ in 0..scripts_length {
        let script_name: String = chunk.read_ut_string(&strings)?;
        script_names.push(script_name);
        chunk.read_u32()?;   // skip counter going up from zero (redundant)
    }

    let mut scripts: Vec<UTScript> = Vec::with_capacity(scripts_length);
    for i in 0..scripts_length {
        let script_id: u32 = script_ids[i];
        let name: String = script_names[i].clone();
        let script: UTScript = UTScript { script_id, name };
        scripts.push(script);
    }

    Ok(scripts)
}

