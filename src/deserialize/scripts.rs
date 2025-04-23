use std::collections::HashMap;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::strings::{GMStringRef, GMStrings};

#[derive(Debug, Clone)]
pub struct GMScript {
    pub name: GMStringRef,
    pub id: Option<u32>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GMScriptRef {
    pub index: usize,
}
impl GMScriptRef {
    pub fn resolve<'a>(&self, scripts: &'a GMScripts) -> Result<&'a GMScript, String> {
        match scripts.scripts_by_index.get(self.index) {
            Some(script) => Ok(script),
            None => Err(format!(
                "Could not resolve script with index {} in list with length {}.",
                self.index, scripts.scripts_by_index.len(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GMScripts {
    pub scripts_by_index: Vec<GMScript>,
    pub abs_pos_to_index: HashMap<usize, usize>,
}
impl GMScripts {
    pub fn get_script_by_index(&self, index: usize) -> Option<GMScriptRef> {
        if index >= self.scripts_by_index.len() {
            return None;
        }
        Some(GMScriptRef {index})
    }
    pub fn len(&self) -> usize {
        self.scripts_by_index.len()
    }
}


pub fn parse_chunk_scpt(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMScripts, String> {
    chunk.file_index = 0;
    let script_count: usize = chunk.read_usize()?;

    let mut absolute_start_positions: Vec<usize> = Vec::with_capacity(script_count);
    for _ in 0..script_count {
        absolute_start_positions.push(chunk.read_usize()?);
    }

    let mut abs_pos_to_index: HashMap<usize, usize> = HashMap::new();
    let mut scripts_by_index: Vec<GMScript> = Vec::with_capacity(script_count);
    for (i, abs_start_position) in absolute_start_positions.iter().enumerate() {
        chunk.file_index = abs_start_position - chunk.abs_pos;
        let name: GMStringRef = chunk.read_gm_string(&strings)?;
        let id: i32 = chunk.read_i32()?;
        if id < -1 {
            return Err(format!("Script with name {} has ID less than -1: {}", name.resolve(strings)?, id))
        }
        let id: Option<u32> = if id == -1 { None } else { Some(id as u32) };

        // println!("Script  {:<10?} {}", id, name.resolve(strings)?);
        scripts_by_index.push(GMScript { name, id });
        abs_pos_to_index.insert(*abs_start_position, i);
    }


    Ok(GMScripts { scripts_by_index, abs_pos_to_index })
}

