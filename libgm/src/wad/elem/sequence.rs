// SPDX-License-Identifier: GPL-3.0-only
pub mod track;

use std::collections::HashMap;

pub use track::Track;
use track::keyframe;
use track::keyframe::BroadcastMessage;
use track::keyframe::Moment;

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::util::init::hashmap_with_capacity;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

/// This struct belong to the chunk SEQN.
/// Sprites can _also_ contain sequences (not by reference; the actual data).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Sequences {
    pub elems: Vec<Option<Sequence>>,
}

gm_named_list_chunk!(SEQN, Sequences, Sequence, nullable);

impl GMElement for Sequences {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.chunk.is_empty() {
            return Ok(Self::default());
        }
        reader.align(4)?;
        reader.read_gms2_chunk_version("SEQN Version")?;
        let elems: Vec<Option<Sequence>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_u32(1); // SEQN Version 1
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub name: GMRef<String>,
    pub playback: PlaybackType,
    pub playback_speed: f32,
    pub playback_speed_type: SpeedType,
    pub length: f32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub volume: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub broadcast_messages: Vec<keyframe::Data<BroadcastMessage>>,
    pub tracks: Vec<Track>,
    pub function_ids: HashMap<i32, GMRef<String>>,
    pub moments: Vec<keyframe::Data<Moment>>,
}

impl GMElement for Sequence {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let playback: PlaybackType = reader.read_enum()?;
        let playback_speed = reader.read_f32()?;
        let playback_speed_type: SpeedType = reader.read_enum()?;
        let length = reader.read_f32()?;
        let origin_x = reader.read_i32()?;
        let origin_y = reader.read_i32()?;
        let volume = reader.read_f32()?;

        let mut width: Option<f32> = None;
        let mut height: Option<f32> = None;
        if reader.version >= GMVersion::GM2024_13 {
            width = Some(reader.read_f32()?);
            height = Some(reader.read_f32()?);
        }

        let broadcast_messages: Vec<keyframe::Data<BroadcastMessage>> =
            reader.read_simple_list()?;
        let tracks: Vec<Track> = reader.read_simple_list()?;

        let function_id_count = reader.read_u32()?;
        let mut function_ids: HashMap<i32, GMRef<String>> =
            hashmap_with_capacity(function_id_count)?;
        for _ in 0..function_id_count {
            let key = reader.read_i32()?;
            let function_id: GMRef<String> = reader.read_gm_string()?;
            function_ids.insert(key, function_id);
        }

        let moments: Vec<keyframe::Data<Moment>> = reader.read_simple_list()?;

        Ok(Self {
            name,
            playback,
            playback_speed,
            playback_speed_type,
            length,
            origin_x,
            origin_y,
            volume,
            width,
            height,
            broadcast_messages,
            tracks,
            function_ids,
            moments,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_enum(self.playback);
        builder.write_f32(self.playback_speed);
        builder.write_enum(self.playback_speed_type);
        builder.write_f32(self.length);
        builder.write_i32(self.origin_x);
        builder.write_i32(self.origin_y);
        builder.write_f32(self.volume);
        if builder.version() >= GMVersion::GM2024_13 {
            builder.write_f32(self.width.ok_or("Sequence width not set in 2024.13+")?);
            builder.write_f32(self.height.ok_or("Sequence height not set in 2024.13+")?);
        }
        builder.write_simple_list(&self.broadcast_messages)?;
        builder.write_simple_list(&self.tracks)?;

        builder.write_usize(self.function_ids.len())?;
        for (&key, &function_id) in &self.function_ids {
            builder.write_i32(key);
            builder.write_gm_string(function_id)?;
        }

        builder.write_simple_list(&self.moments)?;
        Ok(())
    }
}

gm_enum!(PlaybackType {
    /// holy shit oneshot reference
    OneShot = 0,
    Loop = 1,
    PingPong = 2,
});

gm_enum!(SpeedType {
    FramesPerSecond = 0,
    FramesPerGameFrame = 1,
});
