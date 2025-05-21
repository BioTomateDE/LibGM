use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::{parse_instance_type, parse_occurrence_chain, GMInstanceType};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub instance_type: GMInstanceType,
    pub variable_id: Option<i32>,
    pub name_string_id: i32,
}

#[derive(Debug, Clone)]
pub struct GMVariables {
    /// List of all variables; mixing global, local and self.
    pub variables: Vec<GMVariable>,
    /// Maps absolute positions of variable occurrences (in code) to variable references.
    /// Only meant for parsing code; irrelevant (and potentially incorrect) after parsing.
    pub occurrences_to_refs: HashMap<usize, GMRef<GMVariable>>,
    pub scuffed: Option<GMVariablesScuffed>,
}

#[derive(Debug, Clone)]
pub struct GMVariablesScuffed {
    pub globals_count: usize,
    pub instances_count: usize,
    pub locals_count: usize,
}

pub fn parse_chunk_vari(chunk: &mut GMChunk, strings: &GMStrings, general_info: &GMGeneralInfo, chunk_code: &mut GMChunk) -> Result<GMVariables, String> {
    chunk.cur_pos = 0;

    let variables_length: usize = if general_info.bytecode_version >= 15 { 20 } else { 12 };
    let scuffed: Option<GMVariablesScuffed> = if general_info.bytecode_version >= 15 {
        let globals_count: usize = chunk.read_usize()?;         // these variables don't actually represent what they say
        let instances_count: usize = chunk.read_usize()?;       // because gamemaker is weird
        let locals_count: usize = chunk.read_usize()?;          // TODO: probably needs to be incremented when a variable is added?
        Some(GMVariablesScuffed {
            globals_count,
            instances_count,
            locals_count,
        })
    } else { None };

    let mut variables: Vec<GMVariable> = Vec::with_capacity(chunk.data.len() / variables_length);
    let mut occurrence_map: HashMap<usize, GMRef<GMVariable>> = HashMap::new();
    let mut cur_index: usize = 0;

    while chunk.cur_pos + variables_length <= chunk.data.len() {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;

        let mut variable_id: Option<i32> = None;
        let instance_type: GMInstanceType = if general_info.bytecode_version >= 15 {
            let instance_type_: GMInstanceType = parse_instance_type(chunk.read_i32()? as i16)
                .map_err(|e| format!("Could not get instance type for variable \"{}\" while parsing chunk VARI: {e}", name.display(strings)))?;
            variable_id = Some(chunk.read_i32()?);
            instance_type_
        } else {
            GMInstanceType::Undefined   // idk if this information is even available atp
        };

        let occurrences_count: usize = chunk.read_usize()?;
        let first_occurrence_address: i32 = chunk.read_i32()?;

        let (occurrences, name_string_id): (Vec<usize>, i32) = parse_occurrence_chain(
            chunk_code,
            name.display(strings),
            first_occurrence_address,
            occurrences_count,
        )?;

        for occurrence in occurrences {
            if let Some(old_value) = occurrence_map.insert(occurrence, GMRef::new(cur_index)) {
                return Err(format!(
                    "Conflicting occurrence positions while parsing variables: absolute position {} \
                    was already set for {:?} variable #{} with name \"{}\"; trying to set to variable #{} with name \"{}\".",
                    occurrence, instance_type, old_value.index, old_value.resolve(&variables)?.name.display(strings), cur_index, name.display(strings),
                ))
            }
        }

        variables.push(GMVariable {
            name,
            instance_type,
            variable_id,
            name_string_id,
        });
        cur_index += 1;
    }

    Ok(GMVariables {
        variables,
        occurrences_to_refs: occurrence_map,
        scuffed,
    })
}

