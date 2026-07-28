// SPDX-License-Identifier: GPL-3.0-only
//! Functions related to building GameMaker data files.
//!
//! Some of these functions are also re-exported at the crate root.

mod chunk;
mod lists;
mod numbers;
mod pointers;
mod resources;
mod versioning;
pub(crate) mod writer;

use std::path::Path;

use writer::DataWriter;

use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::util::unwind::catch;
use crate::wad::chunk::ChunkName;
use crate::wad::data::Endianness;
use crate::wad::data::GMData;
use crate::wad::elem::data_file::DataFiles;

/// Builds a GameMaker data file and returns a byte buffer.
///
/// If you want to build the data file directly to a file on disk, check out
/// [`build_file`].
///
/// For more information on the data file format, see [`crate::wad`].
#[inline]
pub fn save_bytes(gm_data: &GMData) -> Result<Vec<u8>> {
    serialize(gm_data).ctx("building GameMaker data bytes")
}

#[deprecated(since = "0.8.0", note = "renamed to save_bytes")]
pub fn build_bytes(gm_data: &GMData) -> Result<Vec<u8>> {
    save_bytes(gm_data)
}

/// Builds a GameMaker data file to the specified file path.
///
/// If you want to build the data file to a buffer in memory, check out
/// [`build_bytes`].
///
/// For more information on the data file format, see [`crate::wad`].
pub fn save_file(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()> {
    let path: &Path = path.as_ref();
    let raw_data: Vec<u8> = catch(|| serialize(gm_data))
        .ctx(|| format!("building GameMaker data file {}", path.display()))?;

    let stopwatch = Stopwatch::start();
    std::fs::write(path, raw_data).ctx_any("writing data file")?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

#[deprecated(since = "0.8.0", note = "renamed to save_bytes")]
pub fn build_file(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()> {
    save_file(gm_data, path)
}

fn serialize(data: &GMData) -> Result<Vec<u8>> {
    let stopwatch = Stopwatch::start();
    let mut writer = DataWriter::new(data);

    let root_chunk = match data.meta.endianness {
        Endianness::Little => b"FORM",
        Endianness::Big => b"MROF",
    };
    writer.write_bytes(root_chunk);

    // Write Data length placeholder
    writer.write_u32(0xDEAD_C0DE);

    // TODO: make sure CODE is written before VARI and FUNC!!!

    if data.meta.chunks.is_empty() {
        bail!("Data file needs at least one chunk");
    }

    for chunk_name in &data.meta.chunks {
        match chunk_name {
            ChunkName::ACRV => writer.build_chunk(&data.animation_curves),
            ChunkName::AGRP => writer.build_chunk(&data.audio_groups),
            ChunkName::AUDO => writer.build_chunk(&data.audios),
            ChunkName::BGND => writer.build_chunk(&data.tilesets),
            ChunkName::CODE => writer.build_chunk(&data.codes),
            ChunkName::DAFL => writer.build_chunk(&DataFiles),
            ChunkName::EMBI => writer.build_chunk(&data.embedded_images),
            ChunkName::EXTN => writer.build_chunk(&data.extensions),
            ChunkName::FEAT => writer.build_chunk(&data.feature_flags),
            ChunkName::FEDS => writer.build_chunk(&data.filter_effects),
            ChunkName::FONT => writer.build_chunk(&data.fonts),
            ChunkName::FUNC => writer.build_chunk(&data.functions),
            ChunkName::GEN8 => writer.build_chunk(&data.general_info),
            ChunkName::GLOB => writer.build_chunk(&data.global_init_scripts),
            ChunkName::GMEN => writer.build_chunk(&data.game_end_scripts),
            ChunkName::LANG => writer.build_chunk(&data.language_info),
            ChunkName::OBJT => writer.build_chunk(&data.game_objects),
            ChunkName::OPTN => writer.build_chunk(&data.options),
            ChunkName::PATH => writer.build_chunk(&data.paths),
            ChunkName::PSEM => writer.build_chunk(&data.particle_emitters),
            ChunkName::PSYS => writer.build_chunk(&data.particle_systems),
            ChunkName::ROOM => writer.build_chunk(&data.rooms),
            ChunkName::SCPT => writer.build_chunk(&data.scripts),
            ChunkName::SEQN => writer.build_chunk(&data.sequences),
            ChunkName::SHDR => writer.build_chunk(&data.shaders),
            ChunkName::SOND => writer.build_chunk(&data.sounds),
            ChunkName::SPRT => writer.build_chunk(&data.sprites),
            ChunkName::STRG => writer.build_chunk(&data.strings),
            ChunkName::TAGS => writer.build_chunk(&data.tags),
            ChunkName::TGIN => writer.build_chunk(&data.texture_group_infos),
            ChunkName::TMLN => writer.build_chunk(&data.timelines),
            ChunkName::TPAG => writer.build_chunk(&data.texture_page_items),
            ChunkName::TXTR => writer.build_chunk(&data.texture_pages),
            ChunkName::UILR => writer.build_chunk(&data.ui_nodes),
            ChunkName::VARI => writer.build_chunk(&data.variables),
        }?;
    }

    writer.remove_last_chunk_padding();

    writer.connect_pointer_placeholders()?;

    // Overwrite data length placeholder
    let data_length = writer.pos() - 8; // FORM is excluded
    writer.overwrite_u32(data_length, 4)?; // Pos 4 the length spec in FORM

    log::trace!("Building data file took {stopwatch}");

    let raw_data: Vec<u8> = writer.finish();

    if raw_data.len() >= i32::MAX as usize {
        bail!("Data file is bigger than 2,147,483,646 bytes which will lead to bugs in the runner")
    }

    Ok(raw_data)
}
