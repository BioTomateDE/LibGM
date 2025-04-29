use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::functions::GMFunction;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_func(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "FUNC", abs_pos: data_builder.len() };
    let len: usize = gm_data.functions.functions_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::function(i))?;
    }

    for i in 0..len {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::function(i))?;
        let function: &GMFunction = &gm_data.functions.functions_by_index[i];

        builder.write_gm_string(data_builder, &function.name)?;
        builder.write_usize(function.occurrences.len());
        match function.occurrences.get(0) {
            Some(occurrence_position) => builder.write_usize(*occurrence_position),
            None => builder.write_i32(-1),
        }
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}

