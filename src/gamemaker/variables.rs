use crate::gamemaker::chunk_reading::{vec_with_capacity, DataReader, GMChunk, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::code::{parse_instance_type, GMInstanceType};

#[derive(Debug, Clone)]
pub struct GMVariables {
    /// List of all variables; mixing global, local and self.
    pub variables: Vec<GMVariable>,
    pub scuffed: Option<GMVariablesScuffed>,
    pub exists: bool,
}
impl GMChunkElement for GMVariables {
    fn empty() -> Self {
        Self { variables: vec![], scuffed: None, exists: false }
    }
}
impl GMElement for GMVariables {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let variables_length: usize = if reader.general_info.bytecode_version >= 15 { 20 } else { 12 };
        let variable_count: usize = reader.get_chunk_length() / variables_length;
        let mut scuffed: Option<GMVariablesScuffed> = None;
        if reader.general_info.bytecode_version >= 15 {
            let globals_count: usize = reader.read_usize()?;    // these variables don't actually represent what they say
            let instances_count: usize = reader.read_usize()?;  // because gamemaker is weird
            let locals_count: usize = reader.read_usize()?;     // TODO: probably needs to be incremented when a variable is added?
            scuffed = Some(GMVariablesScuffed {
                globals_count,
                instances_count,
                locals_count,
            })
        };

        let mut variables: Vec<GMVariable> = Vec::with_capacity(variable_count);
        // let mut occurrence_map: HashMap<usize, GMRef<GMVariable>> = HashMap::with_capacity(variable_count);
        let mut cur_index: u32 = 0;

        while reader.cur_pos + variables_length <= reader.chunk.end_pos {
            let name: GMRef<String> = reader.read_gm_string()?;

            let b15_data: Option<GMVariableB15Data> = if reader.general_info.bytecode_version >= 15 {
                let instance_type: GMInstanceType = parse_instance_type(reader.read_i32()? as i16)
                    .map_err(|e| format!("Could not get instance type for variable \"{}\" while parsing chunk VARI: {e}", reader.display_gm_str(name)))?;
                let variable_id: i32 = reader.read_i32()?;
                Some(GMVariableB15Data { instance_type, variable_id })
            } else { None };

            let occurrences_count: i32 = reader.read_i32()?;
            let occurrences_count: usize = if occurrences_count < 0 { 0 } else { occurrences_count as usize };
            let first_occurrence_address: i32 = reader.read_i32()?;

            let (occurrences, name_string_id): (Vec<usize>, i32) = parse_occurrence_chain(reader, first_occurrence_address, occurrences_count)?;

            for occurrence in occurrences {
                if let Some(old_value) = reader.variable_occurrence_map.insert(occurrence, GMRef::new(cur_index)) {
                    return Err(format!(
                        "Conflicting occurrence positions while parsing variables: absolute position {} \
                        was already set for {}variable #{} with name \"{}\"; trying to set to variable #{} with name \"{}\"",
                        occurrence,
                        b15_data.map_or_else(|| "".to_string(), |i| format!("{} ", i.instance_type)),
                        old_value.index,
                        reader.display_gm_str(old_value.resolve(&variables)?.name),
                        cur_index,
                        reader.display_gm_str(name),
                    ))
                }
            }

            variables.push(GMVariable {
                name,
                b15_data,
                name_string_id,
            });
            cur_index += 1;
        }

        Ok(GMVariables { variables, scuffed, exists: true })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub b15_data: Option<GMVariableB15Data>,
    pub name_string_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariableB15Data {
    pub instance_type: GMInstanceType,
    pub variable_id: i32,
}

#[derive(Debug, Clone)]
pub struct GMVariablesScuffed {
    pub globals_count: usize,
    pub instances_count: usize,
    pub locals_count: usize,
}


pub fn parse_occurrence_chain(reader: &mut DataReader, first_occurrence_pos: i32, occurrence_count: usize) -> Result<(Vec<usize>, i32), String> {
    if occurrence_count < 1 {
        return Ok((vec![], first_occurrence_pos));
    }

    let saved_chunk: GMChunk = reader.chunk.clone();
    let saved_position: usize = reader.cur_pos;
    reader.chunk = reader.chunks.get("CODE").cloned().ok_or("Chunk CODE not set while parsing variable occurrences")?;
    
    let mut occurrence_pos: usize = first_occurrence_pos as usize + 4;
    let mut occurrences: Vec<usize> = vec_with_capacity(occurrence_count)?;
    let mut offset: i32 = first_occurrence_pos;

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        reader.cur_pos = occurrence_pos;
        let raw_value: i32 = reader.read_i32()?;
        offset = raw_value & 0x07FFFFFF;
        if offset < 1 {
            return Err(format!(
                "Next occurrence offset is {0} (0x{0:08X}) which is negative while parsing \
                variable occurrences at position {1} (raw value is 0x{2:08X})",
                offset, reader.cur_pos-4, raw_value,
            ))
        }
        occurrence_pos = offset as usize;   // might overflow on last occurrence (name string id) but doesn't matter
    }

    let name_string_id: i32 = offset & 0xFFFFFF;
    reader.chunk = saved_chunk;
    reader.cur_pos = saved_position;
    Ok((occurrences, name_string_id))
}

