use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::sequence::{
    GMAnimationCurveChannel,
    GMAnimationCurveChannelPoint,
    GMKeyframeAudio,
    GMKeyframeBool,
    GMKeyframeGraphic,
    GMKeyframeInstance,
    GMKeyframeParticle,
    GMKeyframeColor,
    GMKeyframeSequence,
    GMKeyframeSpriteFrames,
    GMKeyframeString,
    GMKeyframeText,
    GMTrackKeyframes,
    GMTrackKeyframesData,
    GMSequence,
    GMTrack,
};
use crate::deserialize::strings::GMStrings;
use crate::serialize::chunk_writing::DataBuilder;

pub fn build_sequence(
    builder: &mut DataBuilder,
    general_info: &GMGeneralInfo,
    strings: &GMStrings,
    sequence: &GMSequence,
) -> Result<(), String> {
    builder.write_gm_string(&sequence.name)?;
    builder.write_u32(sequence.playback.into());
    builder.write_f32(sequence.playback_speed);
    builder.write_u32(sequence.playback_speed_type.into());
    builder.write_f32(sequence.length);
    builder.write_i32(sequence.origin_x);
    builder.write_i32(sequence.origin_y);
    builder.write_f32(sequence.volume);
    build_broadcast_messages(builder, &sequence.broadcast_messages)?;
    build_tracks(builder, general_info, strings, &sequence.tracks)?;
    for (key, function_id) in &sequence.function_ids {
        builder.write_i32(*key);
        builder.write_gm_string(&function_id)?;
    }
    for moment in &sequence.moments {
        builder.write_i32(moment.internal_count);
        if let Some(ref event) = moment.event {
            builder.write_gm_string(&event)?;
        }
    }
    Ok(())
}

fn build_broadcast_messages(builder: &mut DataBuilder, broadcast_messages: &Vec<GMRef<String>>) -> Result<(), String> {
    // might be double list?
    builder.write_usize(broadcast_messages.len());
    for broadcast_message in broadcast_messages {
        builder.write_gm_string(&broadcast_message)?;
    }

    Ok(())
}

fn build_tracks(builder: &mut DataBuilder, general_info: &GMGeneralInfo, strings: &GMStrings, tracks: &Vec<GMTrack>) -> Result<(), String> {
    builder.write_usize(tracks.len());

    for track in tracks {
        builder.write_gm_string(&track.model_name)?;
        builder.write_gm_string(&track.name)?;
        builder.write_i32(track.builtin_name.into());
        builder.write_i32(track.traits.into());
        builder.write_bool32(track.is_creation_track);
        builder.write_usize(track.tags.len());
        builder.write_usize(track.owned_resources.len());
        builder.write_usize(track.sub_tracks.len());

        for tag in &track.tags {
            builder.write_i32(*tag);
        }

        for anim_curve in &track.owned_resources {
            match &track.anim_curve_string {
                None => {
                    log::warn!("Anim curve string not set for Track \"{}\" at absolute position {}; writing -1 as fallback",
                        track.name.display(strings), builder.len());
                    builder.write_i32(-1);
                }
                Some(string_ref) => builder.write_gm_string(&string_ref)?,
            }
            builder.write_gm_string(&anim_curve.name)?;
            builder.write_u32(anim_curve.graph_type);
            build_anim_curve_channels(builder, general_info, &anim_curve.channels)?;
        }

        build_tracks(builder, general_info, strings, &track.sub_tracks)?;
        
        // Build keyframes
        builder.align(4);
        match &track.keyframes {
            GMTrackKeyframes::Audio(k) => build_keyframes(builder, k, build_keyframe_audio, false)?,
            GMTrackKeyframes::Instance(k) => build_keyframes(builder, k, build_keyframe_instance, false)?,
            GMTrackKeyframes::Graphic(k) => build_keyframes(builder, k, build_keyframe_graphic, false)?,
            GMTrackKeyframes::Sequence(k) => build_keyframes(builder, k, build_keyframe_sequence, false)?,
            GMTrackKeyframes::SpriteFrames(k) => build_keyframes(builder, k, build_keyframe_sprite_frames, false)?,
            GMTrackKeyframes::Bool(k) => build_keyframes(builder, k, build_keyframe_bool, false)?,
            GMTrackKeyframes::String(k) => build_keyframes(builder, k, build_keyframe_string, false)?,
            GMTrackKeyframes::Color(k) => build_keyframes(builder, k, build_keyframe_color, true)?,
            GMTrackKeyframes::Text(k) => build_keyframes(builder, k, build_keyframe_text, false)?,
            GMTrackKeyframes::Particle(k) => build_keyframes(builder, k, build_keyframe_particle, false)?,
        }
        
    }

    Ok(())
}

fn build_anim_curve_channels(
    builder: &mut DataBuilder,
    general_info: &GMGeneralInfo,
    channels: &Vec<GMAnimationCurveChannel>,
) -> Result<(), String> {
    builder.write_usize(channels.len());

    for channel in channels {
        builder.write_gm_string(&channel.name)?;
        builder.write_u32(channel.curve_type.into());
        builder.write_u32(channel.iterations);
        build_anim_curve_channel_points(builder, general_info, &channel.points)?;
    }

    Ok(())
}

fn build_anim_curve_channel_points(builder: &mut DataBuilder, general_info: &GMGeneralInfo, points: &Vec<GMAnimationCurveChannelPoint>) -> Result<(), String> {
    builder.write_usize(points.len());

    for point in points {
        builder.write_f32(point.x);
        builder.write_f32(point.y);
        if general_info.is_version_at_least(2, 3, 1, 0) {
            if let Some(bezier_data) = &point.bezier_data {
                builder.write_f32(bezier_data.x0);
                builder.write_f32(bezier_data.y0);
                builder.write_f32(bezier_data.x1);
                builder.write_f32(bezier_data.y1);
            } else {
                return Err(format!(
                    "Bezier data not set for Animation Curve Channel Point at absolute position {}",
                    builder.len()
                ))
            }
        }
    }

    Ok(())
}


fn build_keyframes<T>(
    builder: &mut DataBuilder,
    keyframes_data: &GMTrackKeyframesData<T>,
    build_keyframe_fn: impl Fn(&mut DataBuilder, &T) -> Result<(), String>,
    write_interpolation: bool,
) -> Result<(), String> {
    while builder.len() % 4 != 0 {
        builder.write_u8(0);
    }
    
    if write_interpolation {
        builder.write_i32(keyframes_data.interpolation.ok_or("Interpolation data not set")?)
    }

    builder.write_usize(keyframes_data.keyframes.len());
    for keyframe_data in &keyframes_data.keyframes {
        builder.write_f32(keyframe_data.key);
        builder.write_f32(keyframe_data.length);
        builder.write_bool32(keyframe_data.stretch);
        builder.write_bool32(keyframe_data.disabled);

        for (i, keyframe) in &keyframe_data.channels {
            builder.write_i32(*i);
            build_keyframe_fn(builder, keyframe)?;
        }
    }

    Ok(())
}


fn build_keyframe_audio(builder: &mut DataBuilder, keyframe: &GMKeyframeAudio) -> Result<(), String> {
    builder.write_usize(keyframe.sound.index);
    builder.write_i32(0);
    builder.write_i32(keyframe.mode);
    Ok(())
}

fn build_keyframe_instance(builder: &mut DataBuilder, keyframe: &GMKeyframeInstance) -> Result<(), String> {
    builder.write_usize(keyframe.game_object.index);
    Ok(())
}

fn build_keyframe_graphic(builder: &mut DataBuilder, keyframe: &GMKeyframeGraphic) -> Result<(), String> {
    builder.write_usize(keyframe.sprite.index);
    Ok(())
}

fn build_keyframe_sequence(builder: &mut DataBuilder, keyframe: &GMKeyframeSequence) -> Result<(), String> {
    builder.write_usize(keyframe.sequence.index);
    Ok(())
}

fn build_keyframe_sprite_frames(builder: &mut DataBuilder, keyframe: &GMKeyframeSpriteFrames) -> Result<(), String> {
    builder.write_i32(keyframe.value);
    Ok(())
}

fn build_keyframe_bool(builder: &mut DataBuilder, keyframe: &GMKeyframeBool) -> Result<(), String> {
    builder.write_bool32(keyframe.boolean);
    Ok(())
}

fn build_keyframe_string(builder: &mut DataBuilder, keyframe: &GMKeyframeString) -> Result<(), String> {
    builder.write_gm_string(&keyframe.string)?;
    Ok(())
}

fn build_keyframe_color(builder: &mut DataBuilder, keyframe: &GMKeyframeColor) -> Result<(), String> {
    builder.write_f32(keyframe.value);
    Ok(())
}

fn build_keyframe_text(builder: &mut DataBuilder, keyframe: &GMKeyframeText) -> Result<(), String> {
    builder.write_gm_string(&keyframe.text)?;
    builder.write_bool32(keyframe.wrap);
    builder.write_i32((keyframe.alignment_v as i32) << 8 & (keyframe.alignment_h as i32));
    builder.write_i32(keyframe.font_index);
    Ok(())
}

fn build_keyframe_particle(builder: &mut DataBuilder, keyframe: &GMKeyframeParticle) -> Result<(), String> {
    builder.write_usize(keyframe.particle.index);
    Ok(())
}
