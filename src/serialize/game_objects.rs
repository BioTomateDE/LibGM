use crate::deserialize::all::GMData;
use crate::deserialize::game_objects::{GMGameObject, GMGameObjectEventAction};
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_objt(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "OBJT", abs_pos: data_builder.len() };
    let len: usize = gm_data.game_objects.game_objects_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::game_object(i))?;
    }

    for i in 0..len {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::game_object(i))?;
        let game_object: &GMGameObject = &gm_data.game_objects.game_objects_by_index[i];

        builder.write_gm_string(data_builder, &game_object.name)?;
        match &game_object.sprite {
            Some(sprite) => data_builder.push_pointer_placeholder(&mut builder, GMPointer::sprite(sprite.index))?,
            None => builder.write_i32(-1),
        };
        builder.write_bool32(game_object.visible);
        if gm_data.general_info.is_version_at_least(2022, 5, 0, 0) {
            builder.write_bool32(game_object.managed.ok_or_else(|| format!(
                "Bool `managed` not set for game object with name \"{}\"",
                game_object.name.display(&gm_data.strings)))?)
        }
        builder.write_bool32(game_object.solid);
        builder.write_i32(game_object.depth);
        builder.write_bool32(game_object.persistent);
        builder.write_i32(game_object.parent_id);
        match &game_object.texture_mask {
            Some(sprite) => data_builder.push_pointer_placeholder(&mut builder, GMPointer::sprite(sprite.index))?,
            None => builder.write_i32(-1),
        };
        builder.write_bool32(game_object.uses_physics);
        builder.write_bool32(game_object.is_sensor);
        builder.write_u32(game_object.collision_shape.into());
        builder.write_f32(game_object.density);
        builder.write_f32(game_object.restitution);
        builder.write_u32(game_object.group);
        builder.write_f32(game_object.linear_damping);
        builder.write_f32(game_object.angular_damping);
        builder.write_i32(if game_object.uses_physics_shape_vertex {game_object.physics_shape_vertices.len() as i32} else {-1});
        for (x, y) in &game_object.physics_shape_vertices {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        build_game_object_events(data_builder, &mut builder, game_object, i)?;
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}


fn build_game_object_events(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    game_object: &GMGameObject,
    game_object_index: usize,
) -> Result<(), String> {
    builder.write_usize(game_object.events.len());

    for i in 0..game_object.events.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::game_object_event(game_object_index, i))?;
    }

    for (i, event_instances) in game_object.events.iter().enumerate() {
        data_builder.push_pointer_resolve(builder, GMPointer::game_object_event(game_object_index, i))?;
        builder.write_usize(event_instances.len());

        for j in 0..event_instances.len() {
            data_builder.push_pointer_placeholder(builder, GMPointer::game_object_event_instance(game_object_index, i, j))?;
        }

        for (j, event_instance) in event_instances.iter().enumerate() {
            data_builder.push_pointer_resolve(builder, GMPointer::game_object_event_instance(game_object_index, i, j))?;
            builder.write_u32(event_instance.subtype);
            build_game_object_event_instance_actions(data_builder, builder, &event_instance.actions, game_object_index, i, j)?;
        }
    }
    Ok(())
}


fn build_game_object_event_instance_actions(
    data_builder: &mut DataBuilder,
    builder: &mut ChunkBuilder,
    actions: &Vec<GMGameObjectEventAction>,
    game_object_index: usize,
    event_index: usize,
    instance_index: usize,
) -> Result<(), String> {
    builder.write_usize(actions.len());

    for i in 0..actions.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::game_object_event_action(game_object_index, event_index, instance_index, i))?;
    }

    for (i, action) in actions.iter().enumerate() {
        data_builder.push_pointer_resolve(builder, GMPointer::game_object_event_action(game_object_index, event_index, instance_index, i))?;
        builder.write_u32(action.lib_id);
        builder.write_u32(action.id);
        builder.write_u32(action.kind);
        builder.write_bool32(action.use_relative);
        builder.write_bool32(action.is_question);
        builder.write_bool32(action.use_apply_to);
        builder.write_u32(action.exe_type);
        builder.write_gm_string(data_builder, &action.action_name)?;
        if let Some(ref code) = action.code {
            data_builder.push_pointer_placeholder(builder, GMPointer::code(code.index))?;
        } else {
            builder.write_i32(-1);
        }
        builder.write_u32(action.argument_count);
        builder.write_i32(action.who);
        builder.write_bool32(action.relative);
        builder.write_bool32(action.is_not);
        builder.write_u32(action.unknown_always_zero);
    }

    Ok(())
}

