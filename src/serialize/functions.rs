use std::collections::HashMap;
use crate::deserialize::all::GMData;
use crate::deserialize::functions::GMCodeLocal;
use crate::serialize::chunk_writing::DataBuilder;

pub fn build_chunk_func(builder: &mut DataBuilder, gm_data: &GMData, function_occurrences_map: HashMap<usize, Vec<usize>>) -> Result<(), String> { 
    builder.start_chunk("FUNC")?;
    
    // write functions
    if gm_data.general_info.bytecode_version >= 14 {
        builder.write_usize(gm_data.functions.functions_by_index.len());
    }
    
    for (i, function) in gm_data.functions.functions_by_index.iter().enumerate() {
        // there is no pointer list
        builder.write_gm_string(&function.name)?;
        
        if let Some(occurrences) = function_occurrences_map.get(&i) {
            builder.write_usize(occurrences.len());
            builder.write_usize(occurrences[0]);
        } else {
            builder.write_i32(0);
            builder.write_i32(function.name_string_id);
        }
    }
    
    // write code locals
    if gm_data.general_info.bytecode_version >= 15 && !gm_data.general_info.is_version_at_least(2024, 8, 0, 0) {
        builder.write_usize(gm_data.code_locals.len());
        for code_local in &gm_data.code_locals {
            build_code_local(builder, code_local)?;
        }
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


fn build_code_local(builder: &mut DataBuilder, code_local: &GMCodeLocal) -> Result<(), String> {
    builder.write_usize(code_local.variables.len());
    builder.write_gm_string(&code_local.name)?;

    for variable in &code_local.variables {
        builder.write_usize(variable.index);
        builder.write_gm_string(&variable.name)?;
    }
    
    Ok(())
}

