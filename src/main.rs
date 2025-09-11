#![deny(unused_must_use)]
#![deny(unreachable_patterns)]
#![deny(unused_assignments)]
#![deny(unused_macros)]
#![deny(clippy::all)]

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

mod gamemaker;
mod modding;
mod utility;
mod csharp_rng;
pub mod gml;

use std::path::Path;
use std::process::exit;
use crate::utility::Stopwatch;


fn read_data_file(data_file_path: &Path) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let data: Vec<u8> = std::fs::read(data_file_path)
        .map_err(|e| format!("Could not read data file with path \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Reading data file took {stopwatch}");
    Ok(data)
}

fn write_data_file(data: Vec<u8>, data_file_path: &Path) -> Result<(), String> {
    let stopwatch = Stopwatch::start();
    std::fs::write(data_file_path, data)
        .map_err(|e| format!("Could not write data file with path \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

fn path_from_arg<'a>(arg: Option<&'a String>, default: &'a str) -> &'a Path {
    Path::new(arg.map_or(default, |s| s))
}


fn main_open_and_close() -> Result<(), String> {
    use crate::gamemaker::data::GMData;
    use crate::gamemaker::deserialize::parse_data_file;
    use crate::gamemaker::serialize::build_data_file;

    let args: Vec<String> = std::env::args().collect();
    let original_data_file_path: &Path = path_from_arg(args.get(1), "data.win");
    let modified_data_file_path: &Path = path_from_arg(args.get(2), "data_out.win");

    log::info!("Loading data file \"{}\"", original_data_file_path.display());
    let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
        .map_err(|e| format!("{e}\n↳ while reading data file"))?;

    log::info!("Parsing data file");
    let gm_data: GMData = parse_data_file(&original_data_raw, false)
        .map_err(|e| format!("\n{e}\n↳ while parsing data file"))?;
    drop(original_data_raw);
    
    // // sample changes
    // let mut gm_data = gm_data;
    // let original_name: &str = gm_data.general_info.display_name.resolve(&gm_data.strings.strings)?;
    // let modified_name: String = format!("{original_name} - Modded using AcornGM");
    // gm_data.general_info.display_name = gm_data.make_string(&modified_name);
    //
    // // export code disassembly
    // if !std::fs::exists("expasm").unwrap() {
    //     std::fs::create_dir("expasm").unwrap();
    // }
    // for code in &gm_data.codes.codes {
    //     let code_name = code.name.resolve(&gm_data.strings.strings)?;
    //     let assembly = gml::disassembler::disassemble_code(&gm_data, code)?;
    //     // println!("Disassembly of \"{code_name}\": \n{}", assembly);
    //     std::fs::write(format!("expasm/{code_name}.asm"), assembly)
    //         .map_err(|e| format!("Could not write assembly of code \"{code_name}\": {e}"))?;
    // }
    //
    // // export strings
    // let mut raw = String::new();
    // for i in 0..gm_data.strings.strings.len() {
    //     let string_ref = gamemaker::deserialize::GMRef::new(i as u32);
    //     let string = gml::disassembler::format_literal_string(&gm_data, string_ref)?;
    //     raw += &string;
    //     raw += "\n";
    // }
    // std::fs::write(format!("{}_strings.txt", original_data_file_path.to_str().unwrap()), raw)
    //     .map_err(|e| format!("Could not write string: {e}"))?;
    //
    // // find code blocks
    // for (i, code) in gm_data.codes.codes[46..].iter().enumerate() {
    //     let name = code.name.resolve(&gm_data.strings.strings)?;
    //     let blocks = gml::decompiler::blocks::find_basic_blocks(&code.instructions).map_err(|e| e.to_string())?;
    //     println!("{i} - {name}: \n{}\n", blocks.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("\n"));
    //     // std::hint::black_box(blocks);
    //     println!("{:?}", gml::decompiler::control_flow::idk(&blocks, 0));
    //     break
    // }

    // change gamemaker version
    let mut gm_data = gm_data;
    use gamemaker::gm_version::{GMVersion, LTSBranch};
    use gamemaker::elements;
    gm_data.general_info.version = GMVersion::new(2023, 6, 0, 0, LTSBranch::LTS);
    gm_data.general_info.bytecode_version = 17;
    gm_data.general_info.gms2_info = Some(elements::general_info::GMGeneralInfoGMS2 {
        random_uid: [0; 4],
        fps: 30.0,
        allow_statistics: false,
        game_guid: [0x2B, 0x3B, 0x8B, 0x85, 0x8B, 0xF1, 0x4B, 0x57, 0xB4, 0x11, 0x6C, 0xC9, 0x7D, 0x32, 0xF4, 0x93],
        info_timestamp_offset: false,
    });
    for background in &mut gm_data.backgrounds.backgrounds {
        let texture = background.texture.ok_or(format!("Background {} doesn't have a texture page item set", background.name.display(&gm_data.strings)))?;
        let texture = texture.resolve(&gm_data.texture_page_items.texture_page_items)?;
        background.gms2_data = Some(elements::backgrounds::GMBackgroundGMS2Data {
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
    for font in &mut gm_data.fonts.fonts {
        font.ascender_offset = Some(0);
        font.ascender = Some(0);
        font.sdf_spread = Some(0);
        font.line_height = Some(0);
    }
    for room in &mut gm_data.rooms.rooms {
        for obj in &mut room.game_objects {
            obj.image_index = Some(0);
            obj.image_speed = Some(1.0);
        }
        for tile in &mut room.tiles {
            let Some(tile_texture) = &tile.texture else {continue};
            let elements::rooms::GMRoomTileTexture::Background(background_ref) = tile_texture else {continue};
            let background = background_ref.resolve(&gm_data.backgrounds.backgrounds)?;
            let texture = background.texture.map(|t| t.resolve(&gm_data.texture_page_items.texture_page_items)).transpose()?;
            let new_sprite_ref = gamemaker::deserialize::GMRef::new(gm_data.sprites.sprites.len() as u32);
            gm_data.sprites.sprites.push(elements::sprites::GMSprite {
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
                sep_masks: elements::sprites::GMSpriteSepMaskType::AxisAlignedRect,
                // origin_x: texture.map_or(0, |t| t.source_x as i32),
                // origin_y: texture.map_or(0, |t| t.source_x as i32),
                origin_x: 0,
                origin_y: 0,
                textures: vec![background.texture],
                collision_masks: vec![],
                special_fields: Some(elements::sprites::GMSpriteSpecial {
                    special_version: 3,
                    data: elements::sprites::GMSpriteSpecialData::Normal,
                    playback_speed: 1.0,
                    playback_speed_type: elements::sequence::GMAnimSpeedType::FramesPerGameFrame,
                    sequence: None,
                    nine_slice: None,
                    yyswf: None,
                })
            });
            tile.texture = Some(elements::rooms::GMRoomTileTexture::Sprite(new_sprite_ref));
        }
    }
    for texture_page in &mut gm_data.embedded_textures.texture_pages {
        texture_page.generated_mips = Some(0);
        texture_page.texture_block_size = Some(0xDEADC0DE);  // will work if all texture pages are embedded
        let image = texture_page.image.as_ref().ok_or("External Texture pages are not supported")?;
        let image = image.to_dynamic_image()?;
        texture_page.data_2022_9 = Some(elements::embedded_textures::GMEmbeddedTexture2022_9 {
            texture_width: image.width() as i32,
            texture_height: image.height() as i32,
            index_in_group: 0,
        })
    }
    for obj in &mut gm_data.game_objects.game_objects {
        obj.managed = Some(false);
    }

    // build data file
    log::info!("Building data file");
    let modified_data_raw: Vec<u8> = build_data_file(&gm_data)
        .map_err(|e| format!("\n{e}\n↳ while building data file"))?;
    drop(gm_data);

    log::info!("Writing data file \"{}\"", modified_data_file_path.display());
    write_data_file(modified_data_raw, modified_data_file_path)
        .map_err(|e| format!("{e}\n↳ while writing data file"))?;

    Ok(())
}


// fn main_export_mod() -> Result<(), String> {
//     use crate::modding::export::{export_mod};
//     use crate::gamemaker::deserialize::{parse_data_file, GMData};
//     let args: Vec<String> = std::env::args().collect();
//     let original_data_file_path = path_from_arg(args.get(1), "data_original.win");
//     let modified_data_file_path = path_from_arg(args.get(2), "data_modified.win");
//     let mod_data_path = path_from_arg(args.get(3), "acornmod.tar.zst");
// 
//     log::info!("Loading original data file \"{}\"", original_data_file_path.display());
//     let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
//         .map_err(|e| format!("{e}\n↳ while reading original data file"))?;
// 
//     log::info!("Parsing original data file");
//     let original_data: GMData = parse_data_file(&original_data_raw, false)
//         .map_err(|e| format!("{e}\n↳ while parsing original data file"))?;
//     drop(original_data_raw);
// 
//     log::info!("Loading modified data file \"{}\"", modified_data_file_path.display());
//     let modified_data_raw: Vec<u8> = read_data_file(modified_data_file_path)
//         .map_err(|e| format!("{e}\n↳ while reading modified data file"))?;
// 
//     log::info!("Parsing modified data file");
//     let modified_data: GMData = parse_data_file(&modified_data_raw, false)
//         .map_err(|e| format!("{e}\n↳ while parsing modified data file"))?;
//     drop(modified_data_raw);
// 
//     log::info!("Extracting changes and exporting mod to file \"{}\"", mod_data_path.display());
//     export_mod(&original_data, &modified_data, mod_data_path)
//         .map_err(|e| format!("{e}\n↳ while exporting AcornGM mod"))?;
// 
//     Ok(())
// }


fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    log::debug!("============= LibGM v{} =============", env!("CARGO_PKG_VERSION"));
    
    if let Err(e) = main_open_and_close() {
        log::error!("{e}");
        exit(1);
    }

    log::info!("Done");
}

