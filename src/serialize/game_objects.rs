use crate::deserialize::all::GMData;
use crate::deserialize::game_objects::{GMGameObject, GMGameObjectEventAction};
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_objt(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("OBJT")?;
    let len: usize = gm_data.game_objects.game_objects_by_index.len();
    builder.write_usize(len);

    for i in 0..len {
        builder.write_placeholder(GMPointer::GameObject(i))?;
    }

    for (i, game_object) in gm_data.game_objects.game_objects_by_index.iter().enumerate() {
        builder.resolve_pointer(GMPointer::GameObject(i))?;
        builder.write_gm_string(&game_object.name)?;
        match &game_object.sprite {
            Some(sprite) => builder.write_usize(sprite.index),
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
            Some(sprite) => builder.write_usize(sprite.index),
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
        builder.write_i32(if game_object.uses_physics_shape_vertex { game_object.physics_shape_vertices.len() as i32 } else { -1 });
        builder.write_f32(game_object.friction);
        builder.write_bool32(game_object.awake);
        builder.write_bool32(game_object.kinematic);
        for (x, y) in &game_object.physics_shape_vertices {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        build_game_object_events(builder, game_object, i)?;
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


fn build_game_object_events(
    builder: &mut DataBuilder,
    game_object: &GMGameObject,
    game_object_index: usize,
) -> Result<(), String> {
    builder.write_usize(game_object.events.len());

    for i in 0..game_object.events.len() {
        builder.write_placeholder(GMPointer::GameObjectEvent(game_object_index, i))?;
    }

    for (i, event_instances) in game_object.events.iter().enumerate() {
        builder.resolve_pointer(GMPointer::GameObjectEvent(game_object_index, i))?;
        builder.write_usize(event_instances.len());

        for j in 0..event_instances.len() {
            builder.write_placeholder(GMPointer::GameObjectEventInstance(game_object_index, i, j))?;
        }

        for (j, event_instance) in event_instances.iter().enumerate() {
            builder.resolve_pointer(GMPointer::GameObjectEventInstance(game_object_index, i, j))?;
            builder.write_u32(event_instance.subtype);
            build_game_object_event_instance_actions(builder, &event_instance.actions, game_object_index, i, j)?;
        }
    }
    Ok(())
}


fn build_game_object_event_instance_actions(
    builder: &mut DataBuilder,
    actions: &Vec<GMGameObjectEventAction>,
    game_object_index: usize,
    event_index: usize,
    instance_index: usize,
) -> Result<(), String> {
    builder.write_usize(actions.len());

    for i in 0..actions.len() {
        builder.write_placeholder(GMPointer::GameObjectEventInstanceAction(game_object_index, event_index, instance_index, i))?;
    }

    for (i, action) in actions.iter().enumerate() {
        builder.resolve_pointer(GMPointer::GameObjectEventInstanceAction(game_object_index, event_index, instance_index, i))?;
        builder.write_u32(action.lib_id);
        builder.write_u32(action.id);
        builder.write_u32(action.kind);
        builder.write_bool32(action.use_relative);
        builder.write_bool32(action.is_question);
        builder.write_bool32(action.use_apply_to);
        builder.write_u32(action.exe_type);
        builder.write_gm_string_optional(&action.action_name)?;
        if let Some(ref code) = action.code {
            builder.write_usize(code.index);
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

