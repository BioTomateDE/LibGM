use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::sprites::GMSprite;
use crate::deserialize::strings::GMStrings;


#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleSystem {
    name: GMRef<String>,
    origin_x: i32,
    origin_y: i32,
    draw_order: i32,
    global_space_particles: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct GMParticleSystems {
    pub particle_systems: Vec<GMParticleSystem>,
}


#[derive(Debug, Clone)]
pub struct GMParticleEmitter {
    pub name: GMRef<String>,
    pub enabled: Option<bool>,
    pub mode: EmitMode,
    pub emit_count: u32,
    pub data_2023_8: Option<GMParticleEmitter2023_8>,
    pub distribution: EmitterDistribution,
    pub shape: EmitterShape,
    pub region_x: f32,
    pub region_y: f32,
    pub region_w: f32,
    pub region_h: f32,
    pub rotation: f32,
    pub sprite: GMRef<GMSprite>,
    pub texture: EmitterTexture,
    pub frame_index: f32,
    pub data_2023_4: Option<GMParticleEmitter2023_4>,
    pub start_color: u32,
    pub mid_color: u32,
    pub end_color: u32,
    pub additive_blend: bool,
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub speed_increase: f32,
    pub speed_wiggle: f32,
    pub gravity_force: f32,
    pub gravity_direction: f32,
    pub direction_min: f32,
    pub direction_max: f32,
    pub direction_increase: f32,
    pub direction_wiggle: f32,
    pub orientation_min: f32,
    pub orientation_max: f32,
    pub orientation_increase: f32,
    pub orientation_wiggle: f32,
    pub orientation_relative: bool,
    pub spawn_on_death: GMRef<Self>,
    pub spawn_on_death_count: u32,
    pub spawn_on_update: GMRef<Self>,
    pub spawn_on_update_count: u32,
}

#[derive(Debug, Clone)]
pub struct GMParticleEmitter2023_4 {
    pub animate: bool,
    pub stretch: bool,
    pub is_random: bool,
}

#[derive(Debug, Clone)]
pub struct GMParticleEmitter2023_8 {
    pub emit_relative: bool,
    pub delay_min: f32,
    pub delay_max: f32,
    pub delay_unit: TimeUnit,
    pub interval_min: f32,
    pub interval_max: f32,
    pub interval_unit: TimeUnit,
    pub size_min_x: f32,
    pub size_max_x: f32,
    pub size_min_y: f32,
    pub size_max_y: f32,
    pub size_increase_x: f32,
    pub size_increase_y: f32,
    pub size_wiggle_x: f32,
    pub size_wiggle_y: f32,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum EmitMode {
    Stream = 0,
    Burst = 1,
}
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum TimeUnit {
    Seconds = 0,
    Frames = 1,
}
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum EmitterDistribution {
    Linear = 0,
    Gaussian = 1,
    InverseGaussian = 2,
}
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum EmitterShape {
    Rectangle = 0,
    Ellipse = 1,
    Diamond = 2,
    Line = 3,
}
#[repr(i32)]
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum EmitterTexture {
    None = -1,
    Pixel = 0,
    Disk = 1,
    Square = 2,
    Line = 3,
    Star = 4,
    Circle = 5,
    Ring = 6,
    Sphere = 7,
    Flare = 8,
    Spark = 9,
    Explosion = 10,
    Cloud = 11,
    Smoke = 12,
    Snow = 13,
}

struct TempParticleEmitter2023_8 {
    emit_relative: bool,
    delay_min: f32,
    delay_max: f32,
    delay_unit: TimeUnit,
    interval_min: f32,
    interval_max: f32,
    interval_unit: TimeUnit,
}

#[derive(Debug, Clone)]
pub struct GMParticleEmitters {
    pub emitters: Vec<GMParticleEmitter>,
}


pub fn parse_chunk_psys(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMParticleSystems, String> {
    chunk.cur_pos = 0;
    chunk.align(4)?;

    let count: usize = chunk.read_usize_count()?;
    let mut starting_positions: Vec<usize> = Vec::with_capacity(count);
    for _ in 0..count {
        starting_positions.push(chunk.read_usize_pos()? - chunk.abs_pos);
    }

    let mut particle_systems: Vec<GMParticleSystem> = Vec::with_capacity(count);
    for start_position in starting_positions {
        chunk.cur_pos = start_position;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let origin_x: i32 = chunk.read_i32()?;
        let origin_y: i32 = chunk.read_i32()?;
        let draw_order: i32 = chunk.read_i32()?;
        let global_space_particles: Option<bool> = if general_info.is_version_at_least(2023, 8, 0, 0) {
            Some(chunk.read_bool32()?)
        } else { None };
        let emitter_count: usize = chunk.read_usize_count()?;
        let mut emitters: Vec<GMRef<GMParticleEmitter>> = Vec::with_capacity(emitter_count);
        for _ in 0..emitter_count {
            emitters.push(GMRef::new(chunk.read_usize_count()?));
        }
        particle_systems.push(GMParticleSystem {
            name,
            origin_x,
            origin_y,
            draw_order,
            global_space_particles,
        });
    }

    Ok(GMParticleSystems { particle_systems })
}


pub fn parse_chunk_psem(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMParticleEmitters, String> {
    chunk.cur_pos = 0;
    chunk.align(4)?;

    let count: usize = chunk.read_usize_count()?;
    let mut starting_positions: Vec<usize> = Vec::with_capacity(count);
    for _ in 0..count {
        starting_positions.push(chunk.read_usize_pos()? - chunk.abs_pos);
    }

    let mut emitters: Vec<GMParticleEmitter> = Vec::with_capacity(count);
    for (i, start_position) in starting_positions.iter().enumerate() {
        chunk.cur_pos = *start_position;
        let emitter: GMParticleEmitter = parse_particle_emitter(chunk, general_info, strings)
            .map_err(|e| format!("{e} for Particle System Emitter #{i} at position {start_position}"))?;
        emitters.push(emitter);
    }

    Ok(GMParticleEmitters { emitters })
}

fn parse_particle_emitter(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMParticleEmitter, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let enabled: Option<bool> = if general_info.is_version_at_least(2023, 6, 0, 0) {
        Some(chunk.read_bool32()?)
    } else { None };
    let mode: i32 = chunk.read_i32()?;
    let mode: EmitMode = mode.try_into().map_err(|_| format!("Invalid Emit Mode {mode} (0x{mode:04X})"))?;

    let emit_count: u32;
    let data_2023_8: Option<TempParticleEmitter2023_8> = if general_info.is_version_at_least(2023, 8, 0, 0) {
        emit_count = chunk.read_f32()? as u32;              // don't see how a float is a count but ok
        let emit_relative: bool = chunk.read_bool32()?;     // always zero
        let delay_min: f32 = chunk.read_f32()?;
        let delay_max: f32 = chunk.read_f32()?;
        let delay_unit: i32 = chunk.read_i32()?;
        let delay_unit: TimeUnit = delay_unit.try_into().map_err(|_| format!("Invalid Time Unit for delay: {delay_unit} (0x{delay_unit:04X})"))?;
        let interval_min: f32 = chunk.read_f32()?;
        let interval_max: f32 = chunk.read_f32()?;
        let interval_unit: i32 = chunk.read_i32()?;
        let interval_unit: TimeUnit = interval_unit.try_into()
            .map_err(|_| format!("Invalid Time Unit for interval: {interval_unit} (0x{interval_unit:04X})"))?;

        Some(TempParticleEmitter2023_8 {
            emit_relative,
            delay_min,
            delay_max,
            delay_unit,
            interval_min,
            interval_max,
            interval_unit,
        })
    } else {
        emit_count = chunk.read_u32()?;
        None
    };

    let distribution: i32 = chunk.read_i32()?;
    let distribution: EmitterDistribution = distribution.try_into()
        .map_err(|_| format!("Invalid Emitter Distribution {distribution} (0x{distribution:04X})"))?;

    let shape: i32 = chunk.read_i32()?;
    let shape: EmitterShape = shape.try_into().map_err(|_| format!("Invalid Emitter Shape {shape} (0x{shape:04X})"))?;

    let region_x: f32 = chunk.read_f32()?;
    let region_y: f32 = chunk.read_f32()?;
    let region_w: f32 = chunk.read_f32()?;
    let region_h: f32 = chunk.read_f32()?;
    let rotation: f32 = chunk.read_f32()?;
    let sprite: GMRef<GMSprite> = GMRef::new(chunk.read_usize_count()?);    // TODO probably optional

    let texture: i32 = chunk.read_i32()?;
    let texture: EmitterTexture = texture.try_into()
        .map_err(|_| format!("Invalid Emitter Texture {texture} (0x{texture:04X})"))?;

    let frame_index: f32 = chunk.read_f32()?;

    let data_2023_4: Option<GMParticleEmitter2023_4> = if general_info.is_version_at_least(2023, 4, 0, 0) {
        let animate: bool = chunk.read_bool32()?;
        let stretch: bool = chunk.read_bool32()?;
        let is_random: bool = chunk.read_bool32()?;
        Some(GMParticleEmitter2023_4 { animate, stretch, is_random })
    } else { None };

    let start_color: u32 = chunk.read_u32()?;
    let mid_color: u32 = chunk.read_u32()?;
    let end_color: u32 = chunk.read_u32()?;
    let additive_blend: bool = chunk.read_bool32()?;
    let lifetime_min: f32 = chunk.read_f32()?;
    let lifetime_max: f32 = chunk.read_f32()?;
    let scale_x: f32 = chunk.read_f32()?;
    let scale_y: f32 = chunk.read_f32()?;

    let data_2023_8: Option<GMParticleEmitter2023_8> = if general_info.is_version_at_least(2023, 8, 0, 0) {
        let size_min_x: f32 = chunk.read_f32()?;
        let size_max_x: f32 = chunk.read_f32()?;
        let size_min_y: f32 = chunk.read_f32()?;
        let size_max_y: f32 = chunk.read_f32()?;
        let size_increase_x: f32 = chunk.read_f32()?;
        let size_increase_y: f32 = chunk.read_f32()?;
        let size_wiggle_x: f32 = chunk.read_f32()?;
        let size_wiggle_y: f32 = chunk.read_f32()?;
        let temp: TempParticleEmitter2023_8 = data_2023_8.expect("Temp 2023.8 data not set somehow");
        Some(GMParticleEmitter2023_8 {
            emit_relative: temp.emit_relative,
            delay_min: temp.delay_min,
            delay_max: temp.delay_max,
            delay_unit: temp.delay_unit,
            interval_min: temp.interval_min,
            interval_max: temp.interval_max,
            interval_unit: temp.interval_unit,
            size_min_x,
            size_max_x,
            size_min_y,
            size_max_y,
            size_increase_x,
            size_increase_y,
            size_wiggle_x,
            size_wiggle_y,
        })
    } else { None };

    let speed_min: f32 = chunk.read_f32()?;
    let speed_max: f32 = chunk.read_f32()?;
    let speed_increase: f32 = chunk.read_f32()?;
    let speed_wiggle: f32 = chunk.read_f32()?;
    let gravity_force: f32 = chunk.read_f32()?;
    let gravity_direction: f32 = chunk.read_f32()?;
    let direction_min: f32 = chunk.read_f32()?;
    let direction_max: f32 = chunk.read_f32()?;
    let direction_increase: f32 = chunk.read_f32()?;
    let direction_wiggle: f32 = chunk.read_f32()?;
    let orientation_min: f32 = chunk.read_f32()?;
    let orientation_max: f32 = chunk.read_f32()?;
    let orientation_increase: f32 = chunk.read_f32()?;
    let orientation_wiggle: f32 = chunk.read_f32()?;
    let orientation_relative: bool = chunk.read_bool32()?;

    let spawn_on_death: GMRef<GMParticleEmitter> = GMRef::new(chunk.read_usize_count()?);   // TODO probably optional
    let spawn_on_death_count: u32 = chunk.read_u32()?;
    let spawn_on_update: GMRef<GMParticleEmitter> = GMRef::new(chunk.read_usize_count()?);   // TODO probably optional
    let spawn_on_update_count: u32 = chunk.read_u32()?;

    Ok(GMParticleEmitter {
        name,
        enabled,
        mode,
        emit_count,
        data_2023_8,
        distribution,
        shape,
        region_x,
        region_y,
        region_w,
        region_h,
        rotation,
        sprite,
        texture,
        frame_index,
        data_2023_4,
        start_color,
        mid_color,
        end_color,
        additive_blend,
        lifetime_min,
        lifetime_max,
        scale_x,
        scale_y,
        speed_min,
        speed_max,
        speed_increase,
        speed_wiggle,
        gravity_force,
        gravity_direction,
        direction_min,
        direction_max,
        direction_increase,
        direction_wiggle,
        orientation_min,
        orientation_max,
        orientation_increase,
        orientation_wiggle,
        orientation_relative,
        spawn_on_death,
        spawn_on_death_count,
        spawn_on_update,
        spawn_on_update_count,
    })
}