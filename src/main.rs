#![allow(non_snake_case)]
#![allow(dead_code)]

mod printing;

mod deserialize;
use deserialize::all::{parse_data_file, read_data_file};

mod serialize;

mod structs;   // TODO remove this

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
    let data_file_path: &str = "./data.win";
    println!("Loading data file {}", data_file_path);

    let data_file: Vec<u8> = match read_data_file(data_file_path) {
        Ok(data_file) => data_file,
        Err(error) => {
            eprintln!("Error while reading data file: {error}");
            process::exit(1);
        }
    };

    let data: UTData = match parse_data_file(data_file) {
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

    let raw_data2: Vec<u8> = match build_data_file(&data) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Error while building data file: {error}");
            process::exit(1);
        }
    };

    match write_data_file("./data_out.win", &raw_data2) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Error while writing data file: {error}");
            process::exit(1);
        }
    };

    println!("{}", data.strings.get_string_by_pos(12122776).unwrap());
}

