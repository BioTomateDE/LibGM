use crate::deserialize::all::GMData;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::sprites::{GMSpriteMaskEntry, GMSpriteNineSlice, GMSpriteType};
use crate::deserialize::texture_page_items::GMTexturePageItem;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};
use crate::serialize::sequence::build_sequence;

pub fn build_chunk_sprt(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("SPRT")?;
    builder.write_usize(gm_data.sprites.sprites_by_index.len());

    for i in 0..gm_data.sprites.sprites_by_index.len() {
        builder.write_placeholder(GMPointer::Sprite(i))?;
    }

    for (i, sprite) in gm_data.sprites.sprites_by_index.iter().enumerate() {
        builder.resolve_pointer(GMPointer::Sprite(i))?;
        builder.write_gm_string(&sprite.name)?;
        builder.write_u32(sprite.width);
        builder.write_u32(sprite.height);
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
                if specials.special_version >= 2 && specials.sequence.is_some() {
                    builder.write_placeholder(GMPointer::SpriteSequence(i))?;
                }
                if specials.special_version >= 3 && specials.nine_slice.is_some() {
                    builder.write_placeholder(GMPointer::SpriteNineSlice(i))?;
                }
            }

            match &specials.sprite_type {
                GMSpriteType::Normal(sprite_type) => {
                    build_texture_list(builder, &sprite.textures)?;
                    build_mask_data(builder, &sprite_type.collision_masks);
                }

                GMSpriteType::SWF(sprite_type) => {
                    builder.write_i32(sprite_type.swf_version);
                    if sprite_type.swf_version == 8 {
                        build_texture_list(builder, &sprite.textures)?;
                    }
                }

                GMSpriteType::Spine(_sprite_type) => {
                    builder.align(4);
                    return Err(format!("Sprite Type Spine not yet implemented for Sprite \"{}\"", sprite.name.display(&gm_data.strings)))
                }
            }

            if specials.special_version >= 2 {
                if let Some(ref sequence) = specials.sequence {
                    builder.resolve_pointer(GMPointer::SpriteSequence(i))?;
                    builder.write_i32(1);
                    build_sequence(builder, &gm_data.general_info, &gm_data.strings, sequence)?;
                }
            }
            if specials.special_version >= 3 {
                if let Some(ref nine_slice) = specials.nine_slice {
                    builder.resolve_pointer(GMPointer::SpriteNineSlice(i))?;
                    build_nine_slice(builder, nine_slice)?;
                }
            }

        } else {
            build_texture_list(builder, &sprite.textures)?;
            build_mask_data(builder, &sprite.collision_masks);
        }
    }

    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}


fn build_texture_list(builder: &mut DataBuilder, textures: &Vec<GMRef<GMTexturePageItem>>) -> Result<(), String> {
    builder.write_usize(textures.len());
    for i in 0..textures.len() {
        builder.write_placeholder(GMPointer::TexturePageItem(i))?;
    }
    Ok(())
}


fn build_mask_data(builder: &mut DataBuilder, collision_masks: &Vec<GMSpriteMaskEntry>) {
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


fn build_nine_slice(
    builder: &mut DataBuilder,
    nine_slice: &GMSpriteNineSlice,
) -> Result<(), String> {
    builder.write_i32(nine_slice.left);
    builder.write_i32(nine_slice.top);
    builder.write_i32(nine_slice.right);
    builder.write_i32(nine_slice.bottom);
    builder.write_bool32(nine_slice.enabled);

    for i in 0..5 {
        builder.write_i32(nine_slice.tile_modes[i].clone().into())
    }

    Ok(())
}