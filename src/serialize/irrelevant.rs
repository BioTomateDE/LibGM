use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::irrelevant::{GMAudioGroup, GMExtension, GMGlobalInit};
use crate::serialize::chunk_writing::{DataBuilder, DataPlaceholder};

pub fn build_chunk_lang(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let Some(lang) = &gm_data.language_root else { return Ok(()) };
    builder.start_chunk("LANG")?;
    
    builder.write_u32(lang.unknown1);
    builder.write_usize(lang.languages.len());
    builder.write_usize(lang.entry_ids.len());
    for string in &lang.entry_ids {
        builder.write_resource_id(string);
    }
    for language in &lang.languages {
        builder.write_resource_id(&language.name);
        builder.write_resource_id(&language.region);
        for entry in &language.entries {
            builder.write_resource_id(entry);
        }
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


pub fn build_chunk_extn(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    if !gm_data.extensions.serialize {
        return Ok(())
    }
    builder.start_chunk("EXTN")?;
    let extensions: &Vec<GMExtension> = &gm_data.extensions.extensions;
    
    for i in 0..extensions.len() {
        builder.write_placeholder(DataPlaceholder::Extension(i))?;
    }
    
    for (i, extension) in extensions.iter().enumerate() {
        builder.resolve_pointer(DataPlaceholder::Extension(i))?;
        builder.write_gm_string(&extension.name)?;
        builder.write_gm_string(&extension.value)?;
        builder.write_u32(extension.kind.into());
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


pub fn build_chunk_agrp(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    if !gm_data.audio_groups.exists {
        return Ok(())
    }
    builder.start_chunk("AGRP")?;
    let audio_groups: &Vec<GMAudioGroup> = &gm_data.audio_groups.audio_groups;

    for i in 0..audio_groups.len() {
        builder.write_placeholder(DataPlaceholder::AudioGroup(i))?;
    }

    for (i, audio_group) in audio_groups.iter().enumerate() {
        builder.resolve_pointer(DataPlaceholder::AudioGroup(i))?;
        builder.write_gm_string(&audio_group.name)?;
        if gm_data.general_info.is_version_at_least(2024, 14, 0, 0) {
            let path: &GMRef<String> = &audio_group.path.ok_or_else(|| format!(
                "GameMaker Version 2024.14 but Extension Path not set for extension #{i} with name \"{}\"",
                audio_group.name.display(&gm_data.strings),
            ))?;
            builder.write_gm_string(path)?;
        }
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

pub fn build_chunk_glob(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    if !gm_data.global_inits.exists {
        return Ok(())
    }
    builder.start_chunk("GLOB")?;
    let global_inits: &Vec<GMGlobalInit> = &gm_data.global_inits.global_inits;

    for global_init in global_inits {
        builder.write_resource_id(&global_init.code);
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

