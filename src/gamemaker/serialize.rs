mod builder;
pub mod traits;

pub use builder::DataBuilder;
use crate::gamemaker::deserialize::GMData;
use crate::utility::Stopwatch;


pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let mut builder = DataBuilder::new(gm_data);

    builder.write_literal_string("FORM");
    builder.write_u32(0xDEADC0DE);  // data length placeholder

    builder.build_chunk("GEN8", &gm_data.general_info)?;
    builder.build_chunk("OPTN", &gm_data.options)?;
    builder.build_chunk("EXTN", &gm_data.extensions)?;
    builder.build_chunk("SOND", &gm_data.sounds)?;
    builder.build_chunk("AGRP", &gm_data.audio_groups)?;
    builder.build_chunk("SPRT", &gm_data.sprites)?;
    builder.build_chunk("BGND", &gm_data.backgrounds)?;
    builder.build_chunk("PATH", &gm_data.paths)?;
    builder.build_chunk("SCPT", &gm_data.scripts)?;
    builder.build_chunk("SHDR", &gm_data.shaders)?;
    builder.build_chunk("FONT", &gm_data.fonts)?;
    builder.build_chunk("TMLN", &gm_data.timelines)?;
    builder.build_chunk("OBJT", &gm_data.game_objects)?;
    builder.build_chunk("ROOM", &gm_data.rooms)?;
    builder.build_chunk("DAFL", &gm_data.data_files)?;
    builder.build_chunk("TPAG", &gm_data.texture_page_items)?;
    builder.build_chunk("CODE", &gm_data.codes)?;       // CODE has to be written before VARI and FUNC
    builder.build_chunk("VARI", &gm_data.variables)?;
    builder.build_chunk("FUNC", &gm_data.functions)?;
    builder.build_chunk("STRG", &gm_data.strings)?;
    builder.build_chunk("TXTR", &gm_data.embedded_textures)?;
    builder.build_chunk("AUDO", &gm_data.audios)?;

    builder.build_chunk("SEQN", &gm_data.sequences)?;
    builder.build_chunk("PSYS", &gm_data.particle_systems)?;
    builder.build_chunk("PSEM", &gm_data.particle_emitters)?;
    builder.build_chunk("LANG", &gm_data.language_info)?;
    builder.build_chunk("GLOB", &gm_data.global_init_scripts)?;
    builder.build_chunk("GMEN", &gm_data.game_end_scripts)?;
    builder.build_chunk("UILR", &gm_data.root_ui_nodes)?;
    builder.build_chunk("EMBI", &gm_data.embedded_images)?;
    builder.build_chunk("TGIN", &gm_data.texture_group_infos)?;
    builder.build_chunk("TAGS", &gm_data.tags)?;
    builder.build_chunk("FEAT", &gm_data.feature_flags)?;
    builder.build_chunk("FEDS", &gm_data.filter_effects)?;
    builder.build_chunk("ACRV", &gm_data.animation_curves)?;

    // undo padding for last chunk
    let padding_data: Vec<u8> = builder.raw_data.split_off(builder.padding_start_pos);
    if padding_data.iter().any(|i| *i != 0) {
        return Err("Built padding at end of the data contains non-null bytes; something went wrong internally".to_string())
    }

    // resolve pointers/placeholders
    let stopwatch2 = Stopwatch::start();
    for (placeholder_data_pos, element_mem_addr) in &builder.pointer_placeholder_positions {
        let resource_data_pos: u32 = *builder.pointer_resource_positions.get(element_mem_addr).ok_or_else(|| format!(
            "Could not resolve pointer placeholder with data position {} and memory address {}",
            placeholder_data_pos, element_mem_addr,
        ))?;
        // overwrite placeholder 0xDEADC0DE
        let mut_slice: &mut [u8] = builder.raw_data.get_mut(*placeholder_data_pos as usize .. *placeholder_data_pos as usize + 4)
            .ok_or_else(|| format!("Could not get 4 bytes of raw data at position {} while resolving pointer placeholders", placeholder_data_pos))?;
        mut_slice.copy_from_slice(&resource_data_pos.to_le_bytes());
    }
    log::trace!("Resolving {} pointer placeholders to {} resources took {stopwatch2}",
        builder.pointer_placeholder_positions.len(),
        builder.pointer_resource_positions.len(),
    );

    // overwrite data length placeholder
    builder.overwrite_usize(builder.len() - 8, 4)?;

    log::trace!("Building data file took {stopwatch}");
    Ok(builder.raw_data)
}

