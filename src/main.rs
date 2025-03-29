// #![allow(non_snake_case)]
#![allow(dead_code)]

mod printing;

mod deserialize;
use deserialize::all::{parse_data_file, read_data_file};

mod serialize;

mod structs;   // TODO remove this file

use std::process;
use crate::deserialize::all::UTData;
use crate::serialize::all::{build_data_file, write_data_file};

fn main() {
    // let args: Vec<String> = env::args().collect();
    // if (args.len() != 2) {
    //     eprintln!("Usage: ./UndertaleModManager <dataWinFile>");
    //     process::exit(1);
    // }

    // let data_file_path: &str = args[1].as_str();
    let original_data_file_path: &'static str = "./data.win";

    println!("Loading data file \"{}\".", original_data_file_path);
    let original_data: Vec<u8> = match read_data_file(original_data_file_path) {
        Ok(data_file) => data_file,
        Err(error) => {
            eprintln!("Error while reading data file: {error}");
            process::exit(1);
        }
    };

    println!("Parsing data file.");
    let data: UTData = match parse_data_file(original_data) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Error while parsing data file: {error}");
            process::exit(1);
        }
    };

    // println!();
    // print_general_info(&data.general_info);
    // println!();
    // print_options(&data.options);

    println!("Building data file.");
    let modded_data: Vec<u8> = match build_data_file(&data) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Error while building data file: {error}");
            process::exit(1);
        }
    };

    let modded_data_file_path: &'static str = "./data_out.win";
    println!("Writing data file \"{}\".", modded_data_file_path);
    match write_data_file(modded_data_file_path, &modded_data) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Error while writing data file: {error}");
            process::exit(1);
        }
    };

    println!("Done.");

    // println!("{}", data.strings.get_string_by_pos(12122776).unwrap());
}

// TODO:    - Apparently C# implicitly converts numbers to int/long before applying bitwise operations on them.
//          This means that i have to change the thingies in ./deserialize/code.rs because they were actually NOT
//          redundant in the original C# UndertaleModTool code
