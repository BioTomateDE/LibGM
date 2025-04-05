use crate::deserialize::all::UTData;
use crate::deserialize::scripts::{UTScript, UTScriptRef};
use crate::serialize::all::{DataBuilder, UTRef};
use crate::serialize::chunk_writing::ChunkBuilder;

#[allow(non_snake_case)]
pub fn build_chunk_SCPT(data_builder: &mut DataBuilder, ut_data: &UTData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "SCPT", abs_pos: data_builder.len() };
    let len: usize = ut_data.scripts.len();
    builder.write_usize(len)?;

    for i in 0..len {
        data_builder.push_pointer_position(&mut builder, UTRef::Script(UTScriptRef { index: i }))?;
    }

    for i in 0..len {
        data_builder.push_pointing_to(&mut builder, UTRef::Script(UTScriptRef { index: i }))?;
        let script: UTScriptRef = ut_data.scripts.get_script_by_index(i).expect("Script out of bounds while building.");
        let script: &UTScript = script.resolve(&ut_data.scripts)?;

        builder.write_ut_string(&script.name, &ut_data.strings)?;
        match script.id {
            Some(id) => builder.write_u32(id)?,
            None => builder.write_i32(-1)?,
        };
    }

    Ok(())
}

