use crate::deserialize::all::GMData;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_chunk_extn(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "EXTN", abs_pos: data_builder.len() };

    builder.write_usize(0);

    build_chunk(data_builder, builder)?;
    Ok(())
}

pub fn build_chunk_agrp(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "AGRP", abs_pos: data_builder.len() };

    builder.write_usize(0);

    build_chunk(data_builder, builder)?;
    Ok(())
}

pub fn build_chunk_shdr(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "SHDR", abs_pos: data_builder.len() };

    builder.write_usize(0);

    build_chunk(data_builder, builder)?;
    Ok(())
}

pub fn build_chunk_tmln(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "TMLN", abs_pos: data_builder.len() };

    builder.write_usize(0);

    build_chunk(data_builder, builder)?;
    Ok(())
}

pub fn build_chunk_dafl(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "DAFL", abs_pos: data_builder.len() };

    builder.write_usize(0);

    build_chunk(data_builder, builder)?;
    Ok(())
}
