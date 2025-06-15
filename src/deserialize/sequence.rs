use crate::deserialize::chunk_reading::GMRef;
use std::collections::HashMap;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::strings::GMStrings;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::particles::GMParticleSystem;
use crate::deserialize::sounds::GMSound;
use crate::deserialize::sprites::GMSprite;

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
    pub broadcast_messages: Vec<GMRef<String>>,
    pub tracks: Vec<GMTrack>,
    pub function_ids: HashMap<i32, GMRef<String>>,
    pub moments: Vec<GMKeyframeMoment>
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframesData<T> {
    /// only set for RealKeyframes (and IntKeyframes but that doesn't exist)
    pub interpolation: Option<i32>,
    pub keyframes: Vec<GMKeyframeData<T>>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum GMKeyframes {
    Audio(GMKeyframesData<GMKeyframeAudio>),
    Instance(GMKeyframesData<GMKeyframeInstance>),
    Graphic(GMKeyframesData<GMKeyframeGraphic>),
    Sequence(GMKeyframesData<GMKeyframeSequence>),
    SpriteFrames(GMKeyframesData<GMKeyframeSpriteFrames>),
    Bool(GMKeyframesData<GMKeyframeBool>),
    // Asset(GMKeyframes<GMKeyframeAsset>),
    String(GMKeyframesData<GMKeyframeString>),
    // Int(GMKeyframes<GMKeyframeInt>),
    Color(GMKeyframesData<GMKeyframeReal>),
    Text(GMKeyframesData<GMKeyframeText>),
    Particle(GMKeyframesData<GMKeyframeParticle>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeData<T> {
    pub key: f32,
    pub length: f32,
    pub stretch: bool,
    pub disabled: bool,
    pub channels: HashMap<i32, T>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeAudio {
    pub sound: GMRef<GMSound>,
    pub mode: i32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeInstance {
    pub object: GMRef<GMGameObject>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeGraphic {
    pub sprite: GMRef<GMSprite>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeSequence {
    pub sequence: GMRef<GMSequence>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeSpriteFrames {
    pub value: i32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeBool {
    pub boolean: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeString {
    pub string: GMRef<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeReal {
    pub value: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeText {
    pub text: GMRef<String>,
    pub wrap: bool,
    pub alignment_v: i8,
    pub alignment_h: i8,
    pub font_index: i32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeParticle {
    pub particle: GMRef<GMParticleSystem>,
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
    pub keyframes: GMKeyframes,
    pub owned_resources: Vec<GMAnimationCurve>,
    pub anim_curve_string: Option<GMRef<String>>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframeMoment {
    pub internal_count: i32,    // "Should be 0 if none, 1 if there's a message?"
    pub event: Option<GMRef<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurve {
    pub name: GMRef<String>,
    pub graph_type: u32,
    pub channels: Vec<GMAnimationCurveChannel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannel {
    pub name: GMRef<String>,
    pub curve_type: GMAnimationCurveType,
    pub iterations: u32,
    pub points: Vec<GMAnimationCurveChannelPoint>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannelPoint {
    pub x: f32,
    pub y: f32,     // aka Value
    pub bezier_data: Option<GMAnimationCurveChannelPointBezierData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannelPointBezierData {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMAnimationCurveType {
    Linear = 0,
    Smooth = 1,
    // bezier missing idk
}


pub fn parse_sequence(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMSequence, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let playback: u32 = chunk.read_u32()?;
    let playback: GMSequencePlaybackType = playback.try_into()
        .map_err(|_| format!(
            "Invalid Sequence Playback Type 0x{:04X} while parsing sequence at position {} in chunk '{}'",
            playback, chunk.cur_pos, chunk.name))?;
    let playback_speed: f32 = chunk.read_f32()?;
    let playback_speed_type: u32 = chunk.read_u32()?;
    let playback_speed_type: GMAnimSpeedType = playback_speed_type.try_into()
        .map_err(|_| format!(
            "Invalid Sequence Anim Speed Type 0x{:04X} while parsing sequence at position {} in chunk '{}'",
            playback_speed_type, chunk.cur_pos, chunk.name))?;
    let length: f32 = chunk.read_f32()?;
    let origin_x: i32 = chunk.read_i32()?;
    let origin_y: i32 = chunk.read_i32()?;
    let volume: f32 = chunk.read_f32()?;
    let broadcast_messages: Vec<GMRef<String>> = parse_broadcast_messages(chunk, &strings)?;  // might be list in list?
    let tracks: Vec<GMTrack> = parse_tracks(chunk, general_info, &strings)?;

    let function_id_count: usize = chunk.read_usize_count()?;
    let mut function_ids: HashMap<i32, GMRef<String>> = HashMap::with_capacity(function_id_count);
    for _ in 0..function_id_count {
        let key: i32 = chunk.read_i32()?;
        let function_id: GMRef<String> = chunk.read_gm_string(strings)?;
        function_ids.insert(key, function_id);
    }

    let moments_count: usize = chunk.read_usize_count()?;
    let mut moments: Vec<GMKeyframeMoment> = Vec::with_capacity(moments_count);
    for _ in 0..moments_count {
        let internal_count: i32 = chunk.read_i32()?;
        let mut event: Option<GMRef<String>> = None;
        if internal_count > 0 {
            event = Some(chunk.read_gm_string(strings)?);
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
        broadcast_messages,
        tracks,
        function_ids,
        moments,
    })
}


fn parse_broadcast_messages(chunk: &mut GMChunk, strings: &GMStrings) -> Result<Vec<GMRef<String>>, String> {
    // might be double list?
    let messages_count: usize = chunk.read_usize_count()?;
    let mut messages: Vec<GMRef<String>> = Vec::with_capacity(messages_count);

    for _ in 0..messages_count {
        messages.push(chunk.read_gm_string(&strings)?);
    }

    Ok(messages)
}


fn parse_track(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMTrack, String> {
    let model_name: GMRef<String> = chunk.read_gm_string(strings)?;
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let builtin_name: i32 = chunk.read_i32()?;
    let builtin_name: GMTrackBuiltinName = builtin_name.try_into()
        .map_err(|_| format!(
            "Invalid Track builtin name 0x{:04X} while parsing Track at position {} in chunk '{}'",
            builtin_name, chunk.cur_pos, chunk.name
        ))?;
    let traits: i32 = chunk.read_i32()?;
    let traits: GMTrackTraits = traits.try_into()
        .map_err(|_| format!(
            "Invalid Track traits 0x{:04X} while parsing Track at position {} in chunk '{}'",
            traits, chunk.cur_pos, chunk.name
        ))?;
    let is_creation_track: bool = chunk.read_bool32()?;

    let mut tag_count: i32 = chunk.read_i32()?;
    if tag_count == -1 {
        tag_count = 0;
    }
    if tag_count < 0 {
        return Err(format!(
            "Invalid Track tag count {} while parsing Track at position {} in chunk '{}'",
            tag_count, chunk.cur_pos, chunk.name
        ));
    }
    let tag_count: usize = tag_count as usize;

    let mut owned_resources_count: i32 = chunk.read_i32()?;
    if owned_resources_count == -1 {
        owned_resources_count = 0;
    }
    if owned_resources_count < 0 {
        return Err(format!(
            "Invalid Track owned resources count {} while parsing Track at position {} in chunk '{}'",
            owned_resources_count, chunk.cur_pos, chunk.name
        ));
    }
    let owned_resources_count: usize = owned_resources_count as usize;

    let mut track_count: i32 = chunk.read_i32()?;
    if track_count == -1 {
        track_count = 0;
    }
    if track_count < 0 {
        return Err(format!(
            "Invalid Track track count {} while parsing Track at position {} in chunk '{}'",
            track_count, chunk.cur_pos, chunk.name
        ));
    }
    let track_count: usize = track_count as usize;

    let mut tags: Vec<i32> = Vec::with_capacity(tag_count);
    for _ in 0..tag_count {
        tags.push(chunk.read_i32()?);
    }

    let mut anim_curve_string: Option<GMRef<String>> = None;

    let mut owned_resources: Vec<GMAnimationCurve> = Vec::with_capacity(owned_resources_count);
    for _ in 0..owned_resources_count {
        let gm_anim_curve_string: GMRef<String> = chunk.read_gm_string(strings)?;
        if gm_anim_curve_string.resolve(&strings.strings_by_index)? != "GMAnimCurve" {
            return Err(format!(
                "Expected owned resource thingy of Track to be \"GMAnimCurve\"; but found \"{}\" for Track \"{}\" at absolute position {}",
                gm_anim_curve_string.display(strings), name.display(strings), chunk.cur_pos + chunk.abs_pos,
            ));
        }
        if anim_curve_string.is_none() {
            anim_curve_string = Some(gm_anim_curve_string);
        }
        owned_resources.push(parse_anim_curve(chunk, general_info, strings)?);
    }

    let mut sub_tracks: Vec<GMTrack> = Vec::with_capacity(track_count);
    for _ in 0..track_count {
        sub_tracks.push(parse_track(chunk, general_info, strings)?);
    }

    let keyframes = match model_name.resolve(&strings.strings_by_index)?.as_str() {
        "GMAudioTrack" => parse_track_keyframes(chunk, parse_keyframe_audio, GMKeyframes::Audio, false)?,
        "GMInstanceTrack" => parse_track_keyframes(chunk, parse_keyframe_instance, GMKeyframes::Instance, false)?,
        "GMGraphicTrack" => parse_track_keyframes(chunk, parse_keyframe_graphic, GMKeyframes::Graphic, false)?,
        "GMSequenceTrack" => parse_track_keyframes(chunk, parse_keyframe_sequence, GMKeyframes::Sequence, false)?,
        "GMSpriteFramesTrack" => parse_track_keyframes(chunk, parse_keyframe_sprite_frames, GMKeyframes::SpriteFrames, false)?,
        "GMAssetTrack" => return Err("Asset Track not yet supported".to_string()),
        "GMBoolTrack" => parse_track_keyframes(chunk, parse_keyframe_bool, GMKeyframes::Bool, false)?,
        "GMStringTrack" => parse_track_keyframes(chunk, |c| parse_keyframe_string(c, strings), GMKeyframes::String, false)?,
        "GMIntTrack" => return Err("Int Track not yet supported".to_string()),
        "GMColourTrack" | "GMRealTrack" => parse_track_keyframes(chunk, parse_keyframe_color, GMKeyframes::Color, true)?,
        "GMTextTrack" => parse_track_keyframes(chunk, |c| parse_keyframe_text(c, strings), GMKeyframes::Text, false)?,
        "GMParticleTrack" => parse_track_keyframes(chunk, parse_keyframe_particle, GMKeyframes::Particle, false)?,
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

fn parse_tracks(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<Vec<GMTrack>, String> {
    let tracks_count: usize = chunk.read_usize_count()?;
    let mut tracks: Vec<GMTrack> = Vec::with_capacity(tracks_count);

    for _ in 0..tracks_count {
        tracks.push(parse_track(chunk, general_info, strings)?);
    }

    Ok(tracks)
}


fn parse_anim_curve(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMAnimationCurve, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let graph_type: u32 = chunk.read_u32()?;

    let channels_count: usize = chunk.read_usize_count()?;
    let mut channels: Vec<GMAnimationCurveChannel> = Vec::with_capacity(channels_count);
    for _ in 0..channels_count {
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let curve_type: u32 = chunk.read_u32()?;
        let curve_type: GMAnimationCurveType = curve_type.try_into()
            .map_err(|_| format!(
                "Invalid Curve Type {} for Animation Curve \"{}\" at absolute position {}",
                curve_type, name.display(strings), chunk.cur_pos + chunk.abs_pos))?;
        let iterations: u32 = chunk.read_u32()?;
        let points: Vec<GMAnimationCurveChannelPoint> = parse_anim_curve_points(chunk, general_info)?;
        channels.push(GMAnimationCurveChannel {
            name,
            curve_type,
            iterations,
            points,
        })
    }

    Ok(GMAnimationCurve {
        name,
        graph_type,
        channels,
    })
}

fn parse_anim_curve_points(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<Vec<GMAnimationCurveChannelPoint>, String> {
    let points_count: usize = chunk.read_usize_count()?;
    let mut points: Vec<GMAnimationCurveChannelPoint> = Vec::with_capacity(points_count);
    
    for _ in 0..points_count {
        let x: f32 = chunk.read_f32()?;
        let y: f32 = chunk.read_f32()?;
        let bezier_data: Option<GMAnimationCurveChannelPointBezierData> = if general_info.is_version_at_least(2, 3, 1, 0) {
            let x0: f32 = chunk.read_f32()?;
            let y0: f32 = chunk.read_f32()?;
            let x1: f32 = chunk.read_f32()?;
            let y1: f32 = chunk.read_f32()?;
            Some(GMAnimationCurveChannelPointBezierData { x0, y0, x1, y1 })
        } else {
            chunk.read_i32()?;
            None
        };
        points.push(GMAnimationCurveChannelPoint { x, y, bezier_data })
    }
    
    Ok(points)
}


fn parse_track_keyframes<T>(
    chunk: &mut GMChunk,
    parse_keyframe_fn: impl Fn(&mut GMChunk) -> Result<T, String>,
    map_keyframes_fn: fn(GMKeyframesData<T>) -> GMKeyframes,
    read_interpolation: bool,
) -> Result<GMKeyframes, String> {
    chunk.align(4)?;
    let interpolation: Option<i32> = if read_interpolation {
        Some(chunk.read_i32()?)
    } else { None };

    let keyframe_count: usize = chunk.read_usize_count()?;
    let mut keyframes: Vec<GMKeyframeData<T>> = Vec::with_capacity(keyframe_count);
    for _ in 0..keyframe_count {
        let key: f32 = chunk.read_f32()?;
        let length: f32 = chunk.read_f32()?;
        let stretch: bool = chunk.read_bool32()?;
        let disabled: bool = chunk.read_bool32()?;
        let count: usize = chunk.read_usize_count()?;
        let mut channels: HashMap<i32, T> = HashMap::with_capacity(count);
        for _ in 0..count {
            let channel: i32 = chunk.read_i32()?;
            let keyframe: T = parse_keyframe_fn(chunk)?;
            channels.insert(channel, keyframe);
        }
        keyframes.push(GMKeyframeData {
            key,
            length,
            stretch,
            disabled,
            channels,
        });
    }
    let keyframes_data = GMKeyframesData {
        interpolation,
        keyframes,
    };
    Ok(map_keyframes_fn(keyframes_data))
}

fn parse_keyframe_audio(chunk: &mut GMChunk) -> Result<GMKeyframeAudio, String> {
    let sound: GMRef<GMSound> = GMRef::new(chunk.read_usize_count()?);
    let always_zero: u32 = chunk.read_u32()?;
    if always_zero != 0 {
        return Err(format!("Expected 0 in Audio Keyframes; got {always_zero}"))
    }
    let mode: i32 = chunk.read_i32()?;
    Ok(GMKeyframeAudio { sound, mode })
}

fn parse_keyframe_instance(chunk: &mut GMChunk) -> Result<GMKeyframeInstance, String> {
    let object: GMRef<GMGameObject> = GMRef::new(chunk.read_usize_count()?);
    Ok(GMKeyframeInstance { object })
}

fn parse_keyframe_graphic(chunk: &mut GMChunk) -> Result<GMKeyframeGraphic, String> {
    let sprite: GMRef<GMSprite> = GMRef::new(chunk.read_usize_count()?);
    Ok(GMKeyframeGraphic { sprite })
}

fn parse_keyframe_sequence(chunk: &mut GMChunk) -> Result<GMKeyframeSequence, String> {
    let sequence: GMRef<GMSequence> = GMRef::new(chunk.read_usize_count()?);  // TODO parse chunk SEQN
    Ok(GMKeyframeSequence { sequence })
}

fn parse_keyframe_sprite_frames(chunk: &mut GMChunk) -> Result<GMKeyframeSpriteFrames, String> {
    let value: i32 = chunk.read_i32()?;
    Ok(GMKeyframeSpriteFrames { value })
}

fn parse_keyframe_bool(chunk: &mut GMChunk) -> Result<GMKeyframeBool, String> {
    let boolean: bool = chunk.read_bool32()?;
    Ok(GMKeyframeBool { boolean })      // in UTMT, they use SimpleIntData but i assume that's just laziness
}

fn parse_keyframe_string(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMKeyframeString, String> {
    let string: GMRef<String> = chunk.read_gm_string(strings)?;
    Ok(GMKeyframeString { string })
}

fn parse_keyframe_color(chunk: &mut GMChunk) -> Result<GMKeyframeReal, String> {
    let value: f32 = chunk.read_f32()?;
    Ok(GMKeyframeReal { value })
}

fn parse_keyframe_text(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMKeyframeText, String> {
    let text: GMRef<String> = chunk.read_gm_string(strings)?;
    let wrap: bool = chunk.read_bool32()?;
    let alignment: i32 = chunk.read_i32()?;
    let font_index: i32 = chunk.read_i32()?;
    Ok(GMKeyframeText {
        text,
        wrap,
        alignment_v: ((alignment >> 8) & 0xff) as i8,
        alignment_h: (alignment & 0xff) as i8,
        font_index,
    })
}

fn parse_keyframe_particle(chunk: &mut GMChunk) -> Result<GMKeyframeParticle, String> {
    let particle: GMRef<GMParticleSystem> = GMRef::new(chunk.read_usize_count()?);
    Ok(GMKeyframeParticle { particle })
}

