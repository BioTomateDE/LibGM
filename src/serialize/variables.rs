use crate::deserialize::all::GMData;
use crate::deserialize::code::GMVariableType;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::variables::{GMVariable, GMVariableB15Data, GMVariablesScuffed};
use crate::serialize::chunk_writing::DataBuilder;
use crate::serialize::code::{build_instance_type, Occurrences};

pub fn build_chunk_vari(builder: &mut DataBuilder, gm_data: &GMData, variable_occurrences_map: Occurrences) -> Result<(), String> {
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


fn build_variable(builder: &mut DataBuilder, general_info: &GMGeneralInfo, variable: &GMVariable, variable_occurrences: Option<&Vec<(usize, Option<GMVariableType>)>>) -> Result<(), String> {
    builder.write_gm_string(&variable.name)?;
    if general_info.bytecode_version >= 15 {
        let b15_data: &GMVariableB15Data = variable.b15_data.as_ref().ok_or("Bytecode 15 data not set")?;
        builder.write_i32(i32::from(build_instance_type(&b15_data.instance_type)));
        builder.write_i32(b15_data.variable_id);
    }
    
    if let Some(occurrences) = variable_occurrences {
        builder.write_usize(occurrences.len());
        builder.write_usize(occurrences[0].0);
    } else {
        builder.write_i32(0);      // TODO not sure if this should be 0 or -1   (same for functions)
        builder.write_i32(variable.name_string_id);
    }

    Ok(())
}
