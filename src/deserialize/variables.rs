use itertools::izip;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::{parse_instance_type, read_variable_reference, GMInstanceType, GMPopInstruction};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub instance_type: GMInstanceType,
    pub variable_id: Option<i32>,
    pub occurrences: Vec<GMPopInstruction>,
}

#[derive(Debug, Clone)]
pub struct GMVariables {
    pub global_variables: Vec<GMVariable>,
    pub instance_variables: Vec<GMVariable>,
    pub local_variables: Vec<GMVariable>,
}

pub fn parse_chunk_vari(chunk: &mut GMChunk, strings: &GMStrings, general_info: &GMGeneralInfo, chunk_code: &mut GMChunk) -> Result<GMVariables, String> {
    // TODO please someone fix this. also in parsing chunk CODE.
    chunk.cur_pos = 0;
    let globals_count: usize = chunk.read_usize()?;         // the amount of global variables
    let instances_count: usize = chunk.read_usize()?;       // the amount of `Self` variables (local to own object)
    let locals_count: usize = chunk.read_usize()?;          // the amount of `Local` variables (local to own code script)

    let mut global_variables: Vec<GMVariable> = Vec::with_capacity(globals_count);
    let mut instance_variables: Vec<GMVariable> = Vec::with_capacity(instances_count);
    let mut local_variables: Vec<GMVariable> = Vec::with_capacity(locals_count);

    for (variable_count, variables, default_instance_type) in izip!(
        [globals_count, instances_count, locals_count],
        [&mut global_variables, &mut instance_variables, &mut local_variables],
        [GMInstanceType::Global, GMInstanceType::Self_(None), GMInstanceType::Local],
    ) {
        for i in 0..variable_count {
            let name: GMRef<String> = chunk.read_gm_string(strings)?;

            // bytecode>=15 might reads instance type here, so maybe it doesn't have the 3 global/instance/local count in the beginning?
            let mut instance_type: GMInstanceType = default_instance_type.clone();
            let mut variable_id: Option<i32> = None;
            if general_info.bytecode_version >= 15 {
                instance_type = parse_instance_type(chunk.read_i32()? as i16)
                    .map_err(|e| format!("Could not get instance type for variable \"{}\" while parsing chunk VARI: {e}", name.display(strings)))?;
                // let instance_type_: i32 = chunk.read_i32()?;
                // instance_type = instance_type_.try_into()
                //     .map_err(|_| format!("Invalid instance type 0x{instance_type_:8X} at absolute position {} while parsing variables.", chunk.cur_pos+chunk.abs_pos))?;
                variable_id = Some(chunk.read_i32()?);
            }

            let occurrences_count: usize = chunk.read_usize()?;
            let first_occurrence_address: i32 = chunk.read_i32()?;

            let occurrences: Vec<GMPopInstruction> = parse_occurrence_chain(
                chunk_code,
                name.display(strings),
                GMRef::new(i),
                first_occurrence_address,
                occurrences_count
            )?;

            variables.push(GMVariable {
                name,
                instance_type,
                variable_id,
                occurrences,
            });
        }
    }

    // log::debug!("var len: {}", variables_by_index.len());
    Ok(GMVariables {
        global_variables,
        instance_variables,
        local_variables,
    })
}


fn parse_occurrence_chain(
    chunk_code: &mut GMChunk,
    variable_name: &str,
    variable_ref: GMRef<GMVariable>,
    first_occurrence_abs_pos: i32,
    occurrence_count: usize,
) -> Result<Vec<GMPopInstruction>, String> {
    if occurrence_count < 1 {
        return Ok(vec![]);
    }

    let occurrence_pos: i32 = first_occurrence_abs_pos - chunk_code.abs_pos as i32;
    let mut occurrence_pos: usize = occurrence_pos.try_into()
        .map_err(|_| format!(
            "First occurrence of variable \"{}\" is out of bounds; should be: {} <= {} < {}.",
            variable_name, chunk_code.abs_pos, first_occurrence_abs_pos, chunk_code.abs_pos + chunk_code.data.len(),
        ))?;

    let mut occurrences: Vec<GMPopInstruction> = Vec::with_capacity(occurrence_count);

    for _ in 0..occurrence_count {
        chunk_code.cur_pos = occurrence_pos;
        let (instruction, offset): (GMPopInstruction, usize) = read_variable_reference(chunk_code, variable_ref.clone())?;
        occurrence_pos += offset;
        occurrences.push(instruction);
    }

    Ok(occurrences)
}

