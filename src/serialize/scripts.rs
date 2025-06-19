use crate::deserialize::all::GMData;
use crate::deserialize::scripts::GMScript;
use crate::serialize::chunk_writing::{DataBuilder, DataPlaceholder};

pub fn build_chunk_scpt(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("SCPT")?;
    let len: usize = gm_data.scripts.scripts.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(DataPlaceholder::Script(i))?;
    }

    for i in 0..len {
        builder.resolve_pointer(DataPlaceholder::Script(i))?;
        let script: &GMScript = &gm_data.scripts.scripts[i];

        builder.write_gm_string(&script.name)?;
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

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

