use crate::deserialize::chunk_reading::GMRef;
use std::collections::HashMap;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::strings::GMStrings;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::general_info::GMGeneralInfo;
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


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMSequencePlaybackType {
    Oneshot = 0,
    Loop = 1,
    Pingpong = 2
}
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMAnimSpeedType {
    FramesPerSecond = 0,
    FramesPerGameFrame = 1
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMKeyframe {
    pub key: f32,
    pub length: f32,
    pub stretch: bool,
    pub disabled: bool,
    pub channels: Vec<i32>,   // {~~} TODO change ts to HashMap
}

#[derive(Debug, Clone)]
pub struct GMKeyframesAudio {
    pub keyframes: Vec<GMKeyframe>,
    pub mode: i32,
}
#[derive(Debug, Clone)]
pub struct GMKeyframesInstance {
    pub keyframes: Vec<GMKeyframe>,
    pub object: GMRef<GMGameObject>,
}
#[derive(Debug, Clone)]
pub struct GMKeyframesGraphic {
    pub keyframes: Vec<GMKeyframe>,
    pub sprite: GMRef<GMSprite>,
}
#[derive(Debug, Clone)]
pub struct GMKeyframesSequence {
    pub keyframes: Vec<GMKeyframe>,
    pub sequence: GMRef<GMSequence>,
}
#[derive(Debug, Clone)]
pub struct GMKeyframesSpriteFrames {
    pub keyframes: Vec<GMKeyframe>,
    pub value: i32,
}
#[derive(Debug, Clone)]
pub struct GMKeyframesBool {
    pub keyframes: Vec<GMKeyframe>,
    pub boolean: bool,
}

#[derive(Debug, Clone)]
pub struct GMKeyframesString {
    pub keyframes: Vec<GMKeyframe>,
    pub string: GMRef<String>,
}

#[derive(Debug, Clone)]
pub struct GMKeyframesColor {
    pub keyframes: Vec<GMKeyframe>,
    pub interpolation: i32,
}

#[derive(Debug, Clone)]
pub struct GMKeyframesText {
    pub keyframes: Vec<GMKeyframe>,
    pub text: GMRef<String>,
    pub wrap: bool,
    pub alignment_v: i8,
    pub alignment_h: i8,
    pub font_index: i32,
}

// #[derive(Debug, Clone)]
// pub struct GMKeyframesParticle {
//     pub keyframes: Vec<GMKeyframe>,
//     pub particle: GMRef<GMParticle>,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct GMTrack {
    pub model_name: GMRef<String>,
    pub name: GMRef<String>,
    pub builtin_name: GMTrackBuiltinName,
    pub traits: GMTrackTraits,
    pub is_creation_track: bool,
    pub tags: Vec<i32>,
    pub sub_tracks: Vec<GMTrack>,
    pub keyframes: Vec<GMKeyframe>,
    pub owned_resources: Vec<GMAnimationCurve>,
    pub anim_curve_string: Option<GMRef<String>>,
}
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMTrackBuiltinName {
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
    ParagraphSpacing = 23
}
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMTrackTraits {
    None,
    ChildrenIgnoreOrigin
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

    let function_ids_count: usize = chunk.read_usize()?;
    let mut function_ids: HashMap<i32, GMRef<String>> = HashMap::new();
    for _ in 0..function_ids_count {
        let key: i32 = chunk.read_i32()?;
        let function_id: GMRef<String> = chunk.read_gm_string(strings)?;
        function_ids.insert(key, function_id);
    }

    let moments_count: usize = chunk.read_usize()?;
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
    let messages_count: usize = chunk.read_usize()?;
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

    // TODO keyframes with different types {~~}
    let keyframes: Vec<GMKeyframe> = vec![];
    log::info!("asg");

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
    let tracks_count: usize = chunk.read_usize()?;
    let mut tracks: Vec<GMTrack> = Vec::with_capacity(tracks_count);

    for _ in 0..tracks_count {
        tracks.push(parse_track(chunk, general_info, strings)?);
    }

    Ok(tracks)
}


fn parse_anim_curve(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings) -> Result<GMAnimationCurve, String> {
    let name: GMRef<String> = chunk.read_gm_string(strings)?;
    let graph_type: u32 = chunk.read_u32()?;

    let channels_count: usize = chunk.read_usize()?;
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
    let points_count: usize = chunk.read_usize()?;
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

