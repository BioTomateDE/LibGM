mod printing;
mod deserialize;
mod serialize;
mod export_mod;
mod debug_utils;

use std::path::Path;
use std::process::exit;
use log::{info, error};

use crate::deserialize::all::{parse_data_file, read_data_file};
use crate::deserialize::all::GMData;
use crate::serialize::all::{build_data_file, write_data_file};


fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));

    let args: Vec<String> = std::env::args().collect();
    let original_data_file_path: &Path = Path::new(args.get(1).map_or("data.win", |s| s));

    info!("Loading data file \"{}\"", original_data_file_path.display());
    let original_data: Vec<u8> = match read_data_file(original_data_file_path) {
        Ok(data_file) => data_file,
        Err(error) => {
            error!("Error while reading data file: {error}");
            exit(1);
        }
    };

    info!("Parsing data file");
    let data: GMData = match parse_data_file(original_data) {
        Ok(data) => data,
        Err(error) => {
            error!("Error while parsing data file: {error}");
            exit(1);
        }
    };

    info!("Building data file");
    let modded_data: Vec<u8> = match build_data_file(&data) {
        Ok(data) => data,
        Err(error) => {
            error!("Error while building data file: {error}");
            exit(1);
        }
    };

    let modded_data_file_path: &Path = Path::new("./data_out.win");
    info!("Writing data file \"{}\"", modded_data_file_path.display());
    match write_data_file(modded_data_file_path, &modded_data) {
        Ok(data) => data,
        Err(error) => {
            error!("Error while writing data file: {error}");
            exit(1);
        }
    };

    info!("Done");
}

