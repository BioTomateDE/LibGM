#![allow(non_snake_case)]

mod printing;
use printing::print_general_info;
mod general_info;
use general_info::parse_chunk_GEN8;
mod structs;
use structs::*;
mod chunk_reading;
use chunk_reading::*;


use std::collections::HashMap;
use std::{fs, process};
use crate::general_info::parse_chunk_OPTN;
use crate::printing::print_options;

fn read_data_file(data_file_path: &str) -> Result<Vec<u8>, String> {
    return match fs::read(data_file_path) {
        Ok(file) => Ok(file),
        Err(error) => {
            Err(format!("Could not read data file: {error:?}"))
        }
    };
}



fn parse_data_file(raw_data: Vec<u8>) -> Result<UTData, String> {
    let mut raw_all_chunk = UTChunk {
        name: String::from(""),
        data: raw_data.to_owned(),
        file_index: 0
    };

    if raw_all_chunk.read_chunk_name() != "FORM" {
        return Err(String::from(
            "Invalid or corrupted data.win file: 'FORM' chunk missing!",
        ));
    }

    // get chunks
    let raw_data_len: usize = raw_all_chunk.read_u32() as usize + raw_all_chunk.file_index;
    let mut chunks: HashMap<String, UTChunk> = HashMap::new();

    while raw_all_chunk.file_index + 8 < raw_data_len {
        let chunk_name: String = raw_all_chunk.read_chunk_name();
        let chunk_length: usize = raw_all_chunk.read_u32() as usize;
        let chunk_data: Vec<u8> = raw_all_chunk.data[raw_all_chunk.file_index .. raw_all_chunk.file_index + chunk_length].to_owned();
        chunks.insert(
            chunk_name.clone(),
            UTChunk {
                data: chunk_data,
                name: chunk_name,
                file_index: 0,
            },
        );
        raw_all_chunk.file_index += chunk_length;
    }

    let chunk_STRG: UTChunk = match chunks.get("STRG") {
        None => return Err(String::from("Invalid or corrupted data.win file: 'STRG' chunk missing!")),
        Some(chunk) => chunk.clone(),
    };
    let chunk_GEN8: UTChunk = match chunks.get("GEN8") {
        None => return Err(String::from("Invalid or corrupted data.win file: 'GEN8' chunk missing!")),
        Some(chunk) => chunk.clone(),
    };
    let chunk_OPTN: UTChunk = match chunks.get("OPTN") {
        None => return Err(String::from("Invalid or corrupted data.win file: 'OPTN' chunk missing!")),
        Some(chunk) => chunk.clone(),
    };

    let strings: HashMap<u32, String> = parse_chunk_STRG(chunk_STRG);
    let general_info: UTGeneralInfo = parse_chunk_GEN8(chunk_GEN8, &strings);
    let options: UTOptions = parse_chunk_OPTN(chunk_OPTN);

    let data = UTData {
        // chunks,
        strings,
        general_info,
        options,
    };

    // println!("Total data length: {total_length} bytes");
    // println!("Chunk Sizes:");
    // for (chunk_name, chunk) in &data.chunks {
    //     println!("  {}: {} bytes", chunk_name, chunk.data.len());
    // }

    // testong
    // for (chunk_name, chunk) in &chunks {
    //     let path = format!("./_expdat/{chunk_name}.bin");
    //     match fs::write(path, chunk.data.clone()) {
    //         Ok(_) => (),
    //         Err(err) => eprintln!("Failed to write to file for {chunk_name}: {}", err),
    //     }
    // }
    // ^

    Ok(data)
}

fn parse_chunk_STRG(mut chunk: UTChunk) -> HashMap<u32, String> {
    let string_count: usize = chunk.read_u32() as usize;
    let mut string_ids: Vec<u32> = Vec::with_capacity(string_count);
    let mut strings: HashMap<u32, String> = HashMap::with_capacity(string_count);

    for _ in 0..string_count {
        // you have to add 4 to the string id for some unknown reason
        let string_id = 4 + chunk.read_u32();
        string_ids.push(string_id);
    }

    for string_id in string_ids {
        let string_length: usize = chunk.read_u32() as usize;
        let string: String = chunk.read_literal_string(string_length);
        chunk.file_index += 1;  // skip one byte for the null byte after the string
        strings.insert(string_id, string);
    }
    strings
}



fn main() {
    // let args: Vec<String> = env::args().collect();
    // if (args.len() != 2) {
    //     println!("Usage: ./main <dataWinFile>");
    //     process::exit(1);
    // }

    // let data_file_path: &str = args[1].as_str();
    let data_file_path = "C:/Users/BioTomateDE/Documents/RustProjects/UndertaleModManager/dataExper.win";
    println!("Loading data file {}", data_file_path);

    let data_file = match read_data_file(data_file_path) {
        Ok(data_file) => data_file,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    let data = match parse_data_file(data_file) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    println!();
    print_general_info(&data.general_info);
    println!();
    print_options(&data.options);


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
