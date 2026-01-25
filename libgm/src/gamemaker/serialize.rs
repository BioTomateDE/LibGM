//! Functions related to building GameMaker data files.
//!
//! Some of these functions are also re-exported at the crate root.

pub(crate) mod builder;
mod chunk;
mod lists;
mod numbers;
mod pointers;
mod resources;
pub(crate) mod traits;

use std::path::Path;

use builder::DataBuilder;

use crate::{
    gamemaker::{chunk::ChunkName, data::GMData, elements::string::GMStrings},
    prelude::*,
    util::{bench::Stopwatch, panic},
};

/// Builds a GameMaker data file and return a byte buffer.
///
/// For more information on the data file format, see [`crate::gamemaker`].
pub fn build_bytes(gm_data: &GMData) -> Result<Vec<u8>> {
    build(gm_data).context("building GameMaker data bytes")
}

/// Builds a GameMaker data file to the specified file path.
///
/// For more information on the data file format, see [`crate::gamemaker`].
pub fn build_file(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()> {
    let path: &Path = path.as_ref();
    let raw_data: Vec<u8> = build(gm_data)
        .with_context(|| format!("building GameMaker data file {}", path.display()))?;

    let stopwatch = Stopwatch::start();
    std::fs::write(path, raw_data)
        .map_err(|e| e.to_string())
        .context("writing data file")?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

fn build(gm_data: &GMData) -> Result<Vec<u8>> {
    if cfg!(feature = "catch-panic") {
        panic::catch(|| build_impl(gm_data))
    } else {
        build_impl(gm_data)
    }
}

fn build_impl(data: &GMData) -> Result<Vec<u8>> {
    let stopwatch = Stopwatch::start();
    let mut builder = DataBuilder::new(data);

    builder.write_chunk_name(ChunkName::new("FORM"));
    // Write Data length placeholder
    builder.write_u32(0xDEAD_C0DE);

    // GEN8 has to be the first chunk, at least for utmt (?).
    // CODE has to be written before VARI and FUNC.

    builder.build_chunk(&data.general_info)?;
    builder.build_chunk(&data.options)?;
    builder.build_chunk(&data.extensions)?;
    builder.build_chunk(&data.sounds)?;
    builder.build_chunk(&data.audio_groups)?;
    builder.build_chunk(&data.sprites)?;
    builder.build_chunk(&data.backgrounds)?;
    builder.build_chunk(&data.paths)?;
    builder.build_chunk(&data.scripts)?;
    builder.build_chunk(&data.shaders)?;
    builder.build_chunk(&data.fonts)?;
    builder.build_chunk(&data.timelines)?;
    builder.build_chunk(&data.game_objects)?;
    builder.build_chunk(&data.rooms)?;
    builder.build_chunk(&data.texture_page_items)?;
    builder.build_chunk(&data.codes)?;
    builder.build_chunk(&data.variables)?;
    builder.build_chunk(&data.functions)?;
    builder.build_chunk(&data.embedded_textures)?;
    builder.build_chunk(&data.audios)?;
    builder.build_chunk(&data.sequences)?;
    builder.build_chunk(&data.particle_systems)?;
    builder.build_chunk(&data.particle_emitters)?;
    builder.build_chunk(&data.language_info)?;
    builder.build_chunk(&data.global_init_scripts)?;
    builder.build_chunk(&data.game_end_scripts)?;
    builder.build_chunk(&data.root_ui_nodes)?;
    builder.build_chunk(&data.embedded_images)?;
    builder.build_chunk(&data.texture_group_infos)?;
    builder.build_chunk(&data.tags)?;
    builder.build_chunk(&data.feature_flags)?;
    builder.build_chunk(&data.filter_effects)?;
    builder.build_chunk(&data.animation_curves)?;

    builder.build_chunk(&GMStrings)?;

    builder.remove_last_chunk_padding();

    builder.connect_pointer_placeholders()?;

    // Overwrite data length placeholder
    builder.overwrite_usize(builder.len() - 8, 4)?;

    log::trace!("Building data file took {stopwatch}");

    let raw_data: Vec<u8> = builder.finish();

    if raw_data.len() >= i32::MAX as usize {
        bail!("Data file is bigger than 2,147,483,646 bytes which will lead to bugs in the runner")
    }

    Ok(raw_data)
}
