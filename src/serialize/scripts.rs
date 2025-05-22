use crate::deserialize::all::GMData;
use crate::deserialize::scripts::GMScript;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_scpt(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "SCPT");
    let len: usize = gm_data.scripts.scripts_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::Script(i))?;
    }

    for i in 0..len {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::Script(i))?;
        let script: &GMScript = &gm_data.scripts.scripts_by_index[i];

        builder.write_gm_string(data_builder, &script.name)?;
        if let Some(ref code) = script.code {
            if script.is_constructor {
                builder.write_usize(code.index | 0x7FFFFFFF);
            } else {
                builder.write_usize(code.index);
            }
        }
        else {
            builder.write_i32(-1);
        }

    }

    builder.finish(data_builder)?;
    Ok(())
}

