use crate::deserialize::all::GMData;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_chunk_extn(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "EXTN");

    builder.write_usize(0);

    builder.finish(data_builder)?;
    Ok(())
}

pub fn build_chunk_agrp(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "AGRP");

    builder.write_usize(0);

    builder.finish(data_builder)?;
    Ok(())
}

pub fn build_chunk_shdr(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "SHDR");

    builder.write_usize(0);

    builder.finish(data_builder)?;
    Ok(())
}

pub fn build_chunk_tmln(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "TMLN");

    builder.write_usize(0);

    builder.finish(data_builder)?;
    Ok(())
}

pub fn build_chunk_dafl(data_builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "DAFL");

    builder.write_usize(0);

    builder.finish(data_builder)?;
    Ok(())
}
