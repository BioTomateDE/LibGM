use libgm::wad::GMData;
use libgm::wad::elem::general_info::GeneralInfo;
use libgm::wad::elem::string::Strings;

pub fn print_info(data: &GMData) {
    print_general(&data.general_info, &data.strings);
}

pub fn print_general(g: &GeneralInfo, s: &Strings) {
    // println!("======== General Info ========");
    println!("Display Name: {}", g.display_name.display(s));
    println!("Game Name: {}", g.game_name.display(s));
    println!("File Name: {}", g.game_file_name.display(s));
    println!("Version: {} (WAD {})", g.version, g.wad_version);
    println!("Created at: {:?}", g.creation_timestamp);
    println!("Window dimensions: {}x{}", g.window_width, g.window_height);
    println!("Config: {}", g.config.display(s));
    println!(
        "Last object/tile ID: {}, {}",
        g.last_object_id, g.last_tile_id
    );
    println!("Room count: {}", g.room_order.len());
    if g.debugger_enabled {
        println!("GameMaker debugger is enabled");
    }
    if g.debugger_port != 0 {
        println!("Debugger Port: {}", g.debugger_port);
    }
    if g.steam_appid != 0 {
        println!("Steam App ID: {}", g.steam_appid);
    }
}
