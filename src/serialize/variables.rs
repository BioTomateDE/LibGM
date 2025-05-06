use crate::deserialize::all::GMData;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_vari(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "VARI", abs_pos: data_builder.len() };

    let variable_count: usize = gm_data.variables.variables.len();
    builder.write_usize(variable_count);

    for i in 0..variable_count {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::variable(i))?;
    }

    for (i, variable) in gm_data.variables.variables.iter().enumerate() {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::variable(i))?;
        // TODO
    }


    build_chunk(data_builder, builder)?;
    Ok(())
}

