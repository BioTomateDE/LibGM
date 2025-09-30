mod builder;
pub mod traits;
mod numbers;
mod lists;
mod resources;

pub use builder::DataBuilder;
use crate::gamemaker::data::GMData;
use crate::gamemaker::elements::GMChunkElement;
use crate::utility::Stopwatch;



macro_rules! build_chunks {
    ($builder:expr, $gm_data:expr, $(($name:literal, $field:ident)),* $(,)?) => {{
        // First pass: check what exists
        let mut existing_count = 0;
        $(
            if $gm_data.$field.exists() {
                existing_count += 1;
            }
        )*

        // Second pass: build chunks
        let mut chunk_index = 0;
        $(
            #[allow(unused_assignments)]    // either the code is buggy or the compiler is just stupid
            if $gm_data.$field.exists() {
                let is_last = chunk_index == existing_count - 1;
                $builder.build_chunk($name, &$gm_data.$field, is_last)?;
                chunk_index += 1;
            }
        )*
    }};
}


pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let mut builder = DataBuilder::new(gm_data);

    builder.write_chunk_name("FORM")?;
    builder.write_u32(0xDEADC0DE);  // data length placeholder

    // Write chunks
    build_chunks!(builder, gm_data,
        ("GEN8", general_info),     // GEN8 has to be the first chunk, at least for utmt
        ("OPTN", options),
        ("EXTN", extensions),
        ("SOND", sounds),
        ("AGRP", audio_groups),
        ("SPRT", sprites),
        ("BGND", backgrounds),
        ("PATH", paths),
        ("SCPT", scripts),
        ("SHDR", shaders),
        ("FONT", fonts),
        ("TMLN", timelines),
        ("OBJT", game_objects),
        ("ROOM", rooms),
        ("DAFL", data_files),
        ("TPAG", texture_page_items),
        ("CODE", codes),            // CODE has to be written before VARI and FUNC
        ("VARI", variables),
        ("FUNC", functions),
        ("STRG", strings),
        ("TXTR", embedded_textures),
        ("AUDO", audios),
        ("SEQN", sequences),
        ("PSYS", particle_systems),
        ("PSEM", particle_emitters),
        ("LANG", language_info),
        ("GLOB", global_init_scripts),
        ("GMEN", game_end_scripts),
        ("UILR", root_ui_nodes),
        ("EMBI", embedded_images),
        ("TGIN", texture_group_infos),
        ("TAGS", tags),
        ("FEAT", feature_flags),
        ("FEDS", filter_effects),
        ("ACRV", animation_curves),
    );

    // resolve pointers/placeholders
    let stopwatch2 = Stopwatch::start();
    for (placeholder_data_pos, element_mem_addr) in &builder.pointer_placeholder_positions {
        let resource_data_pos: u32 = *builder.pointer_resource_positions.get(element_mem_addr).ok_or_else(|| format!(
            "Could not resolve pointer placeholder with data position {} and memory address {}",
            placeholder_data_pos, element_mem_addr,
        ))?;
        // overwrite placeholder 0xDEADC0DE
        let resource_data: [u8; 4] = if builder.gm_data.is_big_endian {
            resource_data_pos.to_be_bytes()
        } else {
            resource_data_pos.to_le_bytes()
        };
        let mut_slice: &mut [u8] = builder.raw_data.get_mut(*placeholder_data_pos as usize .. *placeholder_data_pos as usize + 4)
            .ok_or_else(|| format!("Could not get 4 bytes of raw data at position {} while resolving pointer placeholders", placeholder_data_pos))?;
        mut_slice.copy_from_slice(&resource_data);
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

