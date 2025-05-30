use std::collections::HashMap;
use crate::deserialize::all::GMData;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::variables::{GMVariable, GMVariablesScuffed};
use crate::serialize::chunk_writing::DataBuilder;
use crate::serialize::code::build_instance_type;

pub fn build_chunk_vari(builder: &mut DataBuilder, gm_data: &GMData, variable_occurrences_map: HashMap<usize, Vec<usize>>) -> Result<(), String> {
    builder.start_chunk("VARI")?;

    if gm_data.general_info.bytecode_version >= 15 {
        let scuffed: &GMVariablesScuffed = gm_data.variables.scuffed.as_ref().ok_or_else(|| "Variables scuffed fields (variable counts) not set")?;
        builder.write_usize(scuffed.globals_count);
        builder.write_usize(scuffed.instances_count);
        builder.write_usize(scuffed.locals_count);
    }

    for (i, variable) in gm_data.variables.variables.iter().enumerate() {
        // there is no pointer list.
        build_variable(builder, &gm_data.general_info, variable, variable_occurrences_map.get(&i))
            .map_err(|e| format!(
                "{e} for variable #{i} with name \"{}\"",
                variable.name.display(&gm_data.strings),
            ))?;
    }
    
    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


fn build_variable(builder: &mut DataBuilder, general_info: &GMGeneralInfo, variable: &GMVariable, variable_occurrences: Option<&Vec<usize>>) -> Result<(), String> {
    builder.write_gm_string(&variable.name)?;
    if general_info.bytecode_version >= 15 {
        builder.write_i32(build_instance_type(&variable.instance_type) as i32);
        builder.write_i32(variable.variable_id.ok_or("Variable ID not set")?);
    }
    
    if let Some(occurrences) = variable_occurrences {
        builder.write_usize(occurrences.len());
        builder.write_usize(occurrences[0]);
    } else {
        builder.write_i32(0);      // TODO not sure if this should be 0 or -1   (same for functions)
        builder.write_i32(variable.name_string_id);
    }

    Ok(())
}
