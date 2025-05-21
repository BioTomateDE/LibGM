use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::parse_occurrence_chain;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMFunction {
    pub name: GMRef<String>,
    pub name_string_id: i32,
}


#[derive(Debug, Clone)]
pub struct GMFunctions {
    pub functions_by_index: Vec<GMFunction>,
    /// Maps absolute positions of function occurrences (in code) to function references.
    /// Only meant for parsing code; irrelevant (and potentially incorrect) after parsing.
    pub occurrences_to_refs: HashMap<usize, GMRef<GMFunction>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeLocalVariable {
    pub index: usize,
    pub name: GMRef<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeLocal {
    pub name: GMRef<String>,
    pub variables: Vec<GMCodeLocalVariable>,
}

pub fn parse_chunk_func(chunk: &mut GMChunk, strings: &GMStrings, chunk_code: &mut GMChunk) -> Result<(GMFunctions, Vec<GMCodeLocal>), String> {
    chunk.cur_pos = 0;
    let functions_count: usize = chunk.read_usize()?;
    let mut functions_by_index: Vec<GMFunction> = Vec::with_capacity(functions_count);
    let mut occurrences_to_refs: HashMap<usize, GMRef<GMFunction>> = HashMap::new();

    for i in 0..functions_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let occurrence_count: usize = chunk.read_usize()?;
        let first_occurrence: i32 = chunk.read_i32()?;
        let (occurrences, name_string_id): (Vec<usize>, i32) = parse_occurrence_chain(chunk_code, name.resolve(&strings.strings_by_index)?, first_occurrence, occurrence_count)?;

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
            name_string_id,
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

    Ok((functions, code_locals))
}
