use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use std::collections::HashMap;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use crate::gamemaker::game_objects::GMGameObject;
use crate::gamemaker::particles::GMParticleSystem;
use crate::gamemaker::sounds::GMSound;
use crate::gamemaker::sprites::GMSprite;
use crate::gm_serialize::DataBuilder;
use crate::utility::{hashmap_with_capacity, vec_with_capacity};

/// This struct belong to the chunk SEQN.
/// Sprites can _also_ contain sequences (not by reference; the actual data).
#[derive(Debug, Clone)]
pub struct GMSequences {
    pub sequences: Vec<GMSequence>,
    pub exists: bool,
}
impl GMChunkElement for GMSequences {
    fn empty() -> Self {
        Self { sequences: vec![], exists: false }
    }
}
impl GMElement for GMSequences {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        if reader.chunk.end_pos - reader.chunk.start_pos == 0 {
            return Ok(Self::empty())
        }
        reader.align(4)?;
        let version: u32 = reader.read_u32()?;
        if version != 1 {
            return Err(format!("Expected SEQN version 1; got {version}"))
        }
        let sequences: Vec<GMSequence> = reader.read_pointer_list()?;
        Ok(Self { sequences, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.align(4);
        builder.write_u32(1);   // SEQN Version 1
        builder.write_pointer_list(&self.sequences)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSequence {
    pub name: GMRef<String>,
    pub playback: GMSequencePlaybackType,
    pub playback_speed: f32,
    pub playback_speed_type: GMAnimSpeedType,
    pub length: f32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub volume: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub broadcast_messages: Vec<GMKeyframeData<GMBroadcastMessage>>,
    pub tracks: Vec<GMTrack>,
    pub function_ids: HashMap<i32, GMRef<String>>,
    pub moments: Vec<GMKeyframeMoment>
}
impl GMElement for GMSequence {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let playback: u32 = reader.read_u32()?;
        let playback: GMSequencePlaybackType = playback.try_into().map_err(|_| format!(
            "Invalid Sequence Playback Type 0x{:04X} while parsing sequence at position {} in chunk '{}'",
            playback, reader.cur_pos, reader.chunk.name,
        ))?;
        let playback_speed: f32 = reader.read_f32()?;
        let playback_speed_type: u32 = reader.read_u32()?;
        let playback_speed_type: GMAnimSpeedType = playback_speed_type.try_into().map_err(|_| format!(
            "Invalid Sequence Anim Speed Type 0x{:04X} while parsing sequence at position {} in chunk '{}'",
            playback_speed_type, reader.cur_pos, reader.chunk.name
        ))?;
        let length: f32 = reader.read_f32()?;
        let origin_x: i32 = reader.read_i32()?;
        let origin_y: i32 = reader.read_i32()?;
        let volume: f32 = reader.read_f32()?;

        let mut width: Option<f32> = None;
        let mut height: Option<f32> = None;
        if reader.general_info.is_version_at_least((2024, 13, 0, 0)) {
            width = Some(reader.read_f32()?);
            height = Some(reader.read_f32()?);
        }

        let broadcast_messages: Vec<GMKeyframeData<GMBroadcastMessage>> = reader.read_simple_list()?;
        let tracks: Vec<GMTrack> = reader.read_simple_list()?;

        let function_id_count: usize = reader.read_usize()?;
        if function_id_count > 10000 {
            return Err(format!("Function id count {function_id_count} larger than failsafe amount 10k"))
        }
        let mut function_ids: HashMap<i32, GMRef<String>> = HashMap::with_capacity(function_id_count);
        for _ in 0..function_id_count {
            let key: i32 = reader.read_i32()?;
            let function_id: GMRef<String> = reader.read_gm_string()?;
            function_ids.insert(key, function_id);
        }

        let moments_count: usize = reader.read_usize()?;
        let mut moments: Vec<GMKeyframeMoment> = Vec::with_capacity(moments_count);
        for _ in 0..moments_count {
            let internal_count: i32 = reader.read_i32()?;
            let mut event: Option<GMRef<String>> = None;
            if internal_count > 0 {
                event = Some(reader.read_gm_string()?);
            }
            let moment: GMKeyframeMoment = GMKeyframeMoment {
                internal_count,
                event
            };
            moments.push(moment);
        }

        Ok(GMSequence {
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_u32(self.playback.into());
        builder.write_f32(self.playback_speed);
        builder.write_u32(self.playback_speed_type.into());
        builder.write_f32(self.length);
        builder.write_i32(self.origin_x);
        builder.write_i32(self.origin_y);
        builder.write_f32(self.volume);
        if builder.is_gm_version_at_least((2024, 13)) {
            builder.write_f32(self.width.ok_or("Sequence width not set in 2024.13+")?);
            builder.write_f32(self.height.ok_or("Sequence height not set in 2024.13+")?);
        }
        builder.write_simple_list(&self.broadcast_messages)?;
        builder.write_simple_list(&self.tracks)?;
        for (key, function_id) in &self.function_ids {
            builder.write_i32(*key);
            builder.write_gm_string(function_id)?;
        }
        builder.write_simple_list(&self.moments)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMTrackKeyframesData<T> {
    pub keyframes: Vec<GMKeyframeData<T>>,
}
impl<T: GMElement> GMElement for GMTrackKeyframesData<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.align(4)?;
        let keyframes: Vec<GMKeyframeData<T>> = reader.read_simple_list()?;
        Ok(Self { keyframes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.align(4);
        builder.write_simple_list(&self.keyframes)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMColorTrackKeyframesData<T> {
    pub interpolation: i32,
    pub keyframes: Vec<GMKeyframeData<T>>,
}
impl<T: GMElement> GMElement for GMColorTrackKeyframesData<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.align(4)?;
        let interpolation: i32 = reader.read_i32()?;
        let keyframes: Vec<GMKeyframeData<T>> = reader.read_simple_list()?;
        Ok(Self { interpolation, keyframes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.align(4);
        builder.write_i32(self.interpolation);
        builder.write_simple_list(&self.keyframes)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMTrackKeyframes {
    Audio(GMTrackKeyframesData<GMKeyframeAudio>),
    Instance(GMTrackKeyframesData<GMKeyframeInstance>),
    Graphic(GMTrackKeyframesData<GMKeyframeGraphic>),
    Sequence(GMTrackKeyframesData<GMKeyframeSequence>),
    SpriteFrames(GMTrackKeyframesData<GMKeyframeSpriteFrames>),
    Bool(GMTrackKeyframesData<GMKeyframeBool>),
    // Asset(GMKeyframes<GMKeyframeAsset>),
    String(GMTrackKeyframesData<GMKeyframeString>),
    // Int(GMKeyframes<GMKeyframeInt>),
    Color(GMColorTrackKeyframesData<GMKeyframeColor>),
    Text(GMTrackKeyframesData<GMKeyframeText>),
    Particle(GMTrackKeyframesData<GMKeyframeParticle>),
    BroadcastMessage(GMTrackKeyframesData<GMBroadcastMessage>),
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeData<T> {
    pub key: f32,
    pub length: f32,
    pub stretch: bool,
    pub disabled: bool,
    pub channels: HashMap<i32, T>,
}
impl<T: GMElement> GMElement for GMKeyframeData<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let key: f32 = reader.read_f32()?;
        let length: f32 = reader.read_f32()?;
        let stretch: bool = reader.read_bool32()?;
        let disabled: bool = reader.read_bool32()?;
        let count: usize = reader.read_usize()?;
        let mut channels: HashMap<i32, T> = hashmap_with_capacity(count)?;
        for _ in 0..count {
            let channel: i32 = reader.read_i32()?;
            let keyframe: T = T::deserialize(reader)?;
            channels.insert(channel, keyframe);
        }
        Ok(Self { key, length, stretch, disabled, channels })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(self.key);
        builder.write_f32(self.length);
        builder.write_bool32(self.stretch);
        builder.write_bool32(self.disabled);
        builder.write_usize(self.channels.len())?;
        for (channel, keyframe) in &self.channels {
            builder.write_i32(*channel);
            keyframe.serialize(builder)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeAudio {
    pub sound: GMRef<GMSound>,
    pub mode: i32,
}
impl GMElement for GMKeyframeAudio {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let sound: GMRef<GMSound> = reader.read_resource_by_id()?;
        let mode: i32 = reader.read_i32()?;
        Ok(Self { sound, mode })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_resource_id(&self.sound);
        builder.write_i32(self.mode);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeInstance {
    pub game_object: GMRef<GMGameObject>,
}
impl GMElement for GMKeyframeInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let game_object: GMRef<GMGameObject> = reader.read_resource_by_id()?;
        Ok(Self { game_object })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_resource_id(&self.game_object);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeGraphic {
    pub sprite: GMRef<GMSprite>,
}
impl GMElement for GMKeyframeGraphic {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let sprite: GMRef<GMSprite> = reader.read_resource_by_id()?;
        Ok(Self { sprite })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_resource_id(&self.sprite);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeSequence {
    pub sequence: GMRef<GMSequence>,
}
impl GMElement for GMKeyframeSequence {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let sequence: GMRef<GMSequence> = reader.read_resource_by_id()?;
        Ok(Self { sequence })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_resource_id(&self.sequence);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeSpriteFrames {
    pub value: i32,
}
impl GMElement for GMKeyframeSpriteFrames {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let value: i32 = reader.read_i32()?;
        Ok(Self { value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.value);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeBool {
    pub boolean: bool,
}
impl GMElement for GMKeyframeBool {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let boolean: bool = reader.read_bool32()?;
        Ok(Self { boolean })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_bool32(self.boolean);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeString {
    pub string: GMRef<String>,
}
impl GMElement for GMKeyframeString {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let string: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { string })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.string)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeColor {
    pub value: f32,
}
impl GMElement for GMKeyframeColor {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let value: f32 = reader.read_f32()?;
        Ok(Self { value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(self.value);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeText {
    pub text: GMRef<String>,
    pub wrap: bool,
    pub alignment_v: i8,
    pub alignment_h: i8,
    pub font_index: i32,
}
impl GMElement for GMKeyframeText {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let text: GMRef<String> = reader.read_gm_string()?;
        let wrap: bool = reader.read_bool32()?;
        let alignment: i32 = reader.read_i32()?;
        let font_index: i32 = reader.read_i32()?;
        Ok(Self {
            text,
            wrap,
            alignment_v: ((alignment >> 8) & 0xff) as i8,
            alignment_h: (alignment & 0xff) as i8,
            font_index,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.text)?;
        builder.write_bool32(self.wrap);
        builder.write_i32((self.alignment_v as i32) << 8 | self.alignment_h as i32);
        builder.write_i32(self.font_index);   // TODO no idea what this is but shouldn't it be a GMRef<GMFont> instead of an i32?
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeParticle {
    pub particle: GMRef<GMParticleSystem>,
}
impl GMElement for GMKeyframeParticle {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let particle: GMRef<GMParticleSystem> = reader.read_resource_by_id()?;
        Ok(Self { particle })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_resource_id(&self.particle);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMBroadcastMessage {
    pub messages: Vec<GMRef<String>>,
}
impl GMElement for GMBroadcastMessage {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let count: usize = reader.read_usize()?;
        let mut messages: Vec<GMRef<String>> = vec_with_capacity(count)?;
        for _ in 0..count {
            messages.push(reader.read_gm_string()?);
        }
        Ok(Self { messages })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_usize(self.messages.len())?;
        for message in &self.messages {
            builder.write_gm_string(message)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeMoment {
    pub internal_count: i32,    // "Should be 0 if none, 1 if there's a message?"
    pub event: Option<GMRef<String>>,
}
impl GMElement for GMKeyframeMoment {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let internal_count: i32 = reader.read_i32()?;
        let event: Option<GMRef<String>> = if internal_count > 0 {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        Ok(Self { internal_count, event })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.internal_count);
        if let Some(ref event) = self.event {
            builder.write_gm_string(event)?;
        }
        // FIXME: maybe there should be null written if event string not set?
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMTrack {
    pub model_name: GMRef<String>,
    pub name: GMRef<String>,
    pub builtin_name: GMTrackBuiltinName,
    pub traits: GMTrackTraits,
    pub is_creation_track: bool,
    pub tags: Vec<i32>,
    pub sub_tracks: Vec<GMTrack>,
    pub keyframes: GMTrackKeyframes,
    pub owned_resources: Vec<GMAnimationCurve>,
    pub anim_curve_string: Option<GMRef<String>>,
}
impl GMElement for GMTrack {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let model_name: GMRef<String> = reader.read_gm_string()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let builtin_name: i32 = reader.read_i32()?;
        let builtin_name: GMTrackBuiltinName = builtin_name.try_into().map_err(|_| format!(
            "Invalid Track builtin name 0x{:04X} while parsing Track at position {} in chunk '{}'",
            builtin_name, reader.cur_pos, reader.chunk.name,
        ))?;
        let traits: i32 = reader.read_i32()?;
        let traits: GMTrackTraits = traits.try_into().map_err(|_| format!(
            "Invalid Track traits 0x{:04X} while parsing Track at position {} in chunk '{}'",
            traits, reader.cur_pos, reader.chunk.name,
        ))?;
        let is_creation_track: bool = reader.read_bool32()?;

        let mut tag_count: i32 = reader.read_i32()?;
        if tag_count == -1 {
            tag_count = 0;
        }
        if tag_count < 0 {
            return Err(format!(
                "Invalid Track tag count {} while parsing Track at position {} in chunk '{}'",
                tag_count, reader.cur_pos, reader.chunk.name,
            ));
        }
        let tag_count: usize = tag_count as usize;

        let mut owned_resources_count: i32 = reader.read_i32()?;
        if owned_resources_count == -1 {
            owned_resources_count = 0;
        }
        if owned_resources_count < 0 {
            return Err(format!(
                "Invalid Track owned resources count {} while parsing Track at position {} in chunk '{}'",
                owned_resources_count, reader.cur_pos, reader.chunk.name,
            ));
        }
        let owned_resources_count: usize = owned_resources_count as usize;

        let mut track_count: i32 = reader.read_i32()?;
        if track_count == -1 {
            track_count = 0;
        }
        if track_count < 0 {
            return Err(format!(
                "Invalid Track track count {} while parsing Track at position {} in chunk '{}'",
                track_count, reader.cur_pos, reader.chunk.name,
            ));
        }
        let track_count: usize = track_count as usize;

        let mut tags: Vec<i32> = vec_with_capacity(tag_count)?;
        for _ in 0..tag_count {
            tags.push(reader.read_i32()?);
        }

        let mut anim_curve_string: Option<GMRef<String>> = None;

        let mut owned_resources: Vec<GMAnimationCurve> = vec_with_capacity(owned_resources_count)?;
        for _ in 0..owned_resources_count {
            let gm_anim_curve_str_ref: GMRef<String> = reader.read_gm_string()?;
            let gm_anim_curve_string: &String = reader.resolve_gm_str(gm_anim_curve_str_ref)?;
            if gm_anim_curve_string != "GMAnimCurve" {
                return Err(format!(
                    "Expected owned resource thingy of Track to be \"GMAnimCurve\"; but found \"{}\" for Track \"{}\" at absolute position {} in chunk '{}'",
                    gm_anim_curve_string, reader.display_gm_str(name), reader.cur_pos, reader.chunk.name,
                ));
            }
            if anim_curve_string.is_none() {
                anim_curve_string = Some(gm_anim_curve_str_ref);
            }
            owned_resources.push(GMAnimationCurve::deserialize(reader)?);
        }

        let mut sub_tracks: Vec<GMTrack> = Vec::with_capacity(track_count);
        for _ in 0..track_count {
            sub_tracks.push(Self::deserialize(reader)?);
        }

        let keyframes = match reader.resolve_gm_str(model_name)?.as_str() {
            "GMAudioTrack" => GMTrackKeyframes::Audio(GMTrackKeyframesData::deserialize(reader)?),
            "GMInstanceTrack" => GMTrackKeyframes::Instance(GMTrackKeyframesData::deserialize(reader)?),
            "GMGraphicTrack" => GMTrackKeyframes::Graphic(GMTrackKeyframesData::deserialize(reader)?),
            "GMSequenceTrack" => GMTrackKeyframes::Sequence(GMTrackKeyframesData::deserialize(reader)?),
            "GMSpriteFramesTrack" => GMTrackKeyframes::SpriteFrames(GMTrackKeyframesData::deserialize(reader)?),
            "GMAssetTrack" => return Err("Asset Track not yet supported".to_string()),
            "GMBoolTrack" => GMTrackKeyframes::Bool(GMTrackKeyframesData::deserialize(reader)?),
            "GMStringTrack" => GMTrackKeyframes::String(GMTrackKeyframesData::deserialize(reader)?),
            "GMIntTrack" => return Err("Int Track not yet supported".to_string()),
            "GMColourTrack" | "GMRealTrack" => GMTrackKeyframes::Color(GMColorTrackKeyframesData::deserialize(reader)?),
            "GMTextTrack" => GMTrackKeyframes::Text(GMTrackKeyframesData::deserialize(reader)?),
            "GMParticleTrack" => GMTrackKeyframes::Particle(GMTrackKeyframesData::deserialize(reader)?),
            other => return Err(format!("Invalid Model Name \"{other}\" while parsing Track")),
        };

        Ok(GMTrack {
            model_name,
            name,
            builtin_name,
            traits,
            is_creation_track,
            tags,
            sub_tracks,
            keyframes,
            owned_resources,
            anim_curve_string,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.model_name)?;
        builder.write_gm_string(&self.name)?;
        builder.write_i32(self.builtin_name.into());
        builder.write_i32(self.traits.into());
        builder.write_bool32(self.is_creation_track);
        builder.write_usize(self.tags.len())?;
        builder.write_usize(self.owned_resources.len())?;
        builder.write_usize(self.sub_tracks.len())?;
        for tag in &self.tags {
            builder.write_i32(*tag);
        }
        for animation_curve in &self.owned_resources {
            builder.write_gm_string(&animation_curve.name)?;
            animation_curve.serialize(builder)?;
        }
        for track in &self.sub_tracks {
            track.serialize(builder)?;
        }
        match &self.keyframes {
            GMTrackKeyframes::Audio(k) => k.serialize(builder)?,
            GMTrackKeyframes::Instance(k) => k.serialize(builder)?,
            GMTrackKeyframes::Graphic(k) => k.serialize(builder)?,
            GMTrackKeyframes::Sequence(k) => k.serialize(builder)?,
            GMTrackKeyframes::SpriteFrames(k) => k.serialize(builder)?,
            GMTrackKeyframes::Bool(k) => k.serialize(builder)?,
            GMTrackKeyframes::String(k) => k.serialize(builder)?,
            GMTrackKeyframes::Color(k) => k.serialize(builder)?,
            GMTrackKeyframes::Text(k) => k.serialize(builder)?,
            GMTrackKeyframes::Particle(k) => k.serialize(builder)?,
            GMTrackKeyframes::BroadcastMessage(k) => k.serialize(builder)?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurve {
    pub name: GMRef<String>,
    pub graph_type: u32,
    pub channels: Vec<GMAnimationCurveChannel>,
}
impl GMElement for GMAnimationCurve {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let graph_type: u32 = reader.read_u32()?;
        let channels: Vec<GMAnimationCurveChannel> = reader.read_simple_list()?;
        Ok(GMAnimationCurve { name, graph_type, channels })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_u32(self.graph_type.into());
        builder.write_simple_list(&self.channels)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannel {
    pub name: GMRef<String>,
    pub curve_type: GMAnimationCurveType,
    pub iterations: u32,
    pub points: Vec<GMAnimationCurveChannelPoint>,
}
impl GMElement for GMAnimationCurveChannel {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let curve_type: u32 = reader.read_u32()?;
        let curve_type: GMAnimationCurveType = curve_type.try_into()
            .map_err(|_| format!(
                "Invalid Curve Type {} for Animation Curve \"{}\" at absolute position {} in chunk '{}'",
                curve_type, reader.display_gm_str(name), reader.cur_pos, reader.chunk.name,
            ))?;
        let iterations: u32 = reader.read_u32()?;
        let points: Vec<GMAnimationCurveChannelPoint> = reader.read_simple_list()?;
        Ok(GMAnimationCurveChannel { name, curve_type, iterations, points })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_u32(self.curve_type.into());
        builder.write_u32(self.iterations);
        builder.write_simple_list(&self.points)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannelPoint {
    pub x: f32,
    pub y: f32,     // aka Value
    pub bezier_data: Option<GMAnimationCurveChannelPointBezierData>,
}
impl GMElement for GMAnimationCurveChannelPoint {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let x: f32 = reader.read_f32()?;
        let y: f32 = reader.read_f32()?;
        let bezier_data: Option<GMAnimationCurveChannelPointBezierData>;
        if reader.general_info.is_version_at_least((2, 3, 1, 0)) {
            bezier_data = Some(GMAnimationCurveChannelPointBezierData::deserialize(reader)?)
        } else {
            reader.read_i32()?;
            bezier_data = None;
        };
        Ok(GMAnimationCurveChannelPoint { x, y, bezier_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(self.x);
        builder.write_f32(self.y);
        if builder.is_gm_version_at_least((2, 3, 1)) {
            let bezier_data: &GMAnimationCurveChannelPointBezierData = self.bezier_data.as_ref()
                .ok_or("Sequence Track Animation Curve Point: Bezier data not set in 2.3.1+")?;
            bezier_data.serialize(builder)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannelPointBezierData {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}
impl GMElement for GMAnimationCurveChannelPointBezierData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let x0: f32 = reader.read_f32()?;
        let y0: f32 = reader.read_f32()?;
        let x1: f32 = reader.read_f32()?;
        let y1: f32 = reader.read_f32()?;
        Ok(Self { x0, y0, x1, y1 })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(self.x0);
        builder.write_f32(self.y0);
        builder.write_f32(self.x1);
        builder.write_f32(self.y1);
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum GMSequencePlaybackType {
    Oneshot = 0,
    Loop = 1,
    Pingpong = 2
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum GMAnimSpeedType {
    FramesPerSecond = 0,
    FramesPerGameFrame = 1
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMTrackBuiltinName {
    None = 0,   // no idea when/why this happens exactly
    Gain = 5,
    Pitch = 6,
    Falloff = 7,
    RotationOrImageAngle = 8,
    BlendAdd = 9,
    BlendMultiplyOrImageBlend = 10,
    Mask = 12,
    Subject = 13,
    Position = 14,
    Scale = 15,
    Origin = 16,
    ImageSpeed = 17,
    ImageIndex = 18,
    FrameSize = 20,
    CharacterSpacing = 21,
    LineSpacing = 22,
    ParagraphSpacing = 23,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMTrackTraits {
    None,
    ChildrenIgnoreOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMAnimationCurveType {
    Linear = 0,
    Smooth = 1,
    // bezier missing idk
}

