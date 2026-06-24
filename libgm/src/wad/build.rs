// SPDX-License-Identifier: GPL-3.0-only
//! Functions related to building GameMaker data files.
//!
//! Some of these functions are also re-exported at the crate root.

pub(crate) mod builder;
mod chunk;
mod lists;
mod numbers;
mod pointers;
mod resources;
mod versioning;

use std::path::Path;

use builder::DataBuilder;

use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::util::unwind;
use crate::wad::chunk::ChunkName;
use crate::wad::data::Endianness;
use crate::wad::data::GMData;

/// Builds a GameMaker data file and returns a byte buffer.
///
/// If you want to build the data file directly to a file on disk, check out
/// [`build_file`].
///
/// For more information on the data file format, see [`crate::wad`].
#[inline]
pub fn build_bytes(gm_data: &GMData) -> Result<Vec<u8>> {
    build(gm_data).ctx("building GameMaker data bytes")
}

/// Builds a GameMaker data file to the specified file path.
///
/// If you want to build the data file to a buffer in memory, check out
/// [`build_bytes`].
///
/// For more information on the data file format, see [`crate::wad`].
pub fn build_file(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()> {
    let path: &Path = path.as_ref();
    let raw_data: Vec<u8> =
        build(gm_data).ctx(|| format!("building GameMaker data file {}", path.display()))?;

    let stopwatch = Stopwatch::start();
    std::fs::write(path, raw_data).ctx_any("writing data file")?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

#[inline]
fn build(gm_data: &GMData) -> Result<Vec<u8>> {
    if cfg!(feature = "catch-panic") {
        unwind::catch(|| build_impl(gm_data))
    } else {
        build_impl(gm_data)
    }
}

fn build_impl(data: &GMData) -> Result<Vec<u8>> {
    let stopwatch = Stopwatch::start();
    let mut builder = DataBuilder::new(data);

    let root_chunk = match data.meta.endianness {
        Endianness::Little => b"FORM",
        Endianness::Big => b"MROF",
    };
    builder.write_bytes(root_chunk);

    // Write Data length placeholder
    builder.write_u32(0xDEAD_C0DE);

    // TODO: make sure CODE is written before VARI and FUNC!!!

    for &chunk_name in &data.meta.chunk_order {
        match chunk_name {
            ChunkName::ACRV => builder.build_chunk(&data.animation_curves),
            ChunkName::AGRP => builder.build_chunk(&data.audio_groups),
            ChunkName::AUDO => builder.build_chunk(&data.audios),
            ChunkName::BGND => builder.build_chunk(&data.backgrounds),
            ChunkName::CODE => builder.build_chunk(&data.codes),
            ChunkName::DAFL => builder.build_chunk(&data.data_files),
            ChunkName::EMBI => builder.build_chunk(&data.embedded_images),
            ChunkName::EXTN => builder.build_chunk(&data.extensions),
            ChunkName::FEAT => builder.build_chunk(&data.feature_flags),
            ChunkName::FEDS => builder.build_chunk(&data.filter_effects),
            ChunkName::FONT => builder.build_chunk(&data.fonts),
            ChunkName::FUNC => builder.build_chunk(&data.functions),
            ChunkName::GEN8 => builder.build_chunk(&data.general_info),
            ChunkName::GLOB => builder.build_chunk(&data.global_init_scripts),
            ChunkName::GMEN => builder.build_chunk(&data.game_end_scripts),
            ChunkName::LANG => builder.build_chunk(&data.language_info),
            ChunkName::OBJT => builder.build_chunk(&data.game_objects),
            ChunkName::OPTN => builder.build_chunk(&data.options),
            ChunkName::PATH => builder.build_chunk(&data.paths),
            ChunkName::PSEM => builder.build_chunk(&data.particle_systems),
            ChunkName::PSYS => builder.build_chunk(&data.particle_systems),
            ChunkName::ROOM => builder.build_chunk(&data.rooms),
            ChunkName::SCPT => builder.build_chunk(&data.scripts),
            ChunkName::SEQN => builder.build_chunk(&data.sequences),
            ChunkName::SHDR => builder.build_chunk(&data.shaders),
            ChunkName::SOND => builder.build_chunk(&data.sounds),
            ChunkName::SPRT => builder.build_chunk(&data.sprites),
            ChunkName::STRG => builder.build_chunk(&data.strings),
            ChunkName::TAGS => builder.build_chunk(&data.tags),
            ChunkName::TGIN => builder.build_chunk(&data.texture_group_infos),
            ChunkName::TMLN => builder.build_chunk(&data.timelines),
            ChunkName::TPAG => builder.build_chunk(&data.texture_page_items),
            ChunkName::TXTR => builder.build_chunk(&data.texture_pages),
            ChunkName::UILR => builder.build_chunk(&data.ui_nodes),
            ChunkName::VARI => builder.build_chunk(&data.variables),
        }?;
    }

    builder.remove_last_chunk_padding();

    builder.connect_pointer_placeholders()?;

    // Overwrite data length placeholder
    builder.overwrite_u32(builder.pos() - 8, 4)?;

    log::trace!("Building data file took {stopwatch}");

    let raw_data: Vec<u8> = builder.finish();

    if raw_data.len() >= i32::MAX as usize {
        bail!("Data file is bigger than 2,147,483,646 bytes which will lead to bugs in the runner")
    }

    Ok(raw_data)
}
