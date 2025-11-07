use crate::prelude::*;
use std::path::Path;
pub(crate) mod builder;
mod lists;
mod numbers;
mod resources;
pub(crate) mod traits;

use crate::gamemaker::data::GMData;
use crate::util::bench::Stopwatch;
use builder::DataBuilder;

pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>> {
    let stopwatch = Stopwatch::start();
    let mut builder = DataBuilder::new(gm_data);

    builder.write_chunk_name("FORM")?;
    // Write Data length placeholder
    builder.write_u32(0xDEADC0DE);

    // GEN8 has to be the first chunk, at least for utmt (?).
    // CODE has to be written before VARI and FUNC.
    builder.build_chunk(&gm_data.general_info)?;
    builder.build_chunk(&gm_data.options)?;
    builder.build_chunk(&gm_data.extensions)?;
    builder.build_chunk(&gm_data.sounds)?;
    builder.build_chunk(&gm_data.audio_groups)?;
    builder.build_chunk(&gm_data.sprites)?;
    builder.build_chunk(&gm_data.backgrounds)?;
    builder.build_chunk(&gm_data.paths)?;
    builder.build_chunk(&gm_data.scripts)?;
    builder.build_chunk(&gm_data.shaders)?;
    builder.build_chunk(&gm_data.fonts)?;
    builder.build_chunk(&gm_data.timelines)?;
    builder.build_chunk(&gm_data.game_objects)?;
    builder.build_chunk(&gm_data.rooms)?;
    builder.build_chunk(&gm_data.texture_page_items)?;
    builder.build_chunk(&gm_data.codes)?;
    builder.build_chunk(&gm_data.variables)?;
    builder.build_chunk(&gm_data.functions)?;
    builder.build_chunk(&gm_data.strings)?;
    builder.build_chunk(&gm_data.embedded_textures)?;
    builder.build_chunk(&gm_data.audios)?;
    builder.build_chunk(&gm_data.sequences)?;
    builder.build_chunk(&gm_data.particle_systems)?;
    builder.build_chunk(&gm_data.particle_emitters)?;
    builder.build_chunk(&gm_data.language_info)?;
    builder.build_chunk(&gm_data.global_init_scripts)?;
    builder.build_chunk(&gm_data.game_end_scripts)?;
    builder.build_chunk(&gm_data.root_ui_nodes)?;
    builder.build_chunk(&gm_data.embedded_images)?;
    builder.build_chunk(&gm_data.texture_group_infos)?;
    builder.build_chunk(&gm_data.tags)?;
    builder.build_chunk(&gm_data.feature_flags)?;
    builder.build_chunk(&gm_data.filter_effects)?;
    builder.build_chunk(&gm_data.animation_curves)?;

    // Remove potential padding from last chunk
    builder.raw_data.truncate(builder.last_chunk.padding_start_pos);
    builder.overwrite_usize(builder.last_chunk.padding_start_pos, builder.last_chunk.length_pos)?;

    // Resolve pointers/placeholders
    let placeholder_count = builder.pointer_placeholder_positions.len();
    let resource_count = builder.pointer_resource_positions.len();
    let stopwatch2 = Stopwatch::start();

    for (placeholder_data_pos, element_mem_addr) in std::mem::take(&mut builder.pointer_placeholder_positions) {
        let resource_data_pos: u32 = *builder
            .pointer_resource_positions
            .get(&element_mem_addr)
            .ok_or_else(|| {
                format!(
                    "Could not resolve pointer placeholder with data position {} and memory address {}",
                    placeholder_data_pos, element_mem_addr,
                )
            })?;
        // Overwrite placeholder 0xDEADC0DE
        builder.overwrite_i32(resource_data_pos as i32, placeholder_data_pos as usize)?;
    }
    log::trace!("Resolving {placeholder_count} pointer placeholders to {resource_count} resources took {stopwatch2}");

    // Overwrite data length placeholder
    builder.overwrite_usize(builder.len() - 8, 4)?;

    log::trace!("Building data file took {stopwatch}");

    if builder.raw_data.len() >= i32::MAX as usize {
        bail!("Data file is bigger than 2,147,483,646 bytes which will lead to bugs in the runner")
    }

    Ok(builder.raw_data)
}

pub fn write_data_file(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()> {
    let raw_data: Vec<u8> = build_data_file(gm_data).context("building data")?;
    let stopwatch = Stopwatch::start();
    std::fs::write(path, raw_data).context("writing data file")?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}
