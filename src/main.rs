#![allow(non_snake_case)]
#![allow(dead_code)]

mod printing;
use printing::{print_general_info, print_options};

mod deserialize;
use deserialize::general_info::{parse_chunk_OPTN, parse_chunk_GEN8};
use deserialize::variables::{UTVariable, parse_chunk_VARI};
use deserialize::scripts::{UTScript, parse_chunk_SCPT};
use deserialize::functions::{UTFunction, UTCodeLocal, parse_chunk_FUNC};

mod structs;
use structs::*;

mod chunk_reading;
use chunk_reading::*;

use std::collections::HashMap;
use std::{fs, process};


fn get_chunk(chunks: &HashMap<String, UTChunk>, chunk_name: &str) -> Result<UTChunk, String> {
    match chunks.get(chunk_name) {
        None => Err(format!(
            "Chunk '{}' is missing in data file (chunk hashmap length: {}).",
            chunk_name,
            chunks.len()
        )),
        Some(chunk) => Ok(chunk.clone())
    }
}

fn read_data_file(data_file_path: &str) -> Result<Vec<u8>, String> {
    match fs::read(data_file_path) {
        Ok(file) => Ok(file),
        Err(error) => {
            Err(format!("Could not read data file: {error}"))
        }
    }
}

fn parse_data_file(raw_data: Vec<u8>) -> Result<UTData, String> {
    let mut all = UTChunk {
        name: "".to_string(),
        data: raw_data.clone(),
        data_len: raw_data.len(),
        file_index: 0
    };

    if all.read_chunk_name()? != "FORM" {
        return Err("Invalid or corrupted data.win file: 'FORM' chunk missing!".to_string());
    }

    // get chunks
    let raw_data_len: usize = all.read_usize()? + all.file_index;
    let mut chunks: HashMap<String, UTChunk> = HashMap::new();

    while all.file_index + 8 < raw_data_len {
        let chunk_name: String = all.read_chunk_name()?;
        let chunk_length: usize = all.read_usize()?;
        let chunk_data: Vec<u8> = all.data[all.file_index .. all.file_index + chunk_length].to_owned();
        chunks.insert(
            chunk_name.clone(),
            UTChunk {
                name: chunk_name,
                data: chunk_data.clone(),
                data_len: chunk_data.len(),
                file_index: 0,
            },
        );
        all.file_index += chunk_length;
    }

    let chunk_STRG: UTChunk = get_chunk(&chunks, "STRG")?;
    let chunk_GEN8: UTChunk = get_chunk(&chunks, "GEN8")?;
    let chunk_OPTN: UTChunk = get_chunk(&chunks, "OPTN")?;
    let chunk_SCPT: UTChunk = get_chunk(&chunks, "SCPT")?;
    let chunk_FUNC: UTChunk = get_chunk(&chunks, "FUNC")?;
    let chunk_VARI: UTChunk = get_chunk(&chunks, "VARI")?;

    let strings: HashMap<u32, String> = parse_chunk_STRG(chunk_STRG)?;
    let general_info: UTGeneralInfo = parse_chunk_GEN8(chunk_GEN8, &strings)?;
    let options: UTOptions = parse_chunk_OPTN(chunk_OPTN)?;
    let scripts: Vec<UTScript> = parse_chunk_SCPT(chunk_SCPT, &strings)?;
    let variables: Vec<UTVariable> = parse_chunk_VARI(chunk_VARI, &strings)?;
    let (functions, code_locals): (Vec<UTFunction>, Vec<UTCodeLocal>) = parse_chunk_FUNC(chunk_FUNC, &strings)?;

    let data = UTData {
        strings,
        general_info,
        options,
        scripts,
        variables,
        functions,
        code_locals
    };

    // println!("Total data length: {total_length} bytes");
    // println!("Chunk Sizes:");
    // for (chunk_name, chunk) in &data.chunks {
    //     println!("  {}: {} bytes", chunk_name, chunk.data.len());
    // }

    // ----- Testing -----
    // for (chunk_name, chunk) in &chunks {
    //     let path = format!("./_expdat/{chunk_name}.bin");
    //     match fs::write(path, chunk.data.clone()) {
    //         Ok(_) => (),
    //         Err(err) => eprintln!("Failed to write to file for {chunk_name}: {}", err),
    //     }
    // }
    // ----- ^^^^^^^^ -----

    Ok(data)
}

fn parse_chunk_STRG(mut chunk: UTChunk) -> Result<HashMap<u32, String>, String> {
    let string_count: usize = chunk.read_usize()?;
    let mut string_ids: Vec<u32> = Vec::with_capacity(string_count);
    let mut strings: HashMap<u32, String> = HashMap::new();

    for _ in 0..string_count {
        // you have to add 4 to the string id for some unknown reason
        let string_id = 4 + chunk.read_u32()?;
        string_ids.push(string_id);
    }

    for string_id in string_ids {
        let string_length: usize = chunk.read_usize()?;
        let string: String = chunk.read_literal_string(string_length)?;
        chunk.file_index += 1;  // skip one byte for the null byte after the string
        strings.insert(string_id, string);
    }
    Ok(strings)
}



fn main() {
    // let args: Vec<String> = env::args().collect();
    // if (args.len() != 2) {
    //     eprintln!("Usage: ./UndertaleModManager <dataWinFile>");
    //     process::exit(1);
    // }

    // let data_file_path: &str = args[1].as_str();
    let data_file_path: &str = "C:/Users/BioTomateDE/Documents/RustProjects/UndertaleModManager/dataExper.win";
    println!("Loading data file {}", data_file_path);

    let data_file: Vec<u8> = match read_data_file(data_file_path) {
        Ok(data_file) => data_file,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    let data: UTData = match parse_data_file(data_file) {
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
    // println!("{}", data.strings[&11246072]);
}

