use crate::chunk_reading::UTChunk;
use std::collections::HashMap;

pub struct UTFunction {
    pub name: String,
    pub occurrences: u32,
    pub first_occurrence: u32,      // pointer to some location in code (position relative/absolute to data.win)
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

pub fn parse_chunk_FUNC(mut chunk: UTChunk, strings: &HashMap<u32, String>) -> Result<(Vec<UTFunction>, Vec<UTCodeLocal>), String> {
    let functions_length: usize = chunk.read_usize()?;
    let mut functions: Vec<UTFunction> = Vec::with_capacity(functions_length);

    for _ in 0..functions_length {
        let function_name: String = chunk.read_ut_string(strings)?;
        let occurrences: u32 = chunk.read_u32()?;   // idk what this values actually represents
        let first_occurrence: u32 = chunk.read_u32()?;
        let function: UTFunction = UTFunction {
            name: function_name,
            occurrences,
            first_occurrence,
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
    //     println!("[Function]    {:<32} | {:<4} | {}", i.name, i.occurrences, i.first_occurrence);
    // }
    // for i in &code_locals {
    //     println!("[Code Local]    {:<48} | {:?}", i.name, i.variables);
    // }

    Ok((functions, code_locals))
}

