use crate::prelude::*;
mod builder;
pub mod traits;
mod numbers;
mod lists;
mod resources;

pub use builder::DataBuilder;
use crate::gamemaker::data::{Endianness, GMData};
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


pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>> {
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

    // Resolve pointers/placeholders
    let placeholder_count = builder.pointer_placeholder_positions.len();
    let resource_count = builder.pointer_resource_positions.len();
    let stopwatch2 = Stopwatch::start();

    for (placeholder_data_pos, element_mem_addr) in std::mem::take(&mut builder.pointer_placeholder_positions) {
        let resource_data_pos: u32 = *builder.pointer_resource_positions.get(&element_mem_addr).ok_or_else(|| format!(
            "Could not resolve pointer placeholder with data position {} and memory address {}",
            placeholder_data_pos, element_mem_addr,
        ))?;
        // overwrite placeholder 0xDEADC0DE
        builder.overwrite_i32(resource_data_pos as i32, placeholder_data_pos as usize)?;
    }
    log::trace!("Resolving {placeholder_count} pointer placeholders to {resource_count} resources took {stopwatch2}");

    // Overwrite data length placeholder
    builder.overwrite_usize(builder.len() - 8, 4)?;

    log::trace!("Building data file took {stopwatch}");
    
    if builder.raw_data.len() >= i32::MAX as usize {
        bail!("Data file is bigger than 2,147,483,647 bytes which will lead to bugs in the runner")
    }
    
    Ok(builder.raw_data)
}

