mod printing;
mod deserialize;
mod serialize;
mod export_mod;
mod debug_utils;
mod qoi;

use std::path::Path;
use std::process::exit;
use log::{info, error};

use crate::deserialize::all::{parse_data_file, read_data_file};
use crate::deserialize::all::GMData;
use crate::serialize::all::{build_data_file, write_data_file};


fn open_and_close() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let original_data_file_path: &Path = Path::new(args.get(1).map_or("data.win", |s| s));
    
    info!("Loading data file \"{}\"", original_data_file_path.display());
    let original_data: Vec<u8> = read_data_file(original_data_file_path)
        .map_err(|e| format!("Error while reading data file: {e}"))?;

    info!("Parsing data file");
    let data: GMData = parse_data_file(original_data)
        .map_err(|e| format!("Error while parsing data file: {e}"))?;

    info!("Building data file");
    let modded_data: Vec<u8> = build_data_file(&data)
        .map_err(|e| format!("Error while building data file: {e}"))?;

    let modified_data_file_path: &Path = Path::new("./data_out.win");
    info!("Writing data file \"{}\"", modified_data_file_path.display());
    write_data_file(modified_data_file_path, &modded_data)
        .map_err(|e| format!("Error while writing data file: {e}"))?;
    
    Ok(())
}


fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    
    if let Err(e) = open_and_close() {
        error!("{e}");
        exit(1);
    }

    info!("Done");
}

