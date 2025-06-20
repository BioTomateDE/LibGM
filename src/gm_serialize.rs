use crate::debug_utils::Stopwatch;
use crate::gm_deserialize::GMData;
use crate::serialize_old::chunk_writing::DataBuilder;

pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let mut builder = DataBuilder::new(gm_data);
    
    builder.write_literal_string("FORM");
    builder.write_u32(0xDEADC0DE);  // data length placeholder
    
    builder.build_chunk("STRG", &gm_data.strings)?;
    builder.build_chunk("GEN8", &gm_data.general_info)?;
    builder.build_chunk("TXTR", &gm_data.embedded_textures)?;
    builder.build_chunk("TPAG", &gm_data.texture_page_items)?;
    builder.build_chunk("VARI", &gm_data.variables)?;
    builder.build_chunk("FUNC", &gm_data.functions)?;
    builder.build_chunk("SCPT", &gm_data.scripts)?;
    builder.build_chunk("CODE", &gm_data.codes)?;
    builder.build_chunk("FONT", &gm_data.fonts)?;
    builder.build_chunk("SPRT", &gm_data.sprites)?;
    builder.build_chunk("OBJT", &gm_data.game_objects)?;
    builder.build_chunk("ROOM", &gm_data.rooms)?;
    builder.build_chunk("BGND", &gm_data.backgrounds)?;
    builder.build_chunk("PATH", &gm_data.paths)?;
    builder.build_chunk("AUDO", &gm_data.audios)?;
    builder.build_chunk("SOND", &gm_data.sounds)?;
    
    builder.build_chunk("PSYS", &gm_data.particle_systems)?;
    builder.build_chunk("PSEM", &gm_data.particle_emitters)?;
    builder.build_chunk("LANG", &gm_data.language_info)?;
    builder.build_chunk("EXTN", &gm_data.extensions)?;
    builder.build_chunk("AGRP", &gm_data.audio_groups)?;
    builder.build_chunk("GLOB", &gm_data.global_init_scripts)?;
    builder.build_chunk("GMEN", &gm_data.game_end_scripts)?;
    
    builder.overwrite_usize(builder.len() - 8, 4)?;   // overwrite data length placeholder
    log::trace!("Building data file took {stopwatch}");
    Ok(builder.raw_data)
}


