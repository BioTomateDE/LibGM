use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::sequence::{GMSequence, GMTrack};
use crate::deserialize::strings::GMStrings;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::ChunkBuilder;

pub fn build_sequence(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, sequence: &GMSequence) -> Result<(), String> {
    builder.write_gm_string(data_builder, &sequence.name)?;
    builder.write_u32(sequence.playback.into());
    builder.write_f32(sequence.playback_speed);
    builder.write_u32(sequence.playback_speed_type.into());
    builder.write_f32(sequence.length);
    builder.write_i32(sequence.origin_x);
    builder.write_i32(sequence.origin_y);
    builder.write_f32(sequence.volume);
    build_broadcast_messages(data_builder, builder, &sequence.broadcast_messages)?;
    build_tracks(data_builder, builder, &sequence.tracks)?;
    for (key, function_id) in sequence.function_ids {
        builder.write_i32(key);
        builder.write_gm_string(data_builder, &function_id)?;
    }
    build_moments(data_builder, builder, &sequence.moments);
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

fn build_tracks(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, tracks: &Vec<GMTrack>) -> Result<(), String> {
    builder.write_usize(tracks.len());

    for track in tracks {
        builder.write_gm_string(data_builder, &track.model_name)?;
        builder.write_gm_string(data_builder, &track.name)?;
        builder.write_i32(track.builtin_name.into());
        builder.write_i32(track.traits.into());
        builder.write_bool(track.is_creation_track);
        builder.write_usize(track.tags.len());
        builder.write_usize(track.);
    }

    Ok(())
}
