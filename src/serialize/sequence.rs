use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::sequence::{GMAnimationCurveChannel, GMAnimationCurveChannelPoint, GMKeyframe, GMSequence, GMTrack};
use crate::deserialize::strings::GMStrings;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_sequence(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, strings: &GMStrings, sequence: &GMSequence) -> Result<(), String> {
    builder.write_gm_string(data_builder, &sequence.name)?;
    builder.write_u64(sequence.playback.into());
    builder.write_f32(sequence.playback_speed);
    builder.write_u64(sequence.playback_speed_type.into());
    builder.write_f32(sequence.length);
    builder.write_i32(sequence.origin_x);
    builder.write_i32(sequence.origin_y);
    builder.write_f32(sequence.volume);
    build_broadcast_messages(data_builder, builder, &sequence.broadcast_messages)?;
    build_tracks(data_builder, builder, general_info, strings, &sequence.tracks)?;
    for (key, function_id) in &sequence.function_ids {
        builder.write_i32(*key);
        builder.write_gm_string(data_builder, &function_id)?;
    }
    for moment in &sequence.moments {
        builder.write_i32(moment.internal_count);
        if let Some(ref event) = moment.event {
            builder.write_gm_string(data_builder, &event)?;
        }
    }
    Ok(())
}

fn build_broadcast_messages(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, broadcast_messages: &Vec<GMRef<String>>) -> Result<(), String> {
    // might be double list?
    builder.write_usize(broadcast_messages.len());
    for broadcast_message in broadcast_messages {
        builder.write_gm_string(data_builder, &broadcast_message)?;
    }

    Ok(())
}

fn build_tracks(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, strings: &GMStrings, tracks: &Vec<GMTrack>) -> Result<(), String> {
    builder.write_usize(tracks.len());

    for track in tracks {
        builder.write_gm_string(data_builder, &track.model_name)?;
        builder.write_gm_string(data_builder, &track.name)?;
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
                    log::warn!("Anim curve string not set for Track \"{}\" at absolute position {}; writing -1 as fallback.",
                        track.name.display(strings), builder.abs_pos+builder.len());
                    builder.write_i32(-1);
                }
                Some(string_ref) => builder.write_gm_string(data_builder, &string_ref)?,
            }
            builder.write_gm_string(data_builder, &anim_curve.name)?;
            builder.write_u64(anim_curve.graph_type);
            build_anim_curve_channels(data_builder, builder, general_info, &anim_curve.channels)?;
        }

        build_tracks(data_builder, builder, general_info, strings, &track.sub_tracks)?;
        build_keyframes(builder, general_info, &track.keyframes)?;
    }

    Ok(())
}

fn build_anim_curve_channels(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    general_info: &GMGeneralInfo,
    channels: &Vec<GMAnimationCurveChannel>,
) -> Result<(), String> {
    builder.write_usize(channels.len());

    for channel in channels {
        builder.write_gm_string(data_builder, &channel.name)?;
        builder.write_u64(channel.curve_type.into());
        builder.write_u64(channel.iterations);
        build_anim_curve_channel_points(builder, general_info, &channel.points)?;
    }

    Ok(())
}

fn build_anim_curve_channel_points(builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, points: &Vec<GMAnimationCurveChannelPoint>) -> Result<(), String> {
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
                    builder.abs_pos + builder.len()
                ))
            }
        }
    }

    Ok(())
}


fn build_keyframes(builder: &mut ChunkBuilder, _general_info: &GMGeneralInfo, keyframes: &Vec<GMKeyframe>) -> Result<(), String> {
    while (builder.len() + builder.abs_pos) % 4 != 0 {
        builder.write_u8(0);
    }

    builder.write_usize(keyframes.len());
    for keyframe in keyframes {
        builder.write_f32(keyframe.key);
        builder.write_f32(keyframe.length);
        builder.write_bool32(keyframe.stretch);
        builder.write_bool32(keyframe.disabled);

        // TODO hashmap
        for ts in &keyframe.channels {
            builder.write_i32(*ts);
            builder.write_i32(0);   // placeholder; probably doesn't work
        }
    }

    Ok(())
}

