use std::collections::HashSet;
use crate::chunk_reading::UTChunk;
use crate::deserialize::strings::UTStrings;

#[derive(Debug, Clone)]
pub struct UTFunction {
    pub name: String,
    pub occurrences: HashSet<usize>,                // set of occurrences (call instructions) positions relative to chunk CODE
}


#[derive(Debug)]
pub struct UTCodeLocalVariable {
    pub index: usize,
    pub name: String,
}
pub struct UTCodeLocal {
    pub name: String,
    pub variables: Vec<UTCodeLocalVariable>,
}

pub fn parse_chunk_FUNC(mut chunk: UTChunk, strings: &UTStrings, chunk_CODE: &UTChunk) -> Result<(Vec<UTFunction>, Vec<UTCodeLocal>), String> {
    let functions_length: usize = chunk.read_usize()?;
    let mut functions: Vec<UTFunction> = Vec::with_capacity(functions_length);

    for _ in 0..functions_length {
        let function_name: String = chunk.read_ut_string(strings)?;
        let occurrence_count: usize = chunk.read_usize()?;
        let first_occurrence: i32 = chunk.read_i32()? - chunk_CODE.abs_pos as i32;
        let occurrences: HashSet<usize> = get_occurrences(occurrence_count, first_occurrence, chunk_CODE);
        let function: UTFunction = UTFunction {
            name: function_name,
            occurrences,
        };
        functions.push(function);
    }

    let code_locals_length: usize = chunk.read_usize()?;
    let mut code_locals: Vec<UTCodeLocal> = Vec::with_capacity(code_locals_length);

    for _ in 0..code_locals_length {
        let local_variables_count: usize = chunk.read_usize()?;
        let name: String = chunk.read_ut_string(&strings)?;
        let mut variables: Vec<UTCodeLocalVariable> = Vec::with_capacity(local_variables_count);

        for _ in 0..local_variables_count {
            let variable_index: usize = chunk.read_usize()?;
            let variable_name: String = chunk.read_ut_string(&strings)?;
            let variable: UTCodeLocalVariable = UTCodeLocalVariable {
                index: variable_index,
                name: variable_name,
            };
            variables.push(variable);
        }

        let code_local: UTCodeLocal = UTCodeLocal {
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


fn get_occurrences(count: usize, first_occurrence: i32, chunk_CODE: &UTChunk) -> HashSet<usize> {
    let mut occurrences: HashSet<usize> = HashSet::new();
    let mut occurrence: i32 = first_occurrence + 4;

    // let mut i = 0;

    for _ in 0..count {
        // println!("occ {occurrence} | {}", chunk_CODE.abs_pos);
        // occurrence -= chunk_CODE.abs_pos as i32;
        occurrences.insert(occurrence as usize - 4);
        // TODO index safety
        let raw: [u8; 4] = chunk_CODE.data[(occurrence as usize) .. (occurrence as usize)+4].try_into().unwrap();
        // println!("{}", hexdump(&chunk_CODE.data, (occurrence as usize)-4, Some(occurrence as usize+4)).unwrap());
        if chunk_CODE.data[occurrence as usize - 1] != 0xD9 {
            break;
        }
        occurrence += i32::from_le_bytes(raw);
        // i += 1;
    }

    // println!("FUNCITONSBDF | expected: {count:<10},  actual: {i:<10}   {}", count==i);

    occurrences
}


pub fn get_function(functions: &[UTFunction], position: usize) -> Result<UTFunction, String> {
    for function in functions {
        // println!("{:<30} {:?}", function.name, function.occurrences);
        if function.occurrences.contains(&position) {
            // lowkey disgusting solution but whatever i need to save memory
            return Ok(UTFunction {
                name: function.name.clone(),
                occurrences: HashSet::with_capacity(0)
            });
            // return Ok(function.clone());
        }
    }
    Err(format!("Could not find function for position {position} (len functions: {}).", functions.len()))
}
