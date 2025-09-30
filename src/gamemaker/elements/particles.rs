﻿use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::elements::sprites::GMSprite;
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::utility::num_enum_from;

#[derive(Debug, Clone)]
pub struct GMParticleSystems {
    pub particle_systems: Vec<GMParticleSystem>,
    pub exists: bool,
}

impl GMChunkElement for GMParticleSystems {
    fn stub() -> Self {
        Self { particle_systems: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMParticleSystems {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.align(4)?;
        let psys_version: u32 = reader.read_u32()?;
        if psys_version != 1 {
            return Err(format!("Invalid or unsupported PSYS version {0} (0x{0:8X})", psys_version))
        }
        let particle_systems: Vec<GMParticleSystem> = reader.read_pointer_list()?;
        Ok(Self { particle_systems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.align(4);
        builder.write_u32(1);   // PSYS Version
        builder.write_pointer_list(&self.particle_systems)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleSystem {
    pub name: GMRef<String>,
    pub origin_x: i32,
    pub origin_y: i32,
    pub draw_order: i32,
    pub global_space_particles: Option<bool>,
    pub emitters: Vec<GMRef<GMParticleEmitter>>,
}
impl GMElement for GMParticleSystem {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let origin_x: i32 = reader.read_i32()?;
        let origin_y: i32 = reader.read_i32()?;
        let draw_order: i32 = reader.read_i32()?;
        let global_space_particles: Option<bool> = reader.deserialize_if_gm_version((2023, 8))?;
        let emitters: Vec<GMRef<GMParticleEmitter>> = reader.read_simple_list_of_resource_ids()?;
        Ok(Self { name, origin_x, origin_y, draw_order, global_space_particles, emitters })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_i32(self.origin_x);
        builder.write_i32(self.origin_y);
        builder.write_i32(self.draw_order);
        self.global_space_particles.serialize_if_gm_ver(builder, "Global Space Particles", (2023, 8))?;
        builder.write_simple_list_of_resource_ids(&self.emitters)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMParticleEmitters {
    pub emitters: Vec<GMParticleEmitter>,
    pub exists: bool,
}
impl GMChunkElement for GMParticleEmitters {
    fn stub() -> Self {
        Self { emitters: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMParticleEmitters {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.align(4)?;
        let psem_version: u32 = reader.read_u32()?;
        if psem_version != 1 {
            return Err(format!("Invalid or unsupported PSEM version {0} (0x{0:8X})", psem_version))
        }
        let emitters: Vec<GMParticleEmitter> = reader.read_pointer_list()?;
        Ok(Self { emitters, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.align(4);
        builder.write_u32(1);   // PSEM Version
        builder.write_pointer_list(&self.emitters)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleEmitter {
    pub name: GMRef<String>,
    pub enabled: Option<bool>,
    pub mode: EmitMode,
    pub emit_count: u32,
    pub data_2023_8: Option<GMParticleEmitter2023_8>,
    pub data_pre_2023_8: Option<GMParticleEmitterPre2023_8>,
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
    pub spawn_on_death: Option<GMRef<Self>>,
    pub spawn_on_death_count: u32,
    pub spawn_on_update: Option<GMRef<Self>>,
    pub spawn_on_update_count: u32,
}
impl GMElement for GMParticleEmitter {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let enabled: Option<bool> = if reader.general_info.is_version_at_least((2023, 6, 0, 0)) {
            Some(reader.read_bool32()?)
        } else {
            None
        };
        let mode: EmitMode = num_enum_from(reader.read_i32()?)?;

        let emit_count: u32;
        let temp_data_2023_8: Option<TempParticleEmitter2023_8> = if reader.general_info.is_version_at_least((2023, 8, 0, 0)) {
            emit_count = reader.read_f32()? as u32;              // don't see how a float is a count but ok
            let emit_relative: bool = reader.read_bool32()?;     // always zero
            let delay_min: f32 = reader.read_f32()?;
            let delay_max: f32 = reader.read_f32()?;
            let delay_unit: TimeUnit = num_enum_from(reader.read_i32()?)?;
            let interval_min: f32 = reader.read_f32()?;
            let interval_max: f32 = reader.read_f32()?;
            let interval_unit: TimeUnit = num_enum_from(reader.read_i32()?)?;

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
            emit_count = reader.read_u32()?;
            None
        };

        let distribution: EmitterDistribution = num_enum_from(reader.read_i32()?)?;
        let shape: EmitterShape = num_enum_from(reader.read_i32()?)?;
        let region_x: f32 = reader.read_f32()?;
        let region_y: f32 = reader.read_f32()?;
        let region_w: f32 = reader.read_f32()?;
        let region_h: f32 = reader.read_f32()?;
        let rotation: f32 = reader.read_f32()?;
        let sprite: GMRef<GMSprite> = reader.read_resource_by_id()?;
        let texture: EmitterTexture = num_enum_from(reader.read_i32()?)?;
        let frame_index: f32 = reader.read_f32()?;

        let data_2023_4: Option<GMParticleEmitter2023_4> = if reader.general_info.is_version_at_least((2023, 4, 0, 0)) {
            let animate: bool = reader.read_bool32()?;
            let stretch: bool = reader.read_bool32()?;
            let is_random: bool = reader.read_bool32()?;
            Some(GMParticleEmitter2023_4 { animate, stretch, is_random })
        } else { None };

        let start_color: u32 = reader.read_u32()?;
        let mid_color: u32 = reader.read_u32()?;
        let end_color: u32 = reader.read_u32()?;
        let additive_blend: bool = reader.read_bool32()?;
        let lifetime_min: f32 = reader.read_f32()?;
        let lifetime_max: f32 = reader.read_f32()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;

        let data_2023_8: Option<GMParticleEmitter2023_8>;
        let data_pre_2023_8: Option<GMParticleEmitterPre2023_8>;
        if reader.general_info.is_version_at_least((2023, 8, 0, 0)) {
            let size_min_x: f32 = reader.read_f32()?;
            let size_max_x: f32 = reader.read_f32()?;
            let size_min_y: f32 = reader.read_f32()?;
            let size_max_y: f32 = reader.read_f32()?;
            let size_increase_x: f32 = reader.read_f32()?;
            let size_increase_y: f32 = reader.read_f32()?;
            let size_wiggle_x: f32 = reader.read_f32()?;
            let size_wiggle_y: f32 = reader.read_f32()?;
            let temp: TempParticleEmitter2023_8 = temp_data_2023_8.expect("Temp 2023.8 data not set somehow");
            data_2023_8 = Some(GMParticleEmitter2023_8 {
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
            });
            data_pre_2023_8 = None;
        } else {
            let size_min: f32 = reader.read_f32()?;
            let size_max: f32 = reader.read_f32()?;
            let size_increase: f32 = reader.read_f32()?;
            let size_wiggle: f32 = reader.read_f32()?;
            data_2023_8 = None;
            data_pre_2023_8 = Some(GMParticleEmitterPre2023_8 {
                size_min,
                size_max,
                size_increase,
                size_wiggle,
            })
        };

        let speed_min: f32 = reader.read_f32()?;
        let speed_max: f32 = reader.read_f32()?;
        let speed_increase: f32 = reader.read_f32()?;
        let speed_wiggle: f32 = reader.read_f32()?;
        let gravity_force: f32 = reader.read_f32()?;
        let gravity_direction: f32 = reader.read_f32()?;
        let direction_min: f32 = reader.read_f32()?;
        let direction_max: f32 = reader.read_f32()?;
        let direction_increase: f32 = reader.read_f32()?;
        let direction_wiggle: f32 = reader.read_f32()?;
        let orientation_min: f32 = reader.read_f32()?;
        let orientation_max: f32 = reader.read_f32()?;
        let orientation_increase: f32 = reader.read_f32()?;
        let orientation_wiggle: f32 = reader.read_f32()?;
        let orientation_relative: bool = reader.read_bool32()?;

        let spawn_on_death: Option<GMRef<GMParticleEmitter>> = reader.read_resource_by_id_opt()?;
        let spawn_on_death_count: u32 = reader.read_u32()?;
        let spawn_on_update: Option<GMRef<GMParticleEmitter>> = reader.read_resource_by_id_opt()?;
        let spawn_on_update_count: u32 = reader.read_u32()?;

        Ok(GMParticleEmitter {
            name,
            enabled,
            mode,
            emit_count,
            data_2023_8,
            data_pre_2023_8,
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        self.enabled.serialize_if_gm_ver(builder, "Enabled", (2023, 6))?;
        builder.write_i32(self.mode.into());
        if builder.is_gm_version_at_least((2023, 8)) {
            let data_2023_8: &GMParticleEmitter2023_8 = self.data_2023_8.as_ref().ok_or("2023.8 Data not set")?;
            builder.write_f32(self.emit_count as f32);
            builder.write_bool32(data_2023_8.emit_relative);
            builder.write_f32(data_2023_8.delay_min);
            builder.write_f32(data_2023_8.delay_max);
            builder.write_i32(data_2023_8.delay_unit.into());
            builder.write_f32(data_2023_8.interval_min);
            builder.write_f32(data_2023_8.interval_max);
            builder.write_i32(data_2023_8.interval_unit.into());
        } else {
            builder.write_u32(self.emit_count);
        }
        builder.write_i32(self.distribution.into());
        builder.write_i32(self.shape.into());
        builder.write_f32(self.region_x);
        builder.write_f32(self.region_y);
        builder.write_f32(self.region_w);
        builder.write_f32(self.region_h);
        builder.write_f32(self.rotation);
        builder.write_resource_id(&self.sprite);
        builder.write_i32(self.texture.into());
        builder.write_f32(self.frame_index);
        if builder.is_gm_version_at_least((2023, 4)) {
            let data_2023_4: &GMParticleEmitter2023_4 = self.data_2023_4.as_ref().ok_or("2023.4 Data not set")?;
            builder.write_bool32(data_2023_4.animate);
            builder.write_bool32(data_2023_4.stretch);
            builder.write_bool32(data_2023_4.is_random);
        }
        builder.write_u32(self.start_color);
        builder.write_u32(self.mid_color);
        builder.write_u32(self.end_color);
        builder.write_bool32(self.additive_blend);
        builder.write_f32(self.lifetime_min);
        builder.write_f32(self.lifetime_max);
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        if builder.is_gm_version_at_least((2023, 8)) {
            let data_2023_8: &GMParticleEmitter2023_8 = self.data_2023_8.as_ref().ok_or("2023.8 Data not set")?;
            builder.write_f32(data_2023_8.size_min_x);
            builder.write_f32(data_2023_8.size_max_x);
            builder.write_f32(data_2023_8.size_min_y);
            builder.write_f32(data_2023_8.size_max_y);
            builder.write_f32(data_2023_8.size_increase_x);
            builder.write_f32(data_2023_8.size_increase_y);
            builder.write_f32(data_2023_8.size_wiggle_x);
            builder.write_f32(data_2023_8.size_wiggle_y);
        } else {
            let data_pre_2023_8: &GMParticleEmitterPre2023_8 = self.data_pre_2023_8.as_ref().ok_or("Pre 2023.8 Data not set")?;
            builder.write_f32(data_pre_2023_8.size_min);
            builder.write_f32(data_pre_2023_8.size_max);
            builder.write_f32(data_pre_2023_8.size_increase);
            builder.write_f32(data_pre_2023_8.size_wiggle);
        }
        builder.write_f32(self.speed_min);
        builder.write_f32(self.speed_max);
        builder.write_f32(self.speed_increase);
        builder.write_f32(self.speed_wiggle);
        builder.write_f32(self.gravity_force);
        builder.write_f32(self.gravity_direction);
        builder.write_f32(self.direction_min);
        builder.write_f32(self.direction_max);
        builder.write_f32(self.direction_increase);
        builder.write_f32(self.direction_wiggle);
        builder.write_f32(self.orientation_min);
        builder.write_f32(self.orientation_max);
        builder.write_f32(self.orientation_increase);
        builder.write_f32(self.orientation_wiggle);
        builder.write_bool32(self.orientation_relative);
        builder.write_resource_id_opt(&self.spawn_on_death);
        builder.write_u32(self.spawn_on_death_count);
        builder.write_resource_id_opt(&self.spawn_on_update);
        builder.write_u32(self.spawn_on_update_count);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleEmitter2023_4 {
    pub animate: bool,
    pub stretch: bool,
    pub is_random: bool,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMParticleEmitterPre2023_8 {
    pub size_min: f32,
    pub size_max: f32,
    pub size_increase: f32,
    pub size_wiggle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum EmitMode {
    Stream = 0,
    Burst = 1,
}
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum TimeUnit {
    Seconds = 0,
    Frames = 1,
}
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum EmitterDistribution {
    Linear = 0,
    Gaussian = 1,
    InverseGaussian = 2,
}
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum EmitterShape {
    Rectangle = 0,
    Ellipse = 1,
    Diamond = 2,
    Line = 3,
}
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
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

