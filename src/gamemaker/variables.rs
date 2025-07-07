use crate::gm_deserialize::{DataReader, GMChunk, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::code::{build_instance_type, parse_instance_type, GMInstanceType};
use crate::gm_serialize::{DataBuilder, GMSerializeIfVersion};
use crate::utility::vec_with_capacity;

#[derive(Debug, Clone)]
pub struct GMVariables {
    /// List of all variables; mixing global, local and self.
    pub variables: Vec<GMVariable>,
    pub scuffed: Option<GMVariablesScuffed>,
    pub yyc: bool,
    pub exists: bool,
}
impl GMChunkElement for GMVariables {
    fn empty() -> Self {
        Self { variables: vec![], scuffed: None, yyc: false, exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMVariables {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        if reader.get_chunk_length() == 0 {
            return Ok(Self { variables: vec![], scuffed: None, yyc: true, exists: true })
        }
        let variables_length: usize = if reader.general_info.bytecode_version >= 15 { 20 } else { 12 };
        let variable_count: usize = reader.get_chunk_length() / variables_length;
        let scuffed: Option<GMVariablesScuffed> = reader.deserialize_if_bytecode_version(15)?;

        let mut variables: Vec<GMVariable> = Vec::with_capacity(variable_count);
        let mut cur_index: u32 = 0;

        while reader.cur_pos + variables_length <= reader.chunk.end_pos {
            let name: GMRef<String> = reader.read_gm_string()?;

            let b15_data: Option<GMVariableB15Data> = reader.deserialize_if_bytecode_version(15)?;

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

            variables.push(GMVariable { name, b15_data, name_string_id });
            cur_index += 1;
        }

        Ok(GMVariables { variables, scuffed, yyc: false, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists || self.yyc { return Ok(()) }
        self.scuffed.serialize_if_bytecode_ver(builder, "Scuffed bytecode 15 fields", 15)?;
        for (i, variable) in self.variables.iter().enumerate() {
            builder.write_gm_string(&variable.name)?;
            variable.b15_data.serialize_if_bytecode_ver(builder, "Bytecode 15 data", 15)?;
            
            let occurrences = builder.variable_occurrences.get(i)
                .ok_or_else(|| format!("Could not resolve variable occurrence with index {i} in list with length {}", builder.function_occurrences.len()))?;
            let occurrence_count: usize = occurrences.len();
            let first_occurrence: i32 = match occurrences.first() {
                Some((occurrence, _)) => *occurrence as i32,
                None => variable.name_string_id,    // not sure if correct tbh
            };
            builder.write_usize(occurrence_count)?;
            builder.write_i32(first_occurrence);
        }
        Ok(())
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
impl GMElement for GMVariableB15Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let instance_type: GMInstanceType = parse_instance_type(reader.read_i32()? as i16)?;
        let variable_id: i32 = reader.read_i32()?;
        Ok(GMVariableB15Data { instance_type, variable_id })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(build_instance_type(&self.instance_type) as i32);
        builder.write_i32(self.variable_id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GMVariablesScuffed {
    pub var_count1: usize,
    pub var_count2: usize,
    pub max_local_var_count: usize,
}
impl GMElement for GMVariablesScuffed {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        // nobody knows what the fuck these values mean
        // TODO remember to increment these when a variable is added by a mod
        let var_count1: usize = reader.read_usize()?;
        let var_count2: usize = reader.read_usize()?;
        let max_local_var_count: usize = reader.read_usize()?;
        Ok(GMVariablesScuffed { var_count1, var_count2, max_local_var_count })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_usize(self.var_count1)?;
        builder.write_usize(self.var_count2)?;
        builder.write_usize(self.max_local_var_count)?;
        Ok(())
    }
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
        occurrence_pos += offset as usize;   // might overflow on last occurrence (name string id) but doesn't matter
    }

    let name_string_id: i32 = offset & 0xFFFFFF;
    reader.chunk = saved_chunk;
    reader.cur_pos = saved_position;
    Ok((occurrences, name_string_id))
}

