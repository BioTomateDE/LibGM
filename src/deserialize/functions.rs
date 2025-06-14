use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::parse_occurrence_chain;
use crate::deserialize::general_info::GMGeneralInfo;
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

pub fn parse_chunk_func(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings, chunk_code: &mut GMChunk) -> Result<(GMFunctions, Vec<GMCodeLocal>), String> {
    chunk.cur_pos = 0;

    // read functions
    let functions_count: usize = if general_info.bytecode_version <= 14 {
        chunk.data.len() / 12
    } else {
        chunk.read_usize_count()?
    };
    
    let mut functions_by_index: Vec<GMFunction> = Vec::with_capacity(functions_count);
    let mut occurrences_to_refs: HashMap<usize, GMRef<GMFunction>> = HashMap::with_capacity(functions_count);

    for i in 0..functions_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let occurrence_count: usize = chunk.read_usize_pos()?;
        let first_occurrence_abs_pos: i32 = chunk.read_i32()?;

        let (occurrences, name_string_id): (Vec<usize>, i32) = parse_occurrence_chain(chunk_code, name.resolve(&strings.strings_by_index)?, first_occurrence_abs_pos, occurrence_count)?;

        for occurrence in &occurrences {
            if let Some(old_value) = occurrences_to_refs.insert(*occurrence, GMRef::new(i)) {
                return Err(format!(
                    "Conflicting occurrence positions while parsing functions: absolute position {} \
                    was already set for function #{} with name \"{}\"; trying to set to function #{} with name \"{}\"",
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

    // read code locals
    let code_locals_count: usize = if general_info.bytecode_version <= 14 || general_info.is_version_at_least(2024, 8, 0, 0) {
        0
    } else {
        chunk.read_usize_count()?
    };
    
    let mut code_locals: Vec<GMCodeLocal> = Vec::with_capacity(code_locals_count);
    for _ in 0..code_locals_count {
        code_locals.push(read_code_local(chunk, strings)?);
    }

    Ok((functions, code_locals))
}


fn read_code_local(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMCodeLocal, String> {
    let local_variables_count: usize = chunk.read_usize_count()?;
    let name: GMRef<String> = chunk.read_gm_string(&strings)?;
    let mut variables: Vec<GMCodeLocalVariable> = Vec::with_capacity(local_variables_count);

    for _ in 0..local_variables_count {
        let variable_index: usize = chunk.read_usize_count()?;
        let variable_name: GMRef<String> = chunk.read_gm_string(&strings)?;
        let variable: GMCodeLocalVariable = GMCodeLocalVariable {
            index: variable_index,
            name: variable_name,
        };
        variables.push(variable);
    }

    Ok(GMCodeLocal {
        name,
        variables,
    })
}

