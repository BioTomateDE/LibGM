use std::collections::{HashMap, HashSet};
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::strings::{GMStringRef, GMStrings};

#[derive(Debug, Clone)]
pub struct GMFunction {
    pub name: GMStringRef,
    // pub occurrences: HashSet<usize>,                // set of occurrences (call instructions) positions relative to chunk CODE
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GMFunctionRef {
    pub index: usize,
}

impl GMFunctionRef {
    pub fn resolve(&self, functions: &GMFunctions) -> Result<GMFunction, String> {
        match functions.functions_by_index.get(self.index) {
            Some(func) => Ok(func.clone()),
            None => Err(format!(     // internal error perchance
                "Could not resolve function with index {} in list with length {}.",
                self.index, functions.functions_by_index.len(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GMFunctions {
    functions_by_index: Vec<GMFunction>,
    occurrences_to_indexes: HashMap<usize, usize>,     // maps all occurrence addresses/positions (relative to chunk CODE) to function indexes
}

impl GMFunctions {
    pub fn get_function_by_occurrence(&self, occurrence_position: usize) -> Result<GMFunctionRef, String> {
        let function_index: usize = match self.occurrences_to_indexes.get(&occurrence_position) {
            Some(index) => *index,
            None => return Err(format!(
                "Could not find any function with absolute occurrence position {} in map with length {} (functions len: {}).",
                occurrence_position, self.occurrences_to_indexes.len(), self.functions_by_index.len(),
            )),
        };
        Ok(GMFunctionRef {
            index: function_index,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GMCodeLocalVariable {
    pub index: usize,
    pub name: GMStringRef,
}
#[derive(Debug, Clone)]
pub struct GMCodeLocal {
    pub name: GMStringRef,
    pub variables: Vec<GMCodeLocalVariable>,
}

pub fn parse_chunk_func(chunk: &mut GMChunk, strings: &GMStrings, chunk_CODE: &GMChunk) -> Result<(GMFunctions, Vec<GMCodeLocal>), String> {
    chunk.file_index = 0;
    let functions_length: usize = chunk.read_usize()?;
    let mut functions_by_index: Vec<GMFunction> = Vec::with_capacity(functions_length);
    let mut occurrences_to_indexes: HashMap<usize, usize> = HashMap::new();

    for i in 0..functions_length {
        let function_name: GMStringRef = chunk.read_gm_string(strings)?;
        let occurrence_count: usize = chunk.read_usize()?;
        let first_occurrence: i32 = chunk.read_i32()? - chunk_CODE.abs_pos as i32;
        let occurrences: HashSet<usize> = get_occurrences(occurrence_count, first_occurrence, chunk_CODE);
        for occurrence in occurrences {
            occurrences_to_indexes.insert(occurrence, i);
        }
        let function: GMFunction = GMFunction {
            name: function_name,
        };
        functions_by_index.push(function);
    }
    let functions: GMFunctions = GMFunctions {
        functions_by_index,
        occurrences_to_indexes,
    };

    let code_locals_length: usize = chunk.read_usize()?;
    let mut code_locals: Vec<GMCodeLocal> = Vec::with_capacity(code_locals_length);

    for _ in 0..code_locals_length {
        let local_variables_count: usize = chunk.read_usize()?;
        let name: GMStringRef = chunk.read_gm_string(&strings)?;
        let mut variables: Vec<GMCodeLocalVariable> = Vec::with_capacity(local_variables_count);

        for _ in 0..local_variables_count {
            let variable_index: usize = chunk.read_usize()?;
            let variable_name: GMStringRef = chunk.read_gm_string(&strings)?;
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


fn get_occurrences(count: usize, first_occurrence: i32, chunk_CODE: &GMChunk) -> HashSet<usize> {
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

