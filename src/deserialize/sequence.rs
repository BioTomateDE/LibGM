use std::collections::HashMap;
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::strings::{UTStringRef, UTStrings};
use num_enum::TryFromPrimitive;

#[derive(Debug, Clone)]
pub struct UTSequence<'a> {
    pub name: UTStringRef<'a>,
    pub playback: UTSequencePlaybackType,
    pub playback_speed: f32,
    pub playback_speed_type: UTSequenceAnimSpeedType,
    pub length: f32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub volume: f32,
    pub broadcast_messages: Vec<UTStringRef<'a>>,
    pub tracks: Vec<UTTrack<'a>>,
    pub function_ids: HashMap<i32, UTStringRef<'a>>,
    pub moments: Vec<UTKeyframeMoment<'a>>
}


#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum UTSequencePlaybackType {
    Oneshot = 0,
    Loop = 1,
    Pingpong = 2
}
#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum UTSequenceAnimSpeedType {
    FramesPerSecond = 0,
    FramesPerGameFrame = 1
}
#[derive(Debug, Clone)]
pub struct UTKeyframe {
    pub key: f32,
    pub length: f32,
    pub stretch: bool,
    pub disabled: bool,
    pub channels: Vec<i32>,
}
#[derive(Debug, Clone)]
pub struct UTTrack<'a> {
    pub model_name: UTStringRef<'a>,
    pub name: UTStringRef<'a>,
    pub builtin_name: UTTrackBuiltinName,
    pub traits: UTTrackTraits,
    pub is_creation_track: bool,
    pub tags: Vec<i32>,
    pub sub_tracks: Vec<UTTrack<'a>>,
    pub keyframes: Vec<UTKeyframe>,
    // pub owned_resources: Vec<UTResource>,
    pub gm_anim_curve_string: String,
}
#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(i32)]
pub enum UTTrackBuiltinName {
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
#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(i32)]
pub enum UTTrackTraits {
    None,
    ChildrenIgnoreOrigin
}
#[derive(Debug, Clone)]
pub struct UTKeyframeMoment<'a> {
    pub internal_count: i32,    // "Should be 0 if none, 1 if there's a message?"
    pub event: Option<UTStringRef<'a>>,
}


pub fn parse_sequence<'a>(chunk: &mut UTChunk, strings: &'a UTStrings) -> Result<UTSequence<'a>, String> {
    let name: UTStringRef = chunk.read_ut_string(strings)?;
    let playback: u32 = chunk.read_u32()?;
    let playback: UTSequencePlaybackType = match playback.try_into() {
        Ok(playback) => playback,
        Err(_) => return Err(format!(
            "Invalid Sequence Playback Type 0x{:04X} while parsing sequence at position {} in chunk '{}'.",
            playback,
            chunk.file_index,
            chunk.name,
        )),
    };
    let playback_speed: f32 = chunk.read_f32()?;
    let playback_speed_type: u32 = chunk.read_u32()?;
    let playback_speed_type: UTSequenceAnimSpeedType = match playback_speed_type.try_into() {
        Ok(playback) => playback,
        Err(_) => return Err(format!(
            "Invalid Sequence Anim Speed Type 0x{:04X} while parsing sequence at position {} in chunk '{}'.",
            playback_speed_type,
            chunk.file_index,
            chunk.name,
        )),
    };
    let length: f32 = chunk.read_f32()?;
    let origin_x: i32 = chunk.read_i32()?;
    let origin_y: i32 = chunk.read_i32()?;
    let volume: f32 = chunk.read_f32()?;
    let broadcast_messages: Vec<UTStringRef> = parse_broadcast_messages(chunk, &strings)?;  // might be list in list?
    let tracks: Vec<UTTrack> = parse_tracks(chunk, &strings)?;

    let function_ids_count: usize = chunk.read_usize()?;
    let mut function_ids: HashMap<i32, UTStringRef> = HashMap::new();
    for _ in 0..function_ids_count {
        let key: i32 = chunk.read_i32()?;
        let function_id: UTStringRef = chunk.read_ut_string(strings)?;
        function_ids.insert(key, function_id);
    }

    let moments_count: usize = chunk.read_usize()?;
    let mut moments: Vec<UTKeyframeMoment> = Vec::with_capacity(moments_count);
    for _ in 0..moments_count {
        let internal_count: i32 = chunk.read_i32()?;
        let mut event: Option<UTStringRef> = None;
        if internal_count > 0 {
            event = Some(chunk.read_ut_string(strings)?);
        }
        let moment: UTKeyframeMoment = UTKeyframeMoment {
            internal_count,
            event
        };
        moments.push(moment);
    }

    Ok(UTSequence {
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


fn parse_broadcast_messages<'a>(chunk: &mut UTChunk, strings: &'a UTStrings) -> Result<Vec<UTStringRef<'a>>, String> {
    // might be double list?
    let messages_count: usize = chunk.read_usize()?;
    let mut messages: Vec<UTStringRef> = Vec::with_capacity(messages_count);

    for _ in 0..messages_count {
        messages.push(chunk.read_ut_string(&strings)?);
    }

    Ok(messages)
}


fn parse_track<'a>(chunk: &mut UTChunk, strings: &'a UTStrings) -> Result<UTTrack<'a>, String> {
    // force read string {}
    let model_name: UTStringRef = chunk.read_ut_string(strings)?;
    let name: UTStringRef = chunk.read_ut_string(strings)?;
    let builtin_name: i32 = chunk.read_i32()?;
    let builtin_name: UTTrackBuiltinName = match builtin_name.try_into() {
        Ok(name) => name,
        Err(_) => return Err(format!(
            "Invalid Track builtin name 0x{:04X} while parsing Track at position {} in chunk '{}'.",
            builtin_name,
            chunk.file_index,
            chunk.name
        )),
    };
    let traits: i32 = chunk.read_i32()?;
    let traits: UTTrackTraits = match traits.try_into() {
        Ok(name) => name,
        Err(_) => return Err(format!(
            "Invalid Track traits 0x{:04X} while parsing Track at position {} in chunk '{}'.",
            traits,
            chunk.file_index,
            chunk.name
        )),
    };
    let is_creation_track: bool = chunk.read_u32()? != 0;

    let mut tag_count: i32 = chunk.read_i32()?;
    if tag_count == -1 {
        tag_count = 0;
    }
    if tag_count < 0 {
        return Err(format!(
            "Invalid Track tag count {} while parsing Track at position {} in chunk '{}'.",
            tag_count,
            chunk.file_index,
            chunk.name
        ));
    }
    let tag_count: usize = tag_count as usize;

    let mut owned_resources_count: i32 = chunk.read_i32()?;
    if owned_resources_count == -1 {
        owned_resources_count = 0;
    }
    if owned_resources_count < 0 {
        return Err(format!(
            "Invalid Track owned resources count {} while parsing Track at position {} in chunk '{}'.",
            owned_resources_count,
            chunk.file_index,
            chunk.name
        ));
    }
    let _owned_resources_count: usize = owned_resources_count as usize;

    let mut track_count: i32 = chunk.read_i32()?;
    if track_count == -1 {
        track_count = 0;
    }
    if track_count < 0 {
        return Err(format!(
            "Invalid Track track count {} while parsing Track at position {} in chunk '{}'.",
            track_count,
            chunk.file_index,
            chunk.name
        ));
    }
    let track_count: usize = track_count as usize;

    let mut tags: Vec<i32> = Vec::with_capacity(tag_count);
    for _ in 0..tag_count {
        tags.push(chunk.read_i32()?);
    }

    // owned resources {}

    let mut sub_tracks: Vec<UTTrack> = Vec::with_capacity(track_count);
    for _ in 0..track_count {
        sub_tracks.push(parse_track(chunk, strings)?);
    }

    // TODO keyframes with different types {}
    let keyframes: Vec<UTKeyframe> = vec![];

    Ok(UTTrack {
        model_name,
        name,
        builtin_name,
        traits,
        is_creation_track,
        tags,
        sub_tracks,
        keyframes,
        gm_anim_curve_string: "GMAnimCurve".to_string(),
    })
}

fn parse_tracks<'a>(chunk: &mut UTChunk, strings: &'a UTStrings) -> Result<Vec<UTTrack<'a>>, String> {
    let tracks_count: usize = chunk.read_usize()?;
    let mut tracks: Vec<UTTrack> = Vec::with_capacity(tracks_count);

    for _ in 0..tracks_count {
        tracks.push(parse_track(chunk, strings)?);
    }

    Ok(tracks)
}

