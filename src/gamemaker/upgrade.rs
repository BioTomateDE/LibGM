use std::collections::HashMap;
use crate::bench;
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::backgrounds::{GMBackground, GMBackgroundGMS2Data};
use crate::gamemaker::elements::code::{CodeVariable, GMCallInstruction, GMCode, GMCodeBytecode15, GMCodeValue, GMDataType, GMDoubleTypeInstruction, GMGotoInstruction, GMInstanceType, GMInstruction, GMPopInstruction, GMPushInstruction, GMSingleTypeInstruction, GMVariableType};
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTexture2022_9;
use crate::gamemaker::elements::general_info::GMGeneralInfoGMS2;
use crate::gamemaker::elements::rooms::GMRoomTileTexture;
use crate::gamemaker::elements::scripts::GMScript;
use crate::gamemaker::elements::sequence::GMAnimSpeedType::FramesPerGameFrame;
use crate::gamemaker::elements::sprites::{GMSprite, GMSpriteSepMaskType, GMSpriteSpecial, GMSpriteSpecialData, GMSprites};
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::variables::GMVariable;
use crate::gamemaker::gm_version::{GMVersion, LTSBranch};


/// Updates GameMaker data file from GMS1 to version 2023.6 LTS, bytecode 17.
pub fn upgrade_to_2023_lts(mut gm_data: GMData) -> Result<GMData, String> {
    bench!("Upgrading to 2023 LTS",
        upgrade_to_2023_lts_(&mut gm_data)
        .map_err(|e| format!("{e}\n↳ upgrading to GameMaker Version 2023 LTS"))?
    );
    Ok(gm_data)
}

fn upgrade_to_2023_lts_(gm_data: &mut GMData) -> Result<(), String> {
    let ported_background_sprites_offset: usize = gm_data.sprites.sprites.len();
    bench!("update_general_info", update_general_info(gm_data));
    bench!("update_backgrounds", update_backgrounds(gm_data)?);
    bench!("update_fonts", update_fonts(gm_data));
    bench!("update_rooms", update_rooms(gm_data)?);
    bench!("update_texture_pages", update_texture_pages(gm_data)?);
    bench!("update_game_objects", update_game_objects(gm_data));
    bench!("replace_instance_create", replace_instance_create(gm_data)?);
    bench!("replace_background_funcs", replace_background_funcs(gm_data, ported_background_sprites_offset)?);
    bench!("replace_action_funcs", replace_action_funcs(gm_data)?);
    bench!("replace_joystick_funcs", replace_joystick_funcs(gm_data)?);
    bench!("replace_layer_funcs", replace_layer_funcs(gm_data)?);
    bench!("generate_steam_stubs", generate_steam_stubs(gm_data)?);
    Ok(())
}


/// Updates general project information for GM 2022.9
fn update_general_info(gm_data: &mut GMData) {
    gm_data.general_info.version = GMVersion::new(2023, 6, 0, 0, LTSBranch::LTS);
    gm_data.general_info.bytecode_version = 17;
    gm_data.general_info.gms2_info = Some(GMGeneralInfoGMS2 {
        random_uid: [0; 4],
        fps: 30.0,
        allow_statistics: false,
        // TODO: this should probably be some hash or randomly generated
        game_guid: [0x2B, 0x3B, 0x8B, 0x85, 0x8B, 0xF1, 0x4B, 0x57, 0xB4, 0x11, 0x6C, 0xC9, 0x7D, 0x32, 0xF4, 0x93],
        info_timestamp_offset: false,
    });
}


/// Updates background resources with GMS2-specific data
fn update_backgrounds(gm_data: &mut GMData) -> Result<(), String> {
    for background in &mut gm_data.backgrounds.backgrounds {
        let texture: GMRef<GMTexturePageItem> = background.texture.ok_or_else(|| format!(
            "Background {} doesn't have a texture page item set",
            background.name.display(&gm_data.strings),
        ))?;
        let texture: &GMTexturePageItem = texture.resolve(&gm_data.texture_page_items.texture_page_items)?;

        background.gms2_data = Some(GMBackgroundGMS2Data {
            tile_width: texture.target_width as u32,
            tile_height: texture.target_height as u32,
            output_border_x: 0,
            output_border_y: 0,
            tile_columns: 1,
            items_per_tile_count: 1,
            frame_length: 66666,
            tile_ids: vec![0],
        });
    }
    Ok(())
}


/// Updates font resources with required GMS2 fields
fn update_fonts(gm_data: &mut GMData) {
    for font in &mut gm_data.fonts.fonts {
        font.ascender_offset = Some(0);
        font.ascender = Some(0);
        font.sdf_spread = Some(0);
        font.line_height = Some(0);
    }
}


/// Updates room data, converting background tiles to sprite tiles
fn update_rooms(gm_data: &mut GMData) -> Result<(), String> {
    let mut sprites_by_name: HashMap<GMRef<String>, GMRef<GMSprite>> = HashMap::new();

    for room in &mut gm_data.rooms.rooms {
        // Update game objects in room
        for obj in &mut room.game_objects {
            obj.image_index = Some(0);
            obj.image_speed = Some(1.0);
        }

        // Convert background tiles to sprite tiles
        for tile in &mut room.tiles {
            let Some(tile_texture) = &tile.texture else { continue };
            let GMRoomTileTexture::Background(background_ref) = tile_texture else { continue };

            let background = background_ref.resolve(&gm_data.backgrounds.backgrounds)?;
            let texture: Option<&GMTexturePageItem> = background.texture
                .map(|t| t.resolve(&gm_data.texture_page_items.texture_page_items))
                .transpose()?;

            // Find or create sprite for this background
            let sprite_ref: GMRef<GMSprite> = sprites_by_name.get(&background.name).copied().unwrap_or_else(|| {
                let new_sprite_ref: GMRef<GMSprite> = GMRef::new(gm_data.sprites.sprites.len() as u32);
                sprites_by_name.insert(background.name, new_sprite_ref);
                create_sprite_from_background(&mut gm_data.sprites, background, texture)
            });

            // Update tile to use sprite instead of background
            tile.texture = Some(GMRoomTileTexture::Sprite(sprite_ref));
        }
    }

    Ok(())
}


/// Creates a new sprite from background data
fn create_sprite_from_background(gm_sprites: &mut GMSprites, background: &GMBackground, texture: Option<&GMTexturePageItem>) -> GMRef<GMSprite> {
    let new_sprite_ref: GMRef<GMSprite> = GMRef::new(gm_sprites.sprites.len() as u32);

    let new_sprite = GMSprite {
        name: background.name,
        width: texture.map_or(0, |t| t.target_width as u32),
        height: texture.map_or(0, |t| t.target_height as u32),
        margin_left: 0,
        margin_right: 0,
        margin_bottom: 0,
        margin_top: 0,
        transparent: false,
        smooth: background.smooth,
        preload: background.preload,
        bbox_mode: 0,
        sep_masks: GMSpriteSepMaskType::AxisAlignedRect,
        origin_x: 0,
        origin_y: 0,
        textures: vec![background.texture],
        collision_masks: vec![],
        special_fields: Some(GMSpriteSpecial {
            special_version: 3,
            data: GMSpriteSpecialData::Normal,
            playback_speed: 1.0,
            playback_speed_type: FramesPerGameFrame,
            sequence: None,
            nine_slice: None,
            yyswf: None,
        }),
    };

    gm_sprites.sprites.push(new_sprite);
    new_sprite_ref
}


/// Updates texture pages with GMS2 2022.9 specific data
fn update_texture_pages(gm_data: &mut GMData) -> Result<(), String> {
    for texture_page in &mut gm_data.embedded_textures.texture_pages {
        texture_page.generated_mips = Some(0);
        // Magic value that works if all texture pages are embedded
        texture_page.texture_block_size = Some(0xDEADC0DE);

        let image = texture_page.image.as_ref().ok_or("External Texture pages are not supported")?;
        let image = image.to_dynamic_image()?;

        texture_page.data_2022_9 = Some(GMEmbeddedTexture2022_9 {
            texture_width: image.width() as i32,
            texture_height: image.height() as i32,
            index_in_group: 0,
        });
    }
    Ok(())
}


/// Updates game objects with GMS2 managed flag
fn update_game_objects(gm_data: &mut GMData) {
    for obj in &mut gm_data.game_objects.game_objects {
        obj.managed = Some(false);
    }
}


/// Replaces the GMS1 `instance_create` function with the GMS2 `instance_create_depth` function.
fn replace_instance_create(gm_data: &mut GMData) -> Result<(), String> {
    let instructions = vec![
        generate_push_argument_var(gm_data, 2)?,
        GMInstruction::PushImmediate(0),    // depth
        GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Variable }),
        generate_push_argument_var(gm_data, 1)?,
        generate_push_argument_var(gm_data, 0)?,
        generate_call_builtin(gm_data, "instance_create_depth", 4)?,
        GMInstruction::Return(GMSingleTypeInstruction { data_type: GMDataType::Variable })
    ];
    generate_script(gm_data, "instance_create", 3, instructions)
}


/// Replaces the GMS1 `draw_background_part_ext` function with the new GMS2 `draw_sprite_part_ext` function.
fn replace_background_funcs(gm_data: &mut GMData, ported_background_sprites_offset: usize) -> Result<(), String> {
    let instructions = vec![
        generate_push_argument_var(gm_data, 10)?,
        generate_push_argument_var(gm_data, 9)?,
        generate_push_argument_var(gm_data, 8)?,
        generate_push_argument_var(gm_data, 7)?,
        generate_push_argument_var(gm_data, 6)?,
        generate_push_argument_var(gm_data, 5)?,
        generate_push_argument_var(gm_data, 4)?,
        generate_push_argument_var(gm_data, 3)?,
        generate_push_argument_var(gm_data, 2)?,
        generate_push_argument_var(gm_data, 1)?,
        GMInstruction::PushImmediate(0),    // animated sprite texture index
        GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Variable }),
        generate_push_argument_var(gm_data, 0)?,
        GMInstruction::Push(GMPushInstruction {value: GMCodeValue::Int32(ported_background_sprites_offset as i32)}),
        GMInstruction::Add(GMDoubleTypeInstruction { right: GMDataType::Variable, left: GMDataType::Variable }),    // index of sprite
        generate_call_builtin(gm_data, "draw_sprite_part_ext", 12)?,
        GMInstruction::Return(GMSingleTypeInstruction { data_type: GMDataType::Variable })
    ];
    generate_script(gm_data, "draw_background_part_ext", 10, instructions)?;

    let instructions = vec![
        generate_push_argument_var(gm_data, 2)?,
        generate_push_argument_var(gm_data, 1)?,
        GMInstruction::PushImmediate(0),    // animated sprite texture index
        GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Variable }),
        generate_push_argument_var(gm_data, 0)?,
        GMInstruction::Push(GMPushInstruction {value: GMCodeValue::Int32(ported_background_sprites_offset as i32)}),
        GMInstruction::Add(GMDoubleTypeInstruction { right: GMDataType::Variable, left: GMDataType::Variable }),    // index of sprite
        generate_call_builtin(gm_data, "draw_sprite", 4)?,
        GMInstruction::Return(GMSingleTypeInstruction { data_type: GMDataType::Variable })
    ];
    generate_script(gm_data, "draw_background", 3, instructions)?;

    // gms1
    generate_script(gm_data, "background_add", 3, RET_ZERO_STUB.to_vec())?;

    Ok(())
}

fn replace_action_funcs(gm_data: &mut GMData) -> Result<(), String> {
    let is_relative_var: GMRef<GMVariable> = gm_data.make_variable_b15("__action_is_relative", GMInstanceType::Global)?;

    let instructions = vec![
        generate_push_argument_var(gm_data, 0)?,
        GMInstruction::Pop(GMPopInstruction {
            type1: GMDataType::Variable,
            type2: GMDataType::Variable,
            destination: CodeVariable {
                variable: is_relative_var,
                variable_type: GMVariableType::Normal,
                instance_type: GMInstanceType::Global,
                is_int32: false,
            },
        }),
    ];
    generate_script(gm_data, "action_set_relative", 1, instructions)?;

    let instructions = vec![
        // if global.__action_is_relative {
        //     x += argument0
        //     y += argument1
        // } else {
        //     x = argument0
        //     y = argument1
        // }
        GMInstruction::Push(GMPushInstruction { value: GMCodeValue::Variable(CodeVariable {
            variable: is_relative_var,
            variable_type: GMVariableType::Normal,
            instance_type: GMInstanceType::Global,
            is_int32: false,
        }) }),
        GMInstruction::BranchUnless(GMGotoInstruction { jump_offset: 15 }),   // TODO check correctness

        generate_push_var(gm_data, "x", GMInstanceType::Builtin)?,
        generate_push_argument_var(gm_data, 0)?,
        GMInstruction::Add(GMDoubleTypeInstruction { right: GMDataType::Variable, left: GMDataType::Variable }),
        generate_pop_builtin_var(gm_data, "x")?,

        generate_push_var(gm_data, "y", GMInstanceType::Builtin)?,
        generate_push_argument_var(gm_data, 1)?,
        GMInstruction::Add(GMDoubleTypeInstruction { right: GMDataType::Variable, left: GMDataType::Variable }),
        generate_pop_builtin_var(gm_data, "y")?,
        GMInstruction::Branch(GMGotoInstruction { jump_offset: 8 }),

        generate_push_argument_var(gm_data, 0)?,
        generate_pop_builtin_var(gm_data, "x")?,
        generate_push_argument_var(gm_data, 1)?,
        generate_pop_builtin_var(gm_data, "y")?,
    ];
    generate_script(gm_data, "action_move_to", 2, instructions)?;

    // TODO:
    generate_script(gm_data, "action_move", 2, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "action_move_point", 3, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "action_set_motion", 2, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "action_previous_room", 0, RET_ZERO_STUB.to_vec())?;

    let instructions = vec![
        generate_call_builtin(gm_data, "instance_destroy", 0)?,
    ];
    generate_script(gm_data, "action_kill_object", 0, instructions)?;


    let instructions = vec![
        generate_push_argument_var(gm_data, 0)?,
        generate_push_argument_var(gm_data, 2)?,
        generate_push_argument_var(gm_data, 1)?,
        generate_call_builtin(gm_data, "instance_create", 3)?,
    ];
    generate_script(gm_data, "action_create_object", 3, instructions)?;


    let instructions = vec![
        generate_push_argument_var(gm_data, 0)?,
        GMInstruction::PushImmediate(-1),   // Instance Type: Self
        generate_push_argument_var(gm_data, 1)?,
        GMInstruction::Pop(GMPopInstruction {
            type1: GMDataType::Variable,
            type2: GMDataType::Variable,
            destination: CodeVariable {
                variable: gm_data.make_variable_b15("alarm", GMInstanceType::Builtin)?,
                variable_type: GMVariableType::Array,
                instance_type: GMInstanceType::Self_(None),
                is_int32: false,
            },
        })
    ];
    generate_script(gm_data, "action_set_alarm", 2, instructions)?;

    let instructions = vec![
        generate_push_argument_var(gm_data, 0)?,
        generate_pop_builtin_var(gm_data, "gravity_direction")?,
        generate_push_argument_var(gm_data, 1)?,
        generate_pop_builtin_var(gm_data, "gravity")?,
    ];
    generate_script(gm_data, "action_set_gravity", 2, instructions)?;

    let instructions = vec![
        generate_push_argument_var(gm_data, 0)?,
        generate_pop_builtin_var(gm_data, "hspeed")?,
    ];
    generate_script(gm_data, "action_set_hspeed", 1, instructions)?;

    let instructions = vec![
        generate_push_argument_var(gm_data, 0)?,
        generate_pop_builtin_var(gm_data, "vspeed")?,
    ];
    generate_script(gm_data, "action_set_vspeed", 1, instructions)?;

    let instructions = vec![
        generate_push_argument_var(gm_data, 0)?,
        generate_pop_builtin_var(gm_data, "friction")?,
    ];
    generate_script(gm_data, "action_set_friction", 1, instructions)?;

    Ok(())
}


fn replace_joystick_funcs(gm_data: &mut GMData) -> Result<(), String> {
    // TODO
    generate_script(gm_data, "joystick_has_pov", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "joystick_buttons", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "joystick_check_button", 2, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "joystick_exists", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "joystick_xpos", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "joystick_ypos", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "joystick_direction", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "joystick_pov", 1, RET_ZERO_STUB.to_vec())?;
    Ok(())
}


fn replace_layer_funcs(gm_data: &mut GMData) -> Result<(), String> {
    // TODO
    generate_script(gm_data, "tile_layer_hide", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "tile_layer_show", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "tile_layer_shift", 3, RET_ZERO_STUB.to_vec())?;
    Ok(())
}


/// Replaces the steam cloud data functions with stub functions that do nothing.
fn generate_steam_stubs(gm_data: &mut GMData) -> Result<(), String> {
    generate_script(gm_data, "steam_initialised", 0, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "steam_file_exists", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "steam_file_delete", 1, RET_ZERO_STUB.to_vec())?;
    generate_script(gm_data, "steam_file_write_file", 2, RET_ZERO_STUB.to_vec())?;

    Ok(())
}


const RET_ZERO_STUB: &[GMInstruction] = &[
    GMInstruction::PushImmediate(0),
    GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Variable }),
    GMInstruction::Return(GMSingleTypeInstruction { data_type: GMDataType::Variable })
];


fn generate_script(gm_data: &mut GMData, name: &str, arguments_count: u16, instructions: Vec<GMInstruction>) -> Result<(), String> {
    // TODO: code locals (FUNC code locals and maybe `arguments`?)
    let name_ref: GMRef<String> = gm_data.make_string(name);

    let code_ref: GMRef<GMCode> = GMRef::new(gm_data.codes.codes.len() as u32);
    let code = GMCode {
        name: name_ref,
        instructions,
        bytecode15_info: Some(GMCodeBytecode15 {
            locals_count: 0,
            arguments_count,
            weird_local_flag: false,
            offset: 0,
            parent: None,
        }),
    };
    gm_data.codes.codes.push(code);

    gm_data.scripts.scripts.push(GMScript {
        name: name_ref,
        is_constructor: false,
        code: Some(code_ref),
    });

    Ok(())
}


fn generate_push_argument_var(gm_data: &mut GMData, index: u8) -> Result<GMInstruction, String> {
    let name: String = format!("argument{index}");
    let code_variable = CodeVariable {
        variable: gm_data.make_variable_b15(&name, GMInstanceType::Argument)?,
        variable_type: GMVariableType::Normal,
        instance_type: GMInstanceType::Argument,
        is_int32: false,
    };
    Ok(GMInstruction::PushBuiltin(GMPushInstruction {
        value: GMCodeValue::Variable(code_variable)
    }))
}


fn generate_push_var(gm_data: &mut GMData, name: &str, instance_type: GMInstanceType) -> Result<GMInstruction, String> {
    let code_variable = CodeVariable {
        variable: gm_data.make_variable_b15(name, instance_type.clone())?,
        variable_type: GMVariableType::Normal,
        instance_type,
        is_int32: false,
    };
    Ok(GMInstruction::Push(GMPushInstruction {
        value: GMCodeValue::Variable(code_variable)
    }))
}


fn generate_pop_builtin_var(gm_data: &mut GMData, name: &str) -> Result<GMInstruction, String> {
    Ok(GMInstruction::Pop(GMPopInstruction {
        type1: GMDataType::Variable,
        type2: GMDataType::Variable,
        destination: CodeVariable {
            variable: gm_data.make_variable_b15(name, GMInstanceType::Builtin)?,
            variable_type: GMVariableType::Normal,
            instance_type: GMInstanceType::Self_(None),
            is_int32: false,
        },
    }))
}


fn generate_call_builtin(gm_data: &mut GMData, function_name: &'static str, arguments_count: u8) -> Result<GMInstruction, String> {
    Ok(GMInstruction::Call(GMCallInstruction {
        arguments_count,
        data_type: GMDataType::Int32,
        function: gm_data.make_builtin_function(function_name)?,
    }))
}

