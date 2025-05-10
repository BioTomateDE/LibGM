use crate::deserialize::all::GMData;
use crate::deserialize::scripts::GMScript;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_scpt(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "SCPT", abs_pos: data_builder.len() };
    let len: usize = gm_data.scripts.scripts_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::script(i))?;
    }

    for i in 0..len {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::script(i))?;
        let script: &GMScript = &gm_data.scripts.scripts_by_index[i];

        builder.write_gm_string(data_builder, &script.name)?;
        match script.id {
            Some(id) => builder.write_u32(id),
            None => builder.write_i32(-1),
        };
    }

    build_chunk(data_builder, builder)?;
    Ok(())
} // TODO

