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
    let functions_length: usize = chunk.read_usize()?;
    let mut functions_by_index: Vec<GMFunction> = Vec::with_capacity(functions_length);
    let mut occurrences_to_refs: HashMap<usize, GMRef<GMFunction>> = HashMap::new();

    for i in 0..functions_length {
        let function_name: GMRef<String> = chunk.read_gm_string(strings)?;
        let occurrence_count: usize = chunk.read_usize()?;
        let first_occurrence: i32 = chunk.read_i32()? - chunk_code.abs_pos as i32;
        let occurrences: Vec<usize> = get_occurrences(occurrence_count, first_occurrence, chunk_code);
        for occurrence in &occurrences {
            occurrences_to_refs.insert(*occurrence, GMRef::function(i));
        }
        let function: GMFunction = GMFunction {
            name: function_name,
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


fn get_occurrences(count: usize, first_occurrence: i32, chunk_code: &GMChunk) -> Vec<usize> {
    let mut occurrences: Vec<usize> = Vec::new();
    let mut occurrence: i32 = first_occurrence + 4;

    // let mut i = 0;

    for _ in 0..count {
        // println!("occ {occurrence} | {}", chunk_CODE.abs_pos);
        // occurrence -= chunk_CODE.abs_pos as i32;
        occurrences.push(occurrence as usize - 4);
        // TODO index safety
        let raw: [u8; 4] = chunk_code.data[(occurrence as usize) .. (occurrence as usize)+4].try_into().unwrap();
        // println!("{}", hexdump(&chunk_CODE.data, (occurrence as usize)-4, Some(occurrence as usize+4)).unwrap());
        if chunk_code.data[occurrence as usize - 1] != 0xD9 {
            break;
        }
        occurrence += i32::from_le_bytes(raw);
        // i += 1;
    }

    // println!("FUNCITONSBDF | expected: {count:<10},  actual: {i:<10}   {}", count==i);

    occurrences
}

