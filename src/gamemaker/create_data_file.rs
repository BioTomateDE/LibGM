use std::path::Path;
use image::{DynamicImage, ImageReader};
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::element::GMChunkElement;
use crate::gamemaker::elements::animation_curves::GMAnimationCurves;
use crate::gamemaker::elements::audio_groups::GMAudioGroups;
use crate::gamemaker::elements::backgrounds::GMBackgrounds;
use crate::gamemaker::elements::code::GMCodes;
use crate::gamemaker::elements::data_files::GMDataFiles;
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudios;
use crate::gamemaker::elements::embedded_images::GMEmbeddedImages;
use crate::gamemaker::elements::embedded_textures::{GMEmbeddedTexture, GMEmbeddedTexture2022_9, GMEmbeddedTextures, GMImage};
use crate::gamemaker::elements::extensions::GMExtensions;
use crate::gamemaker::elements::feature_flags::GMFeatureFlags;
use crate::gamemaker::elements::filter_effects::GMFilterEffects;
use crate::gamemaker::elements::fonts::GMFonts;
use crate::gamemaker::elements::functions::GMFunctions;
use crate::gamemaker::elements::game_objects::{GMGameObject, GMGameObjectCollisionShape, GMGameObjects};
use crate::gamemaker::elements::general_info::{GMFunctionClassifications, GMGeneralInfo, GMGeneralInfoFlags, GMGeneralInfoGMS2};
use crate::gamemaker::elements::global_init::{GMGameEndScripts, GMGlobalInitScripts};
use crate::gamemaker::elements::languages::GMLanguageInfo;
use crate::gamemaker::elements::options::{GMOptions, GMOptionsConstant, GMOptionsFlags};
use crate::gamemaker::elements::particles::{GMParticleEmitters, GMParticleSystems};
use crate::gamemaker::elements::paths::GMPaths;
use crate::gamemaker::elements::rooms::{GMRoom, GMRoomFlags, GMRoomGameObject, GMRooms};
use crate::gamemaker::elements::scripts::GMScripts;
use crate::gamemaker::elements::sequence::GMSequences;
use crate::gamemaker::elements::shaders::GMShaders;
use crate::gamemaker::elements::sounds::GMSounds;
use crate::gamemaker::elements::sprites::{GMSprite, GMSpriteSepMaskType, GMSprites};
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::tags::GMTags;
use crate::gamemaker::elements::texture_group_info::GMTextureGroupInfos;
use crate::gamemaker::elements::texture_page_items::{GMTexturePageItem, GMTexturePageItems};
use crate::gamemaker::elements::timelines::GMTimelines;
use crate::gamemaker::elements::ui_nodes::GMRootUINodes;
use crate::gamemaker::elements::variables::{GMVariables, GMVariablesB15Header};
use crate::gamemaker::gm_version::{GMVersion, LTSBranch};


/// TODO: make use of the `target_version` (more). Right now, it's just building for 2023 LTS.
pub fn new_data_file(target_version: GMVersion, target_bytecode: u8) -> GMData {
    let mut data: GMData = stub_data();
    data.general_info.exists = true;
    data.global_init_scripts.exists = true;
    data.audio_groups.exists = true;
    data.sounds.exists = true;
    data.sprites.exists = true;
    data.backgrounds.exists = true;
    data.paths.exists = true;
    data.scripts.exists = true;
    data.shaders.exists = true;
    data.fonts.exists = true;
    data.timelines.exists = true;
    data.game_objects.exists = true;
    data.rooms.exists = true;
    data.extensions.exists = true;
    data.texture_page_items.exists = true;
    data.codes.exists = true;
    data.variables.exists = true;
    data.functions.exists = true;
    data.functions.code_locals.exists = true;
    data.strings.exists = true;
    data.embedded_textures.exists = true;
    data.language_info.exists = true;

    data.chunk_padding = 16;    // just to be safe i guess

    data.strings = GMStrings {
        strings: vec![],
        is_aligned: true,   // just to be safe i guess
        exists: true,
    };

    data.general_info = GMGeneralInfo {
        is_debugger_disabled: true,
        bytecode_version: target_bytecode,
        unknown_value: 0,
        game_file_name: data.make_string("new_acorngm_game"),
        config: data.make_string("Default"),
        last_object_id: 100_001,
        last_tile_id: 10_000_000,
        game_id: 0,
        directplay_guid: uuid::Builder::from_bytes([0; 16]).into_uuid(),
        game_name: data.make_string("New AcornGM Game"),
        version: GMVersion {
            major: 2023,
            minor: 6,
            release: 0,
            build: 0,
            branch: LTSBranch::LTS,
        },
        default_window_width: 800,
        default_window_height: 600,
        flags: GMGeneralInfoFlags {
            fullscreen: false,
            sync_vertex1: false,
            sync_vertex2: false,
            sync_vertex3: false,
            interpolate: false,
            scale: true,
            show_cursor: true,
            sizeable: false,
            screen_key: false,
            studio_version_b1: false,
            studio_version_b2: false,
            studio_version_b3: false,
            steam_enabled: false,
            local_data_enabled: false,
            borderless_window: false,
            javascript_mode: false,
            license_exclusions: false,
        },
        license_crc32: 0,
        license_md5: [0; 16],
        timestamp_created: chrono::offset::Utc::now(),
        display_name: data.make_string("New AcornGM Game"),
        active_targets: 0,
        function_classifications: GMFunctionClassifications {
            none: true,
            internet: false,
            joystick: false,
            gamepad: true,
            immersion: false,
            screengrab: true,
            math: true,
            action: false,
            matrix_d3d: false,
            d3dmodel: false,
            data_structures: true,
            file: true,
            ini: false,
            filename: false,
            directory: false,
            environment: false,
            unused1: false,
            http: false,
            encoding: true,
            uidialog: true,
            motion_planning: false,
            shape_collision: true,
            instance: true,
            room: true,
            game: true,
            display: false,
            device: false,
            window: true,
            draw_color: true,
            texture: false,
            layer: false,
            string: true,
            tiles: false,
            surface: true,
            skeleton: false,
            io: true,
            variables: true,
            array: true,
            external_call: false,
            notification: false,
            date: true,
            particle: false,
            sprite: true,
            clickable: false,
            legacy_sound: false,
            audio: false,
            event: true,
            unused2: false,
            free_type: true,
            analytics: false,
            unused3: false,
            unused4: false,
            achievement: false,
            cloud_saving: false,
            ads: false,
            os: false,
            iap: false,
            facebook: false,
            physics: false,
            flash_aa: false,
            console: false,
            buffer: true,
            steam: false,
            unused5: false,
            shaders: false,
            vertex_buffers: false,
        },
        steam_appid: 0,
        debugger_port: if target_bytecode >= 14 { Some(0) } else { None },
        room_order: vec![GMRef::new(0)],
        gms2_info: if target_version.is_version_at_least((2, 0)) {
            Some(GMGeneralInfoGMS2 {
                random_uid: [0i64; 4],
                fps: 60.0,
                allow_statistics: false,
                game_guid: [0u8; 16],
                info_timestamp_offset: false,
            })
        } else { None },
        exists: true,
    };

    data.options = GMOptions {
        is_new_format: true,
        unknown1: 2147483648,
        unknown2: 2,
        flags: GMOptionsFlags {
            fullscreen: false,
            interpolate_pixels: false,
            use_new_audio: true,
            no_border: false,
            show_cursor: true,
            sizeable: false,
            stay_on_top: false,
            change_resolution: false,
            no_buttons: false,
            screen_key: false,
            help_key: false,
            quit_key: false,
            save_key: false,
            screenshot_key: false,
            close_sec: false,
            freeze: false,
            show_progress: false,
            load_transparent: false,
            scale_progress: false,
            display_errors: false,
            write_errors: false,
            abort_errors: false,
            variable_errors: true,
            creation_event_order: false,
            use_front_touch: false,
            use_rear_touch: false,
            use_fast_collision: false,
            fast_collision_compatibility: false,
            disable_sandbox: false,
            enable_copy_on_write: false,
        },
        window_scale: 1,
        window_color: 0,
        color_depth: 0,
        resolution: 0,
        frequency: 0,
        vertex_sync: 0,
        priority: 0,
        back_image: None,
        front_image: None,
        load_image: None,
        load_alpha: 0,
        constants: vec![
            GMOptionsConstant { name: data.make_string("@@SleepMargin"), value: data.make_string("0") },
            GMOptionsConstant { name: data.make_string("@@DrawColour"), value: data.make_string("4294967295") },
        ],
        exists: true,
    };

    import_texture_page(&mut data, Path::new("/home/biotomatede/Pictures/ut_txt/dr3_placeholder1.png"));

    data.texture_page_items.texture_page_items.push(GMTexturePageItem {
        source_x: 0,
        source_y: 0,
        source_width: 256,
        source_height: 256,
        target_x: 0,
        target_y: 0,
        target_width: 256,
        target_height: 256,
        bounding_width: 0,
        bounding_height: 0,
        texture_page: GMRef::new(0),
    });

    let sprite = GMSprite {
        name: data.make_string("spr_whatever"),
        width: 256,
        height: 256,
        margin_left: 0,
        margin_right: 0,
        margin_bottom: 0,
        margin_top: 0,
        transparent: false,
        smooth: true,
        preload: true,
        bbox_mode: 0,
        sep_masks: GMSpriteSepMaskType::AxisAlignedRect,
        origin_x: 0,
        origin_y: 0,
        textures: vec![Some(GMRef::new(0))],
        collision_masks: vec![],
        special_fields: None,
    };
    data.sprites.sprites.push(sprite);

    let game_object = GMGameObject {
        name: data.make_string("obj_whatever"),
        sprite: Some(GMRef::new(0)),
        visible: true,
        managed: Some(false),
        solid: false,
        depth: 0,
        persistent: false,
        parent: None,
        texture_mask: None,
        uses_physics: false,
        is_sensor: false,
        collision_shape: GMGameObjectCollisionShape::Box,
        density: 0.0,
        restitution: 0.0,
        group: 0,
        linear_damping: 0.0,
        angular_damping: 0.0,
        friction: 0.0,
        awake: false,
        kinematic: false,
        physics_shape_vertices: vec![],
        uses_physics_shape_vertex: false,
        events: vec![],
    };
    data.game_objects.game_objects.push(game_object);

    data.rooms = GMRooms {
        rooms: vec![
            GMRoom {
                name: data.make_string("Room1"),
                caption: None,
                width: 800,
                height: 600,
                speed: 0,
                persistent: false,
                background_color: 0,
                draw_background_color: false,
                creation_code: None,
                flags: GMRoomFlags {
                    enable_views: false,
                    show_color: false,
                    dont_clear_display_buffer: false,
                    is_gms2: true,
                    is_gms2_3: true,
                },
                backgrounds: vec![],
                views: vec![],
                game_objects: vec![
                    GMRoomGameObject {
                        x: 67,
                        y: 41,
                        object_definition: GMRef::new(0),
                        instance_id: 100_000,
                        creation_code: None,
                        scale_x: 1.0,
                        scale_y: 1.0,
                        image_speed: Some(1.0),
                        image_index: Some(0),
                        color: 0,
                        rotation: 0.0,
                        pre_create_code: None,
                    }
                ],
                tiles: vec![],
                instance_creation_order_ids: vec![],
                world: false,
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                gravity_x: 0.0,
                gravity_y: 0.0,
                meters_per_pixel: 0.0,
                layers: vec![], // TODO?
                sequences: vec![],
            }
        ],
        exists: true,
    };

    data.language_info = GMLanguageInfo {
        unknown1: 1,
        languages: vec![],
        entry_ids: vec![],
        exists: true,
    };

    data.variables.b15_header = Some(GMVariablesB15Header {
        var_count1: 0,
        var_count2: 0,
        max_local_var_count: 0,
    });
    data.fonts.padding = Some(generate_font_padding());

    data
}


fn stub_data() -> GMData {
    GMData {
        general_info: GMGeneralInfo::stub(),
        strings: GMStrings::stub(),
        embedded_textures: GMEmbeddedTextures::stub(),
        texture_page_items: GMTexturePageItems::stub(),
        variables: GMVariables::stub(),
        functions: GMFunctions::stub(),
        scripts: GMScripts::stub(),
        codes: GMCodes::stub(),
        fonts: GMFonts::stub(),
        sprites: GMSprites::stub(),
        game_objects: GMGameObjects::stub(),
        rooms: GMRooms::stub(),
        backgrounds: GMBackgrounds::stub(),
        paths: GMPaths::stub(),
        audios: GMEmbeddedAudios::stub(),
        sounds: GMSounds::stub(),
        options: GMOptions::stub(),
        sequences: GMSequences::stub(),
        particle_systems: GMParticleSystems::stub(),
        particle_emitters: GMParticleEmitters::stub(),
        language_info: GMLanguageInfo::stub(),
        extensions: GMExtensions::stub(),
        audio_groups: GMAudioGroups::stub(),
        global_init_scripts: GMGlobalInitScripts::stub(),
        game_end_scripts: GMGameEndScripts::stub(),
        shaders: GMShaders::stub(),
        root_ui_nodes: GMRootUINodes::stub(),
        data_files: GMDataFiles::stub(),
        timelines: GMTimelines::stub(),
        embedded_images: GMEmbeddedImages::stub(),
        texture_group_infos: GMTextureGroupInfos::stub(),
        tags: GMTags::stub(),
        feature_flags: GMFeatureFlags::stub(),
        filter_effects: GMFilterEffects::stub(),
        animation_curves: GMAnimationCurves::stub(),
        chunk_padding: 0,
        is_big_endian: false,
        original_data_size: 0,
    }
}


const fn generate_font_padding() -> [u8; 512] {
    let mut data = [0u8; 512];
    let mut i = 0u16;
    let mut idx = 0;

    // First 128 values: 0..128 as u16 little-endian
    while i < 0x80 {
        let bytes = i.to_le_bytes();
        data[idx] = bytes[0];
        data[idx + 1] = bytes[1];
        i += 1;
        idx += 2;
    }

    // Next 128 values: all 0x3f as u16 little-endian
    let value_bytes = 0x3fu16.to_le_bytes();
    let mut count = 0;
    while count < 0x80 {
        data[idx] = value_bytes[0];
        data[idx + 1] = value_bytes[1];
        count += 1;
        idx += 2;
    }

    data
}


fn import_texture_page(gm_data: &mut GMData, image_path: &Path) {
    let image: DynamicImage = ImageReader::open(image_path).unwrap().decode().unwrap();
    gm_data.embedded_textures.texture_pages.push(GMEmbeddedTexture {
        scaled: 0,
        generated_mips: Some(0),
        texture_block_size: Some(0xDEADC0DE),
        data_2022_9: Some(GMEmbeddedTexture2022_9 {
            texture_width: image.width() as i32,
            texture_height: image.height() as i32,
            index_in_group: 0,
        }),
        image: Some(GMImage::DynImg(image)),
    });
}

