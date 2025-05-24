use std::collections::HashMap;
use crate::deserialize::all::GMData;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_func(data_builder: &mut DataBuilder, gm_data: &GMData, function_occurrences_map: HashMap<usize, Vec<usize>>) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "FUNC");
    
    // write functions
    builder.write_usize(gm_data.functions.functions_by_index.len());

    for (i, function) in gm_data.functions.functions_by_index.iter().enumerate() {
        // there is no pointer list
        builder.write_gm_string(data_builder, &function.name)?;
        
        if let Some(occurrences) = function_occurrences_map.get(&i) {
            builder.write_usize(occurrences.len());
            builder.write_usize(occurrences[0]);
        } else {
            builder.write_i32(-1);
            builder.write_i32(function.name_string_id);
        }
    }
    
    // write code locals
    builder.write_usize(gm_data.code_locals.len());

    for i in 0..gm_data.code_locals.len() {
        data_builder.write_pointer_placeholder(&mut builder, GMPointer::CodeLocal(i))?;
    }

    for (i, code_local) in gm_data.code_locals.iter().enumerate() {
        data_builder.resolve_pointer(&mut builder, GMPointer::CodeLocal(i))?;
        builder.write_gm_string(data_builder, &code_local.name)?;
        builder.write_usize(code_local.variables.len());
        
        for variable in &code_local.variables {
            builder.write_usize(variable.index);
            builder.write_gm_string(data_builder, &variable.name)?;
        }
    }

    builder.finish(data_builder)?;
    Ok(())
}

