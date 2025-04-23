use crate::deserialize::all::GMData;
use crate::deserialize::scripts::{GMScript, GMScriptRef};
use crate::serialize::all::{DataBuilder, GMRef};
use crate::serialize::chunk_writing::ChunkBuilder;

#[allow(non_snake_case)]
pub fn build_chunk_SCPT(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "SCPT", abs_pos: data_builder.len() };
    let len: usize = gm_data.scripts.len();
    builder.write_usize(len)?;

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, GMRef::Script(GMScriptRef { index: i }))?;
    }

    for i in 0..len {
        data_builder.push_pointing_to(&mut builder, GMRef::Script(GMScriptRef { index: i }))?;
        let script: GMScriptRef = gm_data.scripts.get_script_by_index(i).expect("Script out of bounds while building.");
        let script: &GMScript = script.resolve(&gm_data.scripts)?;

        builder.write_gm_string(&script.name, &gm_data.strings)?;
        match script.id {
            Some(id) => builder.write_u32(id)?,
            None => builder.write_i32(-1)?,
        };
    }

    Ok(())
}

