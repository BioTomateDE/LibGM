use crate::deserialize::all::GMData;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::particles::{GMParticleEmitter, GMParticleEmitter2023_4, GMParticleEmitter2023_8, GMParticleEmitterPre2023_8, GMParticleSystem};
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_psys(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    // if !gm_data.general_info.is_version_at_least(2023, 2, 0, 0) {
    //     return Ok(())
    // }
    builder.align(4);
    builder.start_chunk("PSYS")?;
    builder.write_u32(1);   // PSYS version 1

    let len: usize = gm_data.particle_systems.particle_systems.len();
    builder.write_usize(len);
    for i in 0..len {
        builder.write_placeholder(GMPointer::ParticleSystem(i))?;
    }

    for (i, particle_system) in gm_data.particle_systems.particle_systems.iter().enumerate() {
        builder.resolve_pointer(GMPointer::ParticleSystem(i))?;
        build_particle_system(builder, &gm_data.general_info, particle_system)
            .map_err(|e| format!("{e} for Particle System #{i} with name \"{}\"", particle_system.name.display(&gm_data.strings)))?;
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

fn build_particle_system(builder: &mut DataBuilder, general_info: &GMGeneralInfo, particle_system: &GMParticleSystem) -> Result<(), String> {
    builder.write_gm_string(&particle_system.name)?;
    builder.write_i32(particle_system.origin_x);
    builder.write_i32(particle_system.origin_y);
    builder.write_i32(particle_system.draw_order);
    if general_info.is_version_at_least(2023, 8, 0, 0) {
        builder.write_bool32(particle_system.global_space_particles.ok_or("GameMaker Version 2023.8 but Global Space Particles not set")?);
    }
    for emitter_ref in &particle_system.emitters {
        builder.write_usize(emitter_ref.index);
    }
    Ok(())
}


pub fn build_chunk_psem(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    // if !gm_data.general_info.is_version_at_least(2023, 2, 0, 0) {
    //     return Ok(())
    // }
    builder.align(4);
    builder.start_chunk("PSEM")?;
    builder.write_u32(1);   // PSEM version 1

    let emitters: &Vec<GMParticleEmitter> = &gm_data.particle_emitters.emitters;
    builder.write_usize(emitters.len());
    for i in 0..emitters.len() {
        builder.write_placeholder(GMPointer::ParticleEmitter(i))?;
    }

    for (i, emitter) in emitters.iter().enumerate() {
        builder.resolve_pointer(GMPointer::ParticleSystem(i))?;
        build_particle_emitter(builder, &gm_data.general_info, emitter)
            .map_err(|e| format!("{e} for Particle Emitter #{i} with name \"{}\"", emitter.name.display(&gm_data.strings)))?;
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


fn build_particle_emitter(builder: &mut DataBuilder, general_info: &GMGeneralInfo, emitter: &GMParticleEmitter) -> Result<(), String> {
    builder.write_gm_string(&emitter.name)?;
    if general_info.is_version_at_least(2023, 6, 0, 0) {
        builder.write_bool32(emitter.enabled.ok_or("GameMaker Version 2023.6 but Enabled not set")?);
    }
    builder.write_i32(emitter.mode.into());
    if general_info.is_version_at_least(2023, 8, 0, 0) {
        let data_2023_8: &GMParticleEmitter2023_8 = emitter.data_2023_8.as_ref().ok_or("GameMaker Version 2023.8 but its data not set")?;
        builder.write_f32(emitter.emit_count as f32);
        builder.write_bool32(data_2023_8.emit_relative);
        builder.write_f32(data_2023_8.delay_min);
        builder.write_f32(data_2023_8.delay_max);
        builder.write_i32(data_2023_8.delay_unit.into());
        builder.write_f32(data_2023_8.interval_min);
        builder.write_f32(data_2023_8.interval_max);
        builder.write_i32(data_2023_8.interval_unit.into());
    } else {
        builder.write_u32(emitter.emit_count);
    }
    builder.write_i32(emitter.distribution.into());
    builder.write_i32(emitter.shape.into());
    builder.write_f32(emitter.region_x);
    builder.write_f32(emitter.region_y);
    builder.write_f32(emitter.region_w);
    builder.write_f32(emitter.region_h);
    builder.write_f32(emitter.rotation);
    builder.write_usize(emitter.sprite.index);
    builder.write_i32(emitter.texture.into());
    builder.write_f32(emitter.frame_index);
    if general_info.is_version_at_least(2023, 4, 0, 0) {
        let data_2023_4: &GMParticleEmitter2023_4 = emitter.data_2023_4.as_ref().ok_or("GameMaker Version 2023.4 but its data not set")?;
        builder.write_bool32(data_2023_4.animate);
        builder.write_bool32(data_2023_4.stretch);
        builder.write_bool32(data_2023_4.is_random);
    }
    builder.write_u32(emitter.start_color);
    builder.write_u32(emitter.mid_color);
    builder.write_u32(emitter.end_color);
    builder.write_bool32(emitter.additive_blend);
    builder.write_f32(emitter.lifetime_min);
    builder.write_f32(emitter.lifetime_max);
    builder.write_f32(emitter.scale_x);
    builder.write_f32(emitter.scale_y);
    if general_info.is_version_at_least(2023, 8, 0, 0) {
        let data_2023_8: &GMParticleEmitter2023_8 = emitter.data_2023_8.as_ref().ok_or("GameMaker Version 2023.8 but its data not set")?;
        builder.write_f32(data_2023_8.size_min_x);
        builder.write_f32(data_2023_8.size_max_x);
        builder.write_f32(data_2023_8.size_min_y);
        builder.write_f32(data_2023_8.size_max_y);
        builder.write_f32(data_2023_8.size_increase_x);
        builder.write_f32(data_2023_8.size_increase_y);
        builder.write_f32(data_2023_8.size_wiggle_x);
        builder.write_f32(data_2023_8.size_wiggle_y);
    } else {
        let data_pre_2023_8: &GMParticleEmitterPre2023_8 = emitter.data_pre_2023_8.as_ref().ok_or("GameMaker Version pre 2023.8 but its data not set")?;
        builder.write_f32(data_pre_2023_8.size_min);
        builder.write_f32(data_pre_2023_8.size_max);
        builder.write_f32(data_pre_2023_8.size_increase);
        builder.write_f32(data_pre_2023_8.size_wiggle);
    }
    builder.write_f32(emitter.speed_min);
    builder.write_f32(emitter.speed_max);
    builder.write_f32(emitter.speed_increase);
    builder.write_f32(emitter.speed_wiggle);
    builder.write_f32(emitter.gravity_force);
    builder.write_f32(emitter.gravity_direction);
    builder.write_f32(emitter.direction_min);
    builder.write_f32(emitter.direction_max);
    builder.write_f32(emitter.direction_increase);
    builder.write_f32(emitter.direction_wiggle);
    builder.write_f32(emitter.orientation_min);
    builder.write_f32(emitter.orientation_max);
    builder.write_f32(emitter.orientation_increase);
    builder.write_f32(emitter.orientation_wiggle);
    builder.write_bool32(emitter.orientation_relative);
    
    builder.write_usize(emitter.spawn_on_death.index);
    builder.write_u32(emitter.spawn_on_death_count);
    builder.write_usize(emitter.spawn_on_update.index);
    builder.write_u32(emitter.spawn_on_update_count);
    Ok(())
}

