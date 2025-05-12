use crate::deserialize::all::GMData;
use crate::deserialize::code::GMInstanceType;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::variables::GMVariable;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_vari(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "VARI");

    let variable_count: usize = gm_data.variables.variables.len();
    builder.write_usize(variable_count);

    for i in 0..variable_count {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::variable(i))?;
    }

    for (i, variable) in gm_data.variables.variables.iter().enumerate() {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::variable(i))?;
        build_variable(data_builder, &mut builder, &gm_data.general_info, variable)
            .map_err(|e| format!("{e} for variable #{i} with name {}", variable.name.display(&gm_data.strings)))?;
    }


    builder.finish(data_builder)?;
    Ok(())
}

fn build_variable(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, variable: &GMVariable) -> Result<(), String> {
    builder.write_gm_string(data_builder, &variable.name)?;
    if general_info.bytecode_version >= 15 {
        builder.write_i32(build_instance_type(&variable.instance_type));
        builder.write_i32(variable.variable_id.ok_or("Variable ID not set")?);
    }
    // TODO: go through code and find occurrences idk

    Ok(())
}


pub fn build_instance_type(instance_type: &GMInstanceType) -> i32 {
    match instance_type {
        GMInstanceType::Undefined => 0,
        GMInstanceType::Instance(Some(obj_ref)) => obj_ref.index as i32,
        GMInstanceType::Instance(None) => -1,
        GMInstanceType::Other => -2,
        GMInstanceType::All => -3,
        GMInstanceType::Noone => -4,
        GMInstanceType::Global => -5,
        GMInstanceType::Local => -7,
        GMInstanceType::Stacktop => -9,
        GMInstanceType::Arg => -15,
        GMInstanceType::Static => -16,
    }
}

