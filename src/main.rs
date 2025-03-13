use std::collections::HashMap;
use std::{fs, process};

struct UTChunk {
    name: String,
    data: Vec<u8>,
}

impl UTChunk {
    fn apply_changes(&self, mut changes: Vec<DataChange>) {
        changes.sort_by(|a, b| b.index.cmp(&a.index));
        for change in changes {
            println!(
                "[DataChange @ {}] Index: {} | Len: {} | Delete: {}",
                self.name,
                change.index,
                change.content.len(),
                change.delete
            );
            change.apply(self.data.clone());
        }
    }
}

struct UTGeneralInfo {
    is_debugger_disabled: bool,
    bytecode_version: u8,
    unknown_value: u16,
    game_file_name: String,
    config: String,
    last_object_id: u32,
    last_tile_id: u32,
    game_id: u32,
    game_name: String,
    major_version: u32,
    minor_version: u32,
    release_version: u32,
    stable_version: u32,
    default_window_width: u32,
    default_window_height: u32,
    flags: UTGeneralInfoFlags,
    display_name: String,
    active_targets: u64,
    function_classifications: UTGeneralInfoFunctionClassifications,
    steam_appid: u32,
    debugger_port: u16,
}

struct UTGeneralInfoFlags {
    // taken from https://github.com/UnderminersTeam/UndertaleModTool/blob/master/UndertaleModLib/Models/UndertaleGeneralInfo.cs
    fullscreen: bool,
    sync_vertex1: bool,
    sync_vertex2: bool,
    sync_vertex3: bool,
    interpolate: bool,
    scale: bool,
    show_curser: bool,
    sizeable: bool,
    screen_key: bool,
    studio_version_b1: bool,
    studio_version_b2: bool,
    studio_version_b3: bool,
    // studio_version_mask
    steam_enabled: bool,
    local_data_enabled: bool,
    borderless_window: bool,
    javascript_mode: bool,
    // license_exclusions: bool,
}

struct UTGeneralInfoFunctionClassifications {
    none: bool,
    internet: bool,
    joystick: bool,
    gamepad: bool,
    immersion: bool,
    screengrab: bool,
    math: bool,
    action: bool,
    matrix_d3d: bool,
    d3dmodel: bool,
    data_structures: bool,
    file: bool,
    ini: bool,
    filename: bool,
    directory: bool,
    environment: bool,
    unused1: bool,
    http: bool,
    encoding: bool,
    uidialog: bool,
    motion_planning: bool,
    shape_collision: bool,
    instance: bool,
    room: bool,
    game: bool,
    display: bool,
    device: bool,
    window: bool,
    draw_color: bool,
    texture: bool,
    layer: bool,
    string: bool,
    tiles: bool,
    surface: bool,
    skeleton: bool,
    io: bool,
    variables: bool,
    array: bool,
    external_call: bool,
    notification: bool,
    date: bool,
    particle: bool,
    sprite: bool,
    clickable: bool,
    legacy_sound: bool,
    audio: bool,
    event: bool,
    unused2: bool,
    free_type: bool,
    analytics: bool,
    unused3: bool,
    unused4: bool,
    achievement: bool,
    cloud_saving: bool,
    ads: bool,
    os: bool,
    iap: bool,
    facebook: bool,
    physics: bool,
    flash_aa: bool,
    console: bool,
    buffer: bool,
    steam: bool,
    unused5: bool,
    shaders: bool,
    vertex_buffers: bool,
}

struct UTData {
    chunks: HashMap<String, UTChunk>, // remove when all chunks parsed properly
    strings: HashMap<u32, String>,    // STRG
    general_info: UTGeneralInfo,      // GEN8
}

struct DataChange {
    index: usize,
    content: Vec<u8>,
    delete: bool,
}

impl DataChange {
    fn apply(&self, data: Vec<u8>) {
        if self.delete {
            let _ = self.__delete(data);
        } else {
            self.__insert(data)
        }
    }

    fn __insert(&self, mut data: Vec<u8>) {
        data.splice(self.index..self.index, self.content.clone());
    }

    fn __delete(&self, mut data: Vec<u8>) -> Result<(), String> {
        let len: usize = self.content.len();
        if data[self.index..self.index + len] != self.content {
            return Err(format!(
                "Could not delete {} bytes at position {} because they dont exist in the code at the specified location!",
                len, self.index
            ));
        }
        data.splice(self.index..self.index + len, []);
        Ok(())
    }
}

fn read_data_file(data_file_path: &str) -> Vec<u8> {
    let data_file = match fs::read(data_file_path) {
        Ok(file) => file,
        Err(error) => {
            panic!("Could not read file: {error:?}");
        }
    };
    data_file
}

fn chunk_read_u64(raw_data: &Vec<u8>, offset: usize) -> u64 {
    // Read unsigned 64-bit integer from raw file data (little endian)
    let mut length: u64 = 0;
    for i in (0..8).rev() {
        length <<= 8;
        length |= u64::from(raw_data[offset + i]);
    }
    length
}

fn chunk_read_i32(raw_data: &Vec<u8>, offset: usize) -> i32 {
    // Read signed 32-bit integer from raw file data (little endian)
    let mut length: i32 = 0;
    for i in (0..4).rev() {
        length <<= 8;
        length |= i32::from(raw_data[offset + i]);
    }
    length
}

fn chunk_read_u32(raw_data: &Vec<u8>, offset: usize) -> u32 {
    // Read unsigned 32-bit integer from raw file data (little endian)
    let mut length: u32 = 0;
    for i in (0..4).rev() {
        length <<= 8;
        length |= u32::from(raw_data[offset + i]);
    }
    length
}

fn chunk_read_u16(raw_data: &Vec<u8>, offset: usize) -> u16 {
    // Read unsigned 16-bit integer from raw file data (little endian)
    let mut length: u16 = 0;
    for i in (0..2).rev() {
        length <<= 8;
        length |= u16::from(raw_data[offset + i]);
    }
    length
}

fn read_chunk_name(raw_data: &Vec<u8>, offset: usize) -> String {
    let string = raw_data[offset..offset + 4].to_owned();
    let string = match String::from_utf8(string) {
        Ok(string) => string,
        Err(error) => {
            panic!("Invalid or corrupted data.win file (could not parse chunk name): {error}");
        }
    };
    string
}

fn chunk_read_string(raw_data: &Vec<u8>, offset: usize, length: usize) -> String {
    let string = raw_data[offset..offset + length].to_owned();
    let string = match String::from_utf8(string) {
        Ok(string) => string,
        Err(error) => {
            panic!("Invalid or corrupted data.win file (could not parse string): {error}");
        }
    };
    string
}

fn print_general_info(general_info: &UTGeneralInfo) {
    println!("\nGeneral Info:");
    println!(
        "  GMS Debugger Disabled: {}",
        general_info.is_debugger_disabled
    );
    println!("  Bytecode Version: {}", general_info.bytecode_version);
    println!("  File Name: {}", general_info.game_file_name);
    println!("  Config: {}", general_info.config);
    println!("  Last object ID: {}", general_info.last_object_id);
    println!("  Last tile ID: {}", general_info.last_tile_id);
    println!("  Game ID: {}", general_info.game_id);
    println!("  Game Name: {}", general_info.game_name);
    println!(
        "  Version: {}.{}.{}.{}",
        general_info.major_version,
        general_info.minor_version,
        general_info.release_version,
        general_info.stable_version
    );
    println!(
        "  Default Window Size: {}x{}",
        general_info.default_window_width, general_info.default_window_height
    );
    // println!("  Flags: {}", general_info.flags.iNoNoWanna);
    println!("  Display Name: {}", general_info.display_name);
    println!("  Active Targets: {}", general_info.active_targets);
    // println!("  Function Classifications: {}", general_info.function_classifications.iNoNoWanna);
    println!("  Steam AppID: {}", general_info.steam_appid);
    println!("  Debugger Port: {}", general_info.debugger_port);
    println!();
}

fn parse_data_file(raw_data: Vec<u8>) -> Result<UTData, String> {
    if read_chunk_name(&raw_data, 0) != "FORM" {
        return Err(String::from(
            "Invalid or corrupted data.win file: 'FORM' chunk missing!",
        ));
    }

    // let total_length: usize = chunk_read_u32(&raw_data, 4) as usize;

    // get chunks
    let mut index: usize = 8;
    let raw_data_len: usize = raw_data.len();
    let mut chunks: HashMap<String, UTChunk> = HashMap::new();

    while index + 8 < raw_data_len {
        let chunk_name: String = read_chunk_name(&raw_data, index);
        index += 4;
        let chunk_length: usize = chunk_read_u32(&raw_data, index) as usize;
        index += 4;
        let chunk_data: Vec<u8> = raw_data[index..index + chunk_length].to_owned();
        chunks.insert(
            chunk_name.clone(),
            UTChunk {
                data: chunk_data,
                name: chunk_name,
            },
        );
        index += chunk_length;
    }

    let strings = parse_chunk_STRG(&chunks["STRG"]);
    let general_info: UTGeneralInfo = parse_chunk_GEN8(&chunks["GEN8"], &strings);

    let data = UTData {
        chunks,
        strings,
        general_info,
    };

    // println!("Total data length: {total_length} bytes");
    // println!("Chunk Sizes:");
    // for (chunk_name, chunk) in &data.chunks {
    //     println!("  {}: {} bytes", chunk_name, chunk.data.len());
    // }

    // testong
    // for (chunk_name, chunk) in &data.chunks {
    //     let path = format!("./_expdat/{chunk_name}.bin");
    //     match fs::write(path, chunk.data.clone()) {
    //         Ok(_) => (),
    //         Err(err) => eprintln!("Failed to write to file for {chunk_name}: {}", err),
    //     }
    // }
    // ^

    Ok(data)
}

fn parse_chunk_STRG(chunk: &UTChunk) -> HashMap<u32, String> {
    let mut file_index = 0;
    let string_count: usize = chunk_read_u32(&chunk.data, 0) as usize;
    file_index += 4;
    let mut string_ids: Vec<u32> = Vec::with_capacity(string_count);
    let mut strings: HashMap<u32, String> = HashMap::with_capacity(string_count);

    for _ in 0..string_count {
        // you have to add 4 to the string id for some unknown reason
        let string_id = 4 + chunk_read_u32(&chunk.data, file_index);
        file_index += 4;
        string_ids.push(string_id);
    }

    for string_id in string_ids {
        let string_length: usize = chunk_read_u32(&chunk.data, file_index) as usize;
        file_index += 4;
        let string = chunk_read_string(&chunk.data, file_index, string_length);
        file_index += string_length + 1; // add one for the null byte after the string
        strings.insert(string_id, string);
    }
    strings
}

fn parse_chunk_GEN8(chunk: &UTChunk, strings: &HashMap<u32, String>) -> UTGeneralInfo {
    let mut file_index: usize = 0;

    let is_debugger_disabled: bool = chunk.data[file_index] != 0;
    file_index += 1;

    let bytecode_version: u8 = chunk.data[file_index];
    file_index += 1;

    let unknown_value: u16 = chunk_read_u16(&chunk.data, file_index);
    file_index += 2;

    let game_file_name: String = strings[&chunk_read_u32(&chunk.data, file_index)].clone();
    file_index += 4;

    let config: String = strings[&chunk_read_u32(&chunk.data, file_index)].clone();
    file_index += 4;

    let last_object_id: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let last_tile_id: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let game_id: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    // skip 16 bytes (DirectPlay GUID)
    file_index += 16;

    let game_name: String = strings[&chunk_read_u32(&chunk.data, file_index)].clone();
    file_index += 4;

    let major_version: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let minor_version: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let release_version: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let stable_version: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let default_window_width: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let default_window_height: u32 = chunk_read_u32(&chunk.data, file_index);
    file_index += 4;

    let flags: UTGeneralInfoFlags = parse_flags(&chunk.data, file_index);
    file_index += 8;

    // skip 16 bytes (License MD5)
    file_index += 16;

    // skip 8 bytes (Timestamp created)
    file_index += 8;

    let display_name: String = strings[&chunk_read_u32(&chunk.data, file_index)].clone();
    file_index += 4;

    // probably not actually u64 (rather u32) but it's zero and there's null bytes surrounding it so idk
    let active_targets: u64 = chunk_read_u64(&chunk.data, file_index);
    file_index += 8;

    let function_classifications: UTGeneralInfoFunctionClassifications =
        parse_function_classifications(&chunk.data, file_index);
    file_index += 8;

    let steam_appid: u32 = (-chunk_read_i32(&chunk.data, file_index)) as u32;
    file_index += 4;

    let debugger_port: u16 = chunk_read_u16(&chunk.data, file_index);
    file_index += 4;

    UTGeneralInfo {
        is_debugger_disabled,
        bytecode_version,
        unknown_value,
        game_file_name,
        config,
        last_object_id,
        last_tile_id,
        game_id,
        game_name,
        major_version,
        minor_version,
        release_version,
        stable_version,
        default_window_width,
        default_window_height,
        flags,
        display_name,
        active_targets,
        function_classifications,
        steam_appid,
        debugger_port,
    }
}

fn parse_flags(raw_data: &Vec<u8>, offset: usize) -> UTGeneralInfoFlags {
    let raw: u64 = chunk_read_u64(raw_data, offset);
    UTGeneralInfoFlags {
        fullscreen: 0 != raw & 0x0001,
        sync_vertex1: 0 != raw & 0x0002,
        sync_vertex2: 0 != raw & 0x0004,
        sync_vertex3: 0 != raw & 0x0100,
        interpolate: 0 != raw & 0x0008,
        scale: 0 != raw & 0x0010,
        show_curser: 0 != raw & 0x0020,
        sizeable: 0 != raw & 0x0040,
        screen_key: 0 != raw & 0x0080,
        studio_version_b1: 0 != raw & 0x0200,
        studio_version_b2: 0 != raw & 0x0400,
        studio_version_b3: 0 != raw & 0x0800,
        steam_enabled: 0 != raw & 0x1000,
        local_data_enabled: 0 != raw & 0x2000,
        borderless_window: 0 != raw & 0x4000,
        javascript_mode: 0 != raw & 0x8000,
    }
}

fn parse_function_classifications(
    raw_data: &Vec<u8>,
    offset: usize,
) -> UTGeneralInfoFunctionClassifications {
    let raw = chunk_read_u64(raw_data, offset);
    UTGeneralInfoFunctionClassifications {
        none: 0 != raw & 0x0,
        internet: 0 != raw & 0x1,
        joystick: 0 != raw & 0x2,
        gamepad: 0 != raw & 0x4,
        immersion: 0 != raw & 0x8,
        screengrab: 0 != raw & 0x10,
        math: 0 != raw & 0x20,
        action: 0 != raw & 0x40,
        matrix_d3d: 0 != raw & 0x80,
        d3dmodel: 0 != raw & 0x100,
        data_structures: 0 != raw & 0x200,
        file: 0 != raw & 0x400,
        ini: 0 != raw & 0x800,
        filename: 0 != raw & 0x1000,
        directory: 0 != raw & 0x2000,
        environment: 0 != raw & 0x4000,
        unused1: 0 != raw & 0x8000,
        http: 0 != raw & 0x10000,
        encoding: 0 != raw & 0x20000,
        uidialog: 0 != raw & 0x40000,
        motion_planning: 0 != raw & 0x80000,
        shape_collision: 0 != raw & 0x100000,
        instance: 0 != raw & 0x200000,
        room: 0 != raw & 0x400000,
        game: 0 != raw & 0x800000,
        display: 0 != raw & 0x1000000,
        device: 0 != raw & 0x2000000,
        window: 0 != raw & 0x4000000,
        draw_color: 0 != raw & 0x8000000,
        texture: 0 != raw & 0x10000000,
        layer: 0 != raw & 0x20000000,
        string: 0 != raw & 0x40000000,
        tiles: 0 != raw & 0x80000000,
        surface: 0 != raw & 0x100000000,
        skeleton: 0 != raw & 0x200000000,
        io: 0 != raw & 0x400000000,
        variables: 0 != raw & 0x800000000,
        array: 0 != raw & 0x1000000000,
        external_call: 0 != raw & 0x2000000000,
        notification: 0 != raw & 0x4000000000,
        date: 0 != raw & 0x8000000000,
        particle: 0 != raw & 0x10000000000,
        sprite: 0 != raw & 0x20000000000,
        clickable: 0 != raw & 0x40000000000,
        legacy_sound: 0 != raw & 0x80000000000,
        audio: 0 != raw & 0x100000000000,
        event: 0 != raw & 0x200000000000,
        unused2: 0 != raw & 0x400000000000,
        free_type: 0 != raw & 0x800000000000,
        analytics: 0 != raw & 0x1000000000000,
        unused3: 0 != raw & 0x2000000000000,
        unused4: 0 != raw & 0x4000000000000,
        achievement: 0 != raw & 0x8000000000000,
        cloud_saving: 0 != raw & 0x10000000000000,
        ads: 0 != raw & 0x20000000000000,
        os: 0 != raw & 0x40000000000000,
        iap: 0 != raw & 0x80000000000000,
        facebook: 0 != raw & 0x100000000000000,
        physics: 0 != raw & 0x200000000000000,
        flash_aa: 0 != raw & 0x400000000000000,
        console: 0 != raw & 0x800000000000000,
        buffer: 0 != raw & 0x1000000000000000,
        steam: 0 != raw & 0x2000000000000000,
        unused5: 0 != raw & 2310346608841064448,
        shaders: 0 != raw & 0x4000000000000000,
        vertex_buffers: 0 != raw & 9223372036854775808,
    }
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    // if (args.len() != 2) {
    //     println!("Usage: ./main <dataWinFile>");
    //     process::exit(1);
    // }

    // let data_file_path: &str = args[1].as_str();
    let data_file_path = "C:/Users/BioTomateDE/Documents/RustProjects/UndertaleModManager/data.win";
    println!("Loading data file {}", data_file_path);
    let data_file = read_data_file(data_file_path);
    let data = match parse_data_file(data_file) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    print_general_info(&data.general_info);

    // let changes: Vec<DataChange> = vec![
    //     DataChange {
    //         index: 2346,
    //         content: vec![69, 42, 0],
    //         delete: false,
    //     },
    //     DataChange {
    //         index: 2345,
    //         content: vec![32, 32, 0, 24, 124, 32, 95],
    //         delete: false,
    //     },
    //     DataChange {
    //         index: 421,
    //         content: vec![0, 0, 0, 0, 0, 0],
    //         delete: true,
    //     },
    // ];
    //
    // data.chunks["TXTR"].apply_changes(changes);
}
