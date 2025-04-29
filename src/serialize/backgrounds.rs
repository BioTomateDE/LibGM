use crate::deserialize::all::GMData;
use crate::deserialize::backgrounds::GMBackground;
use crate::deserialize::chunk_reading::GMRef;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_chunk_bgnd(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "BGND", abs_pos: data_builder.len() };
    let len: usize = gm_data.backgrounds.backgrounds_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMRef::background(i))?;
    }

    for i in 0..len {
        data_builder.push_pointer_resolve(&mut builder, GMRef::background(i))?;
        let background: &GMBackground = &gm_data.backgrounds.backgrounds_by_index[i];

        builder.write_gm_string(data_builder, &background.name)?;
        builder.write_bool(background.transparent);
        builder.write_bool(background.smooth);
        builder.write_bool(background.preload);
        data_builder.push_pointer_placeholder(&mut builder, background.texture.clone())?;
        if gm_data.general_info.is_version_at_least(2, 0, 0, 0) {
            todo!() // TODO
        }
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}


