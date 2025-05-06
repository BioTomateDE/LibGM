use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone)]
pub struct GMFunction {
    pub name: GMRef<String>,
    pub occurrences: Vec<usize>,                // list of occurrences (call instructions) positions relative to chunk CODE
}


#[derive(Debug, Clone)]
pub struct GMFunctions {
    pub functions_by_index: Vec<GMFunction>,
    pub occurrences_to_refs: HashMap<usize, GMRef<GMFunction>>,     // maps all occurrence addresses/positions (relative to chunk CODE) to function refs
}

#[derive(Debug, Clone)]
pub struct GMCodeLocalVariable {
    pub index: usize,
    pub name: GMRef<String>,
}
#[derive(Debug, Clone)]
pub struct GMCodeLocal {
    pub name: GMRef<String>,
    pub variables: Vec<GMCodeLocalVariable>,
}

pub fn parse_chunk_func(chunk: &mut GMChunk, strings: &GMStrings, chunk_code: &GMChunk) -> Result<(GMFunctions, Vec<GMCodeLocal>), String> {
    chunk.cur_pos = 0;
    let functions_count: usize = chunk.read_usize()?;
    let mut functions_by_index: Vec<GMFunction> = Vec::with_capacity(functions_count);
    let mut occurrences_to_refs: HashMap<usize, GMRef<GMFunction>> = HashMap::new();

    for i in 0..functions_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let occurrence_count: usize = chunk.read_usize()?;
        let first_occurrence: i32 = chunk.read_i32()?;
        let occurrences: Vec<usize> = parse_occurrence_chain(chunk_code, name.resolve(&strings.strings_by_index)?, first_occurrence, occurrence_count)?;

        for occurrence in &occurrences {
            if let Some(old_value) = occurrences_to_refs.insert(*occurrence, GMRef::new(i)) {
                return Err(format!(
                    "Conflicting occurrence positions while parsing functions: absolute position {} \
                    was already set for function #{} with name \"{}\"; trying to set to function #{} with name \"{}\".",
                    occurrence, old_value.index, old_value.resolve(&functions_by_index)?.name.display(strings), i, name.display(strings),
                ))
            }
        }

        let function: GMFunction = GMFunction {
            name,
            occurrences,
        };
        functions_by_index.push(function);
    }
    let functions: GMFunctions = GMFunctions {
        functions_by_index,
        occurrences_to_refs,
    };

    let code_locals_length: usize = chunk.read_usize()?;
    let mut code_locals: Vec<GMCodeLocal> = Vec::with_capacity(code_locals_length);

    for _ in 0..code_locals_length {
        let local_variables_count: usize = chunk.read_usize()?;
        let name: GMRef<String> = chunk.read_gm_string(&strings)?;
        let mut variables: Vec<GMCodeLocalVariable> = Vec::with_capacity(local_variables_count);

        for _ in 0..local_variables_count {
            let variable_index: usize = chunk.read_usize()?;
            let variable_name: GMRef<String> = chunk.read_gm_string(&strings)?;
            let variable: GMCodeLocalVariable = GMCodeLocalVariable {
                index: variable_index,
                name: variable_name,
            };
            variables.push(variable);
        }

        let code_local: GMCodeLocal = GMCodeLocal {
            name,
            variables,
        };
        code_locals.push(code_local);

    }

    // for i in &functions {
    //     println!("[Function]    {:<32} | {:<4} | {:?}", i.name, i.occurrences.len(), i.occurrences);
    // }
    // for i in &code_locals {
    //     println!("[Code Local]    {:<48} | {:?}", i.name, i.variables);
    // }

    Ok((functions, code_locals))
}


fn parse_occurrence_chain(chunk_code: &GMChunk, function_name: &str, first_occurrence_abs_pos: i32, occurrence_count: usize) -> Result<Vec<usize>, String> {
    if occurrence_count < 1 {
        return Ok(vec![]);
    }

    let mut occurrence_pos: i32 = first_occurrence_abs_pos - chunk_code.abs_pos as i32 + 4;
    let mut occurrence_pos: usize = occurrence_pos.try_into()
        .map_err(|_| format!(
            "First occurrence of function \"{}\" is out of bounds; should be: {} <= {} < {}.",
            function_name, chunk_code.abs_pos, first_occurrence_abs_pos, chunk_code.abs_pos + chunk_code.data.len(),
        ))?;

    let mut occurrences: Vec<usize> = Vec::with_capacity(occurrence_count);

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        let raw: [u8; 4] = chunk_code.data.get(occurrence_pos .. occurrence_pos+4)
            .ok_or_else(|| format!("Trying to read next occurrence offset out of bounds \
            while parsing function reference chain: {} > {}", occurrence_pos, chunk_code.data.len()))?
            .try_into().unwrap();

        if chunk_code.data[occurrence_pos- 1] != 0xD9 {
            log::error!("Function {function_name} not D9: {occurrence_pos} {}", chunk_code.data[occurrence_pos - 1]);
            break;
        }
        occurrence_pos += i32::from_le_bytes(raw) as usize;
    }


    Ok(occurrences)
}

