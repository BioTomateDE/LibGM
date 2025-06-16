use crate::debug_utils::Stopwatch;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::deserialize::all::GMData;
use crate::serialize::backgrounds::build_chunk_bgnd;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};
use crate::serialize::code::{build_chunk_code, Occurrences};
use crate::serialize::embedded_audio::build_chunk_audo;
use crate::serialize::embedded_textures::build_chunk_txtr;
use crate::serialize::fonts::build_chunk_font;
use crate::serialize::functions::build_chunk_func;
use crate::serialize::game_objects::build_chunk_objt;
use crate::serialize::strings::build_chunk_strg;
use crate::serialize::general_info::build_chunk_gen8;
use crate::serialize::paths::build_chunk_path;
use crate::serialize::rooms::build_chunk_room;
use crate::serialize::scripts::build_chunk_scpt;
use crate::serialize::sounds::build_chunk_sond;
use crate::serialize::sprites::build_chunk_sprt;
use crate::serialize::stubs::{build_chunk_agrp, build_chunk_dafl, build_chunk_extn, build_chunk_shdr, build_chunk_tmln};
use crate::serialize::texture_page_items::build_chunk_tpag;
use crate::serialize::variables::build_chunk_vari;
use crate::bench_build;
use crate::serialize::options::build_chunk_optn;
use crate::serialize::particles::{build_chunk_psem, build_chunk_psys};

pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>, String> {
    let stopwatch_all = Stopwatch::start();
    let mut builder = DataBuilder {
        raw_data: Vec::new(),
        chunk_start_pos: None,
        pool_placeholders: HashMap::new(),
        placeholder_pool_resources: HashMap::new(),
    };

    builder.write_literal_string("FORM");
    builder.write_placeholder(GMPointer::FormLength)?;

    // same chunk order as in undertale 1.01
    bench_build!("GEN8", build_chunk_gen8(&mut builder, &gm_data)?);
    bench_build!("OPTN", build_chunk_optn(&mut builder, &gm_data)?);
    bench_build!("EXTN", build_chunk_extn(&mut builder, &gm_data)?);      // stub
    bench_build!("SOND", build_chunk_sond(&mut builder, &gm_data)?);
    bench_build!("AGRP", build_chunk_agrp(&mut builder, &gm_data)?);      // stub
    bench_build!("SPRT", build_chunk_sprt(&mut builder, &gm_data)?);
    bench_build!("BGND", build_chunk_bgnd(&mut builder, &gm_data)?);
    bench_build!("PATH", build_chunk_path(&mut builder, &gm_data)?);
    bench_build!("SCPT", build_chunk_scpt(&mut builder, &gm_data)?);
    bench_build!("SHDR", build_chunk_shdr(&mut builder, &gm_data)?);      // stub
    bench_build!("FONT", build_chunk_font(&mut builder, &gm_data)?);
    bench_build!("TMLN", build_chunk_tmln(&mut builder, &gm_data)?);      // stub
    bench_build!("OBJT", build_chunk_objt(&mut builder, &gm_data)?);
    bench_build!("ROOM", build_chunk_room(&mut builder, &gm_data)?);
    bench_build!("DAFL", build_chunk_dafl(&mut builder, &gm_data)?);      // stub
    bench_build!("TPAG", build_chunk_tpag(&mut builder, &gm_data)?);
    let (variable_occurrences_map, function_occurrences_map): (Occurrences, Occurrences) = bench_build!("CODE", build_chunk_code(&mut builder, &gm_data)?);
    bench_build!("VARI", build_chunk_vari(&mut builder, &gm_data, variable_occurrences_map)?);
    bench_build!("FUNC", build_chunk_func(&mut builder, &gm_data, function_occurrences_map)?);
    bench_build!("STRG", build_chunk_strg(&mut builder, &gm_data)?);
    bench_build!("TXTR", build_chunk_txtr(&mut builder, &gm_data)?);
    bench_build!("AUDO", build_chunk_audo(&mut builder, &gm_data)?);
    if gm_data.general_info.is_version_at_least(2023, 2, 0, 0) {
        bench_build!("PSYS", build_chunk_psys(&mut builder, &gm_data)?);
        bench_build!("PSEM", build_chunk_psem(&mut builder, &gm_data)?);
    }
    
    let raw_data_len: i32 = builder.len() as i32 - 8;
    builder.resolve_placeholder(GMPointer::FormLength, raw_data_len)?;
    
    let stopwatch_placeholders = Stopwatch::start();
    // resolve pointer placeholders
    for (placeholder_position, pointer) in &builder.pool_placeholders {
        let resource_data: i32 = *builder.placeholder_pool_resources.get(&pointer)
            .ok_or_else(|| format!(
                "Could not resolve resource {:?} for placeholder position {}",
                pointer, placeholder_position,
            ))?;

        let raw: &[u8; 4] = &(resource_data as u32).to_le_bytes();
        for (i, byte) in raw.iter().enumerate() {
            let source_byte: &mut u8 = builder.raw_data.get_mut(placeholder_position + i)
                .ok_or_else(|| format!(
                    "Could not overwrite {} bytes at position {} while resolving placeholders",
                    raw.len(), placeholder_position + i,
                ))?;
            *source_byte = *byte;
        }
    }
    
    log::trace!("Resolving {} pointers took {stopwatch_placeholders}", builder.pool_placeholders.len());
    log::trace!("Building data took {stopwatch_all}");

    Ok(builder.raw_data)
}


pub fn write_data_file(data_file_path: &Path, raw_data: &[u8]) -> Result<(), String> {
    let stopwatch = Stopwatch::start();
    fs::write(data_file_path, raw_data)
        .map_err(|e| format!("Could not write data file to location \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

