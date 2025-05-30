use crate::deserialize::all::GMData;
use crate::serialize::chunk_writing::DataBuilder;

pub fn build_chunk_extn(builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("EXTN")?;

    builder.write_usize(0);

    builder.finish_chunk()?;
    Ok(())
}

pub fn build_chunk_agrp(builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("AGRP")?;

    builder.write_usize(0);

    builder.finish_chunk()?;
    Ok(())
}

pub fn build_chunk_shdr(builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("SHDR")?;

    builder.write_usize(0);

    builder.finish_chunk()?;
    Ok(())
}

pub fn build_chunk_tmln(builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("TMLN")?;

    builder.write_usize(0);

    builder.finish_chunk()?;
    Ok(())
}

pub fn build_chunk_dafl(builder: &mut DataBuilder, _gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("DAFL")?;

    builder.write_usize(0);

    builder.finish_chunk()?;
    Ok(())
}
