use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::sprites::{GMSpriteMaskEntry, GMSpriteType};
use crate::deserialize::texture_page_items::GMTexture;
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};
use crate::serialize::sequence::build_sequence;

pub fn build_chunk_sprt(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "SPRT", abs_pos: data_builder.len() };
    builder.write_usize(gm_data.sprites.sprites_by_index.len());

    for i in 0..gm_data.sprites.sprites_by_index.len() {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::sprite(i))?;
    }

    for (i, sprite) in gm_data.sprites.sprites_by_index.iter().enumerate() {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::sprite(i))?;
        builder.write_gm_string(data_builder, &sprite.name)?;
        builder.write_usize(sprite.width);
        builder.write_usize(sprite.height);
        builder.write_i32(sprite.margin_left);
        builder.write_i32(sprite.margin_right);
        builder.write_i32(sprite.margin_bottom);
        builder.write_i32(sprite.margin_top);
        builder.write_bool32(sprite.transparent);
        builder.write_bool32(sprite.smooth);
        builder.write_bool32(sprite.preload);
        builder.write_i32(sprite.bbox_mode);
        builder.write_u32(sprite.sep_masks.into());
        builder.write_i32(sprite.origin_x);
        builder.write_i32(sprite.origin_y);

        if let Some(specials) = &sprite.special_fields {
            // {~~} assert >= version 2.0.0.0
            builder.write_i32(-1);
            builder.write_u32(specials.special_version);
            builder.write_u32(match specials.sprite_type {
                GMSpriteType::Normal(_) => 0,
                GMSpriteType::SWF(_) => 1,
                GMSpriteType::Spine(_) => 2,
            });

            if gm_data.general_info.is_version_at_least(2, 0, 0, 0) {
                builder.write_f32(specials.playback_speed);
                builder.write_u32(specials.playback_speed_type.into());
                if specials.special_version >= 2 {
                    let position: usize = builder.abs_pos + builder.len();
                    data_builder.push_pointer_placeholder(&mut builder, GMPointer::sprite_sequence(position))?;
                }
                if specials.special_version >= 3 {
                    let position: usize = builder.abs_pos + builder.len();
                    data_builder.push_pointer_placeholder(&mut builder, GMPointer::sprite_nine_slice(position))?;
                }
            }

            match &specials.sprite_type {
                GMSpriteType::Normal(sprite_type) => {
                    build_texture_list(data_builder, &mut builder, &sprite.textures)?;
                    build_mask_data(&mut builder, &sprite_type.collision_masks);
                }

                GMSpriteType::SWF(sprite_type) => {
                    builder.write_i32(sprite_type.swf_version);
                    if sprite_type.swf_version == 8 {
                        build_texture_list(data_builder, &mut builder, &sprite.textures)?;
                    }
                }

                GMSpriteType::Spine(_sprite_type) => {
                    align_writer(&mut builder, 4, 0x00);
                    return Err(format!("Sprite Type Spine not yet implemented for Sprite \"{}\"", sprite.name.display(&gm_data.strings)))
                }
            }

            if specials.special_version >= 2 {
                let position: usize = builder.abs_pos + builder.len();
                data_builder.push_pointer_resolve(&mut builder, GMPointer::sprite_sequence(position))?;
                builder.write_i32(1);
                match &specials.sequence {
                    Some(sequence) => build_sequence(data_builder, &mut builder, &gm_data.general_info, &gm_data.strings, sequence)?,
                    None => return Err(format!(
                        "Sequence not set for Sprite \"{}\" at absolute position {}.",
                        sprite.name.display(&gm_data.strings), builder.abs_pos + builder.len(),
                    )),
                }
                // TODO continue ts

            }

        } else {
            build_texture_list(data_builder, &mut builder, &sprite.textures)?;
            build_mask_data(&mut builder, &sprite.collision_masks);
        }
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}


fn build_texture_list(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, textures: &Vec<GMRef<GMTexture>>) -> Result<(), String> {
    builder.write_usize(textures.len());
    for i in 0..textures.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::texture(i))?;
    }
    Ok(())
}


fn build_mask_data(builder: &mut ChunkBuilder, collision_masks: &Vec<GMSpriteMaskEntry>) {
    builder.write_usize(collision_masks.len());
    let mut total_length: usize = 0;

    for collision_mask in collision_masks {
        builder.raw_data.extend(&collision_mask.data);
        total_length += collision_mask.data.len();
    }

    // padding
    while total_length % 4 != 0 {
        builder.write_u8(0);
        total_length += 1;
    }

    // {~~} check if dimensions are valid
}


fn align_writer(builder: &mut ChunkBuilder, alignment: usize, padding_byte: u8) {
    while (builder.abs_pos + builder.len()) & (alignment - 1) != padding_byte as usize {
        builder.write_u8(padding_byte);
    }
}

