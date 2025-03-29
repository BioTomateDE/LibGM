use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::strings::{UTStringRef, UTStrings};

#[derive(Debug, Clone)]
pub struct UTScript<'a> {
    pub script_id: u32,
    pub name: UTStringRef<'a>,
}

pub fn parse_chunk_SCPT<'a>(chunk: &mut UTChunk, strings: &'a UTStrings) -> Result<Vec<UTScript<'a>>, String> {
    chunk.file_index = 0;
    let scripts_length: usize = chunk.read_usize()?;

    let mut script_ids: Vec<u32> = Vec::with_capacity(scripts_length);
    for _ in 0..scripts_length {
        let script_id: u32 = chunk.read_u32()?;
        script_ids.push(script_id);
    }

    let mut script_names: Vec<UTStringRef> = Vec::with_capacity(scripts_length);
    for _ in 0..scripts_length {
        let script_name: UTStringRef = chunk.read_ut_string(&strings)?;
        script_names.push(script_name);
        chunk.read_u32()?;   // skip counter going up from zero (redundant)
    }

    let mut scripts: Vec<UTScript> = Vec::with_capacity(scripts_length);
    for i in 0..scripts_length {
        // println!("{} {}", script_ids[i], script_names[i]);
        let script_id: u32 = script_ids[i];
        let name: UTStringRef = script_names[i].clone();
        let script: UTScript = UTScript { script_id, name };
        scripts.push(script);
    }

    Ok(scripts)
}

