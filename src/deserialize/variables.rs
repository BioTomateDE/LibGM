use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::{parse_instruction, read_variable_reference, GMCodeBlob, GMInstanceType, GMInstruction, GMPopInstruction};
use crate::deserialize::functions::GMFunctions;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub instance_type: GMInstanceType,
    pub variable_id: i32,
    pub occurrences: Vec<GMPopInstruction>,
    // pub occurrences_count: u32,
    // pub first_occurrence_address: u32,
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

    for i in 0..globals_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        if general_info.bytecode_version >= 15 {
            let instance_type: i32 = chunk.read_i32()?;
            let variable_id: i32 = chunk.read_i32()?;
            // TODO
        }
        let occurrences_count: usize = chunk.read_usize()?;
        let first_occurrence_address: i32 = chunk.read_i32()?;
        let occurrences: Vec<GMPopInstruction> = parse_occurrence_chain(chunk_code, name.display(strings), GMRef::new(i), first_occurrence_address, occurrences_count)?;

        global_variables.push(GMVariable {
            name,
            instance_type: GMInstanceType::Global,
            variable_id: 7, // stub
            occurrences,
        })
    }

    for i in 0..instances_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let variable_id: i32 = chunk.read_i32()?;
        let occurrences_count: usize = chunk.read_usize()?;
        let first_occurrence_address: i32 = chunk.read_i32()?;
        let occurrences: Vec<GMPopInstruction> = parse_occurrence_chain(chunk_code, name.display(strings), GMRef::new(i), first_occurrence_address, occurrences_count)?;

        instance_variables.push(GMVariable {
            name,
            instance_type: GMInstanceType::Self_,
            variable_id,
            occurrences,
        })
    }

    for i in 0..locals_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let variable_id: i32 = chunk.read_i32()?;
        let occurrences_count: usize = chunk.read_usize()?;
        let first_occurrence_address: i32 = chunk.read_i32()?;
        let occurrences: Vec<GMPopInstruction> = parse_occurrence_chain(chunk_code, name.display(strings), GMRef::new(i), first_occurrence_address, occurrences_count)?;

        local_variables.push(GMVariable {
            name,
            instance_type: GMInstanceType::Local,
            variable_id,
            occurrences,
        })
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
        // let raw_data: &[u8] = chunk_code.data.get(occurrence_pos..occurrence_pos + 8)
        //     .ok_or(format!(
        //         "First occurrence of variable \"{}\" is out of bounds (requires more than 8 bytes in chunk CODE); should be: {} <= {} < {}.",
        //         variable_name, chunk_code.abs_pos, first_occurrence_abs_pos + 8, chunk_code.abs_pos + chunk_code.data.len(),
        //     ))?;

        // let mut blob = GMCodeBlob {
        //     raw_data: raw_data.to_vec(),
        //     len: raw_data.len(),
        //     file_index: 0,
        // };
        //
        // let fake_variables = GMVariables { variables_by_index: vec![] };
        // let fake_functions = GMFunctions { functions_by_index: vec![], occurrences_to_refs: Default::default() };
        // let instruction: GMInstruction = parse_instruction(&mut blob, bytecode14, &fake_variables, &fake_functions, occurrence_pos)?;
        // if let GMInstruction::Pop(pop_instruction) = instruction {
        //     pop_instruction.destination
        // } else {
        //     return Err(format!("Unexpected instruction type while parsing variable occurrences for variable \"{variable_name}\": {instruction:?}"))
        // }
        let (instruction, offset): (GMPopInstruction, usize) = read_variable_reference(chunk_code, variable_ref.clone())?;
        occurrence_pos += offset;
        occurrences.push(instruction);
    }

    Ok(occurrences)
}

