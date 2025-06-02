use crate::debug_utils::DurationExt;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::deserialize::all::GMData;
use crate::serialize::backgrounds::build_chunk_bgnd;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};
use crate::serialize::code::build_chunk_code;
use crate::serialize::embedded_audio::build_chunk_audo;
use crate::serialize::embedded_textures::build_chunk_txtr;
use crate::serialize::fonts::build_chunk_font;
use crate::serialize::functions::build_chunk_func;
use crate::serialize::game_objects::build_chunk_objt;
use crate::serialize::strings::build_chunk_strg;
use crate::serialize::general_info::{build_chunk_optn, build_chunk_gen8};
use crate::serialize::paths::build_chunk_path;
use crate::serialize::rooms::build_chunk_room;
use crate::serialize::scripts::build_chunk_scpt;
use crate::serialize::sounds::build_chunk_sond;
use crate::serialize::sprites::build_chunk_sprt;
use crate::serialize::stubs::{build_chunk_agrp, build_chunk_dafl, build_chunk_extn, build_chunk_shdr, build_chunk_tmln};
use crate::serialize::texture_page_items::build_chunk_tpag;
use crate::serialize::variables::build_chunk_vari;
use crate::trace_build;


pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>, String> {
    let mut builder = DataBuilder {
        raw_data: Vec::new(),
        chunk_start_pos: None,
        pool_placeholders: HashMap::new(),
        placeholder_pool_resources: HashMap::new(),
    };

    builder.write_literal_string("FORM");
    builder.write_placeholder(GMPointer::FormLength)?;

    // same chunk order as in undertale 1.01
    trace_build!("GEN8", build_chunk_gen8(&mut builder, &gm_data)?);
    trace_build!("OPTN", build_chunk_optn(&mut builder, &gm_data)?);
    trace_build!("EXTN", build_chunk_extn(&mut builder, &gm_data)?);      // stub
    trace_build!("SOND", build_chunk_sond(&mut builder, &gm_data)?);
    trace_build!("AGRP", build_chunk_agrp(&mut builder, &gm_data)?);      // stub
    trace_build!("SPRT", build_chunk_sprt(&mut builder, &gm_data)?);
    trace_build!("BGND", build_chunk_bgnd(&mut builder, &gm_data)?);
    trace_build!("PATH", build_chunk_path(&mut builder, &gm_data)?);
    trace_build!("SCPT", build_chunk_scpt(&mut builder, &gm_data)?);
    trace_build!("SHDR", build_chunk_shdr(&mut builder, &gm_data)?);      // stub
    trace_build!("FONT", build_chunk_font(&mut builder, &gm_data)?);
    trace_build!("TMLN", build_chunk_tmln(&mut builder, &gm_data)?);      // stub
    trace_build!("OBJT", build_chunk_objt(&mut builder, &gm_data)?);
    trace_build!("ROOM", build_chunk_room(&mut builder, &gm_data)?);
    trace_build!("DAFL", build_chunk_dafl(&mut builder, &gm_data)?);      // stub
    trace_build!("TPAG", build_chunk_tpag(&mut builder, &gm_data)?);
    let (variable_occurrences_map, function_occurrences_map): (HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>) = trace_build!("CODE", build_chunk_code(&mut builder, &gm_data)?);
    trace_build!("VARI", build_chunk_vari(&mut builder, &gm_data, variable_occurrences_map)?);
    trace_build!("FUNC", build_chunk_func(&mut builder, &gm_data, function_occurrences_map)?);
    trace_build!("STRG", build_chunk_strg(&mut builder, &gm_data)?);
    trace_build!("TXTR", build_chunk_txtr(&mut builder, &gm_data)?);
    trace_build!("AUDO", build_chunk_audo(&mut builder, &gm_data)?);
    
    builder.resolve_placeholder(GMPointer::FormLength, builder.len() as i32)?;
    
    let t_start = cpu_time::ProcessTime::now();
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
                    "Could not overwrite {} bytes at position {} while resolving pointer placeholders",
                    raw.len(),
                    placeholder_position + i,
                ))?;
            *source_byte = *byte;
        }
    }
    
    log::trace!("Resolving {} pointers took {}", builder.pool_placeholders.len(), t_start.elapsed().ms());

    Ok(builder.raw_data)
}


pub fn write_data_file(data_file_path: &Path, raw_data: &[u8]) -> Result<(), String> {
    fs::write(data_file_path, raw_data)
        .map_err(|e| format!("Could not write data file to location \"{}\": {e}", data_file_path.display()))
}

