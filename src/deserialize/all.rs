use std::collections::HashMap;
use std::fs;
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::code::{parse_chunk_CODE, UTCode};
use crate::deserialize::fonts::{parse_chunk_FONT, UTFont};
use crate::deserialize::functions::{parse_chunk_FUNC, UTCodeLocal, UTFunction};
use crate::deserialize::general_info::{parse_chunk_GEN8, parse_chunk_OPTN};
use crate::deserialize::scripts::{parse_chunk_SCPT, UTScript};
use crate::deserialize::strings::{parse_chunk_STRG, UTStrings};
use crate::deserialize::variables::{parse_chunk_VARI, UTVariable};
use crate::deserialize::general_info::{UTGeneralInfo, UTOptions};


pub struct UTData {
    pub strings: UTStrings,                 // STRG
    pub general_info: UTGeneralInfo,        // GEN8
    pub options: UTOptions,                 // OPTN
    pub scripts: Vec<UTScript>,             // SCPT
    pub variables: Vec<UTVariable>,         // VARI
    pub functions: Vec<UTFunction>,         // FUNC
    pub code_locals: Vec<UTCodeLocal>,      // FUNC
    pub code: Vec<UTCode>,                  // CODE
    pub fonts: Vec<UTFont>,                 // FONT
}

pub fn parse_data_file(raw_data: Vec<u8>) -> Result<UTData, String> {
    let mut all = UTChunk {
        name: "".to_string(),
        abs_pos: 0,
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
        // println!("{} {}", chunk_name, all.file_index);
        chunks.insert(
            chunk_name.clone(),
            UTChunk {
                name: chunk_name,
                abs_pos: all.file_index,
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
    let chunk_CODE: UTChunk = get_chunk(&chunks, "CODE")?;
    let chunk_FONT: UTChunk = get_chunk(&chunks, "FONT")?;

    let strings: UTStrings = parse_chunk_STRG(chunk_STRG)?;
    // for (id,st) in &strings {
    //     if st == "Greetings." {
    //         println!("id: {id}");
    //     }
    // }
    let general_info: UTGeneralInfo = parse_chunk_GEN8(chunk_GEN8, &strings)?;
    let bytecode14: bool = general_info.bytecode_version >= 14;
    let options: UTOptions = parse_chunk_OPTN(chunk_OPTN)?;
    let scripts: Vec<UTScript> = parse_chunk_SCPT(chunk_SCPT, &strings)?;
    let variables: Vec<UTVariable> = parse_chunk_VARI(chunk_VARI, &strings)?;
    let (functions, code_locals): (Vec<UTFunction>, Vec<UTCodeLocal>) = parse_chunk_FUNC(chunk_FUNC, &strings, &chunk_CODE)?;
    let code: Vec<UTCode> = parse_chunk_CODE(chunk_CODE, bytecode14, &strings, &variables, &functions)?;
    let fonts: Vec<UTFont> = parse_chunk_FONT(chunk_FONT, &general_info, &strings)?;

    let data = UTData {
        strings,
        general_info,
        options,
        scripts,
        variables,
        functions,
        code_locals,
        code,
        fonts
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


pub fn read_data_file(data_file_path: &str) -> Result<Vec<u8>, String> {
    match fs::read(data_file_path) {
        Ok(file) => Ok(file),
        Err(error) => {
            Err(format!("Could not read data file: {error}"))
        }
    }
}


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

