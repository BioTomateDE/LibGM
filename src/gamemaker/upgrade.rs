use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::backgrounds::{GMBackground, GMBackgroundGMS2Data};
use crate::gamemaker::elements::code::{CodeVariable, GMCallInstruction, GMCode, GMCodeBytecode15, GMCodeValue, GMDataType, GMDoubleTypeInstruction, GMInstanceType, GMInstruction, GMPushInstruction, GMSingleTypeInstruction, GMVariableType};
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTexture2022_9;
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::general_info::GMGeneralInfoGMS2;
use crate::gamemaker::elements::rooms::GMRoomTileTexture;
use crate::gamemaker::elements::scripts::GMScript;
use crate::gamemaker::elements::sequence::GMAnimSpeedType::FramesPerGameFrame;
use crate::gamemaker::elements::sprites::{GMSprite, GMSpriteSepMaskType, GMSpriteSpecial, GMSpriteSpecialData, GMSprites};
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::gm_version::{GMVersion, LTSBranch};


/// Updates GameMaker project data to version 2022.9 LTS
pub fn migrate_to_gm_2022_9(gm_data: GMData) -> Result<GMData, String> {
    migrate_to_gm_2022_9_(gm_data).map_err(|e| format!("{e}\n↳ upgrading to GameMaker Version 2022.9 LTS"))
}

fn migrate_to_gm_2022_9_(mut gm_data: GMData) -> Result<GMData, String> {
    update_general_info(&mut gm_data);
    update_backgrounds(&mut gm_data)?;
    update_fonts(&mut gm_data);
    update_rooms(&mut gm_data)?;
    update_texture_pages(&mut gm_data)?;
    update_game_objects(&mut gm_data);
    replace_instance_create_calls(&mut gm_data)?;
    Ok(gm_data)
}


/// Updates general project information for GM 2022.9
fn update_general_info(gm_data: &mut GMData) {
    gm_data.general_info.version = GMVersion::new(2022, 9, 0, 0, LTSBranch::LTS);
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
            let sprite_ref: GMRef<GMSprite> = find_or_create_sprite_for_background(&mut gm_data.sprites, background, texture);

            // Update tile to use sprite instead of background
            tile.texture = Some(GMRoomTileTexture::Sprite(sprite_ref));
        }
    }

    Ok(())
}


/// Finds existing sprite with same name as background, or creates a new one
fn find_or_create_sprite_for_background(gm_sprites: &mut GMSprites, background: &GMBackground, texture: Option<&GMTexturePageItem>) -> GMRef<GMSprite> {
    // Try to find existing sprite with same name
    for (i, sprite) in gm_sprites.sprites.iter().enumerate() {
        if sprite.name == background.name {
            return GMRef::new(i as u32);
        }
    }

    // Create new sprite from background
    create_sprite_from_background(gm_sprites, background, texture)
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


/// Replaces all calls to the `instance_create` function with a call to the `instance_create_depth` function.
fn replace_instance_create_calls(gm_data: &mut GMData) -> Result<(), String> {
    let script_name: GMRef<String> = gm_data.make_string("AcornScript_instance_create");

    // create the script code entry
    let script_code_ref: GMRef<GMCode> = GMRef::new(gm_data.codes.codes.len() as u32);
    let script_code = GMCode {
        name: script_name,
        instructions: vec![
            // push.v arg.argument2
            // pushi.e 0
            // conv.i.v
            // push.v arg.argument1
            // push.v arg.argument0
            // call.i instance_create_depth(argc=4)
            // ret.v
            GMInstruction::Push(GMPushInstruction {value: GMCodeValue::Variable(CodeVariable {
                variable: gm_data.make_variable_b15("argument2", GMInstanceType::Argument)?,
                variable_type: GMVariableType::Normal,
                instance_type: GMInstanceType::Argument,
                is_int32: false,
            }) }),
            GMInstruction::PushImmediate(0),    // depth
            GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Variable }),
            GMInstruction::Push(GMPushInstruction {value: GMCodeValue::Variable(CodeVariable {
                variable: gm_data.make_variable_b15("argument1", GMInstanceType::Argument)?,
                variable_type: GMVariableType::Normal,
                instance_type: GMInstanceType::Argument,
                is_int32: false,
            }) }),
            GMInstruction::Push(GMPushInstruction {value: GMCodeValue::Variable(CodeVariable {
                variable: gm_data.make_variable_b15("argument0", GMInstanceType::Argument)?,
                variable_type: GMVariableType::Normal,
                instance_type: GMInstanceType::Argument,
                is_int32: false,
            }) }),
            GMInstruction::Call(GMCallInstruction {
                arguments_count: 4,
                data_type: GMDataType::Int32,
                function: gm_data.make_builtin_function("instance_create_depth")?,
            }),
            GMInstruction::Return(GMSingleTypeInstruction { data_type: GMDataType::Variable })
        ],
        bytecode15_info: Some(GMCodeBytecode15 {
            locals_count: 0,
            arguments_count: 3,
            weird_local_flag: false,
            offset: 0,
            parent: None,
        }),
    };
    gm_data.codes.codes.push(script_code);

    // create the matching script (TODO: idk if SCPT is even used by the runner...)
    // let script_ref = GMRef::new(gm_data.scripts.scripts.len() as u32);
    gm_data.scripts.scripts.push(GMScript {
        name: script_name,
        is_constructor: false,
        code: Some(script_code_ref),
    });

    // create matching function
    // let function_ref = GMRef::new(gm_data.functions.functions.len() as u32);
    gm_data.functions.functions.push(GMFunction {
        name: script_name,
    });

    // replace `instance_create` function entry
    for function in &mut gm_data.functions.functions  {
        let name: &String = function.name.resolve(&gm_data.strings.strings)?;
        if name == "instance_create" {
            function.name = script_name;
        }
    }

    Ok(())
}

