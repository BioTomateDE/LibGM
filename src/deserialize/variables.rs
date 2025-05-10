use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::{parse_instance_type, GMInstanceType};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub instance_type: GMInstanceType,
    pub variable_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct GMVariables {
    /// List of all variables; mixing global, local and self.
    pub variables: Vec<GMVariable>,
    /// Maps absolute positions of variable occurrences (in code) to variable references.
    /// Only meant for parsing code; irrelevant (and potentially incorrect) after parsing.
    pub occurrence_map: HashMap<usize, GMRef<GMVariable>>,
}

pub fn parse_chunk_vari(chunk: &mut GMChunk, strings: &GMStrings, general_info: &GMGeneralInfo, chunk_code: &mut GMChunk) -> Result<GMVariables, String> {
    chunk.cur_pos = 0;

    let variables_length: usize = if general_info.bytecode_version >= 15 {
        let _globals_count: usize = chunk.read_usize()?;
        let _instances_count: usize = chunk.read_usize()?;
        let _locals_count: usize = chunk.read_usize()?;
        20
    } else {12};

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

        let occurrences: Vec<usize> = parse_occurrence_chain(
            chunk_code,
            name.display(strings),
            first_occurrence_address,
            occurrences_count
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
        });
        cur_index += 1;
    }

    Ok(GMVariables {
        variables,
        occurrence_map,
    })
}



// could be made more efficient by passing in a &mut to the
// occurrence map rather than inserting them all later
// (this also applies to function occurrences)
fn parse_occurrence_chain(
    chunk_code: &mut GMChunk,
    variable_name: &str,
    first_occurrence_abs_pos: i32,
    occurrence_count: usize,
) -> Result<Vec<usize>, String> {
    if occurrence_count < 1 {
        return Ok(vec![]);
    }

    let occurrence_pos: i32 = first_occurrence_abs_pos - chunk_code.abs_pos as i32 + 4;
    let mut occurrence_pos: usize = occurrence_pos.try_into()
        .map_err(|_| format!(
            "First occurrence of variable \"{}\" is out of bounds; should be: {} <= {} < {}.",
            variable_name, chunk_code.abs_pos, first_occurrence_abs_pos, chunk_code.abs_pos + chunk_code.data.len(),
        ))?;

    let mut occurrences: Vec<usize> = Vec::with_capacity(occurrence_count);

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        chunk_code.cur_pos = occurrence_pos;
        let offset: usize = read_variable_reference(chunk_code)?;
        occurrence_pos += offset;
    }

    // occurrence_pos now represents "name string id" {~~}

    Ok(occurrences)
}


pub fn read_variable_reference(chunk_code: &mut GMChunk) -> Result<usize, String> {
    // log::debug!("{} | {}", chunk_code.cur_pos, crate::printing::hexdump(chunk_code.data, chunk_code.cur_pos-8, Some(chunk_code.cur_pos+8))?);
    let raw_value: i32 = chunk_code.read_i32()?;
    let next_occurrence_offset: i32 = raw_value & 0x07FFFFFF;
    // log::info!("b {next_occurrence_offset}");
    let next_occurrence_offset: usize = next_occurrence_offset as usize;
    Ok(next_occurrence_offset)
}

