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
use crate::export_mod::export::export_mod;
use crate::serialize::all::{build_data_file, write_data_file};


fn main_open_and_close() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let original_data_file_path: &Path = Path::new(args.get(1).map_or("data.win", |s| s));
    
    info!("Loading data file \"{}\"", original_data_file_path.display());
    let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
        .map_err(|e| format!("Error while reading data file: {e}"))?;

    info!("Parsing data file");
    let original_data: GMData = parse_data_file(original_data_raw)
        .map_err(|e| format!("Error while parsing data file: {e}"))?;

    info!("Building data file");
    let modified_data_raw: Vec<u8> = build_data_file(&original_data)
        .map_err(|e| format!("Error while building data file: {e}"))?;

    let modified_data_file_path: &Path = Path::new("./data_out.win");
    info!("Writing data file \"{}\"", modified_data_file_path.display());
    write_data_file(modified_data_file_path, &modified_data_raw)
        .map_err(|e| format!("Error while writing data file: {e}"))?;
    
    Ok(())
}


fn main_export_mod() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let original_data_file_path: &Path = Path::new(args.get(1).map_or("data_original.win", |s| s));
    let modified_data_file_path: &Path = Path::new(args.get(2).map_or("data_modified.win", |s| s));
    let mod_data_path: &Path = Path::new(args.get(3).map_or("mod.acornmod", |s| s));

    info!("Loading original data file \"{}\"", original_data_file_path.display());
    let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
        .map_err(|e| format!("Error while reading data file: {e}"))?;

    info!("Parsing original data file");
    let original_data: GMData = parse_data_file(original_data_raw)
        .map_err(|e| format!("Error while parsing original data file: {e}"))?;

    info!("Loading modified data file \"{}\"", modified_data_file_path.display());
    let modified_data_raw: Vec<u8> = read_data_file(modified_data_file_path)
        .map_err(|e| format!("Error while reading data file: {e}"))?;

    info!("Parsing modified data file");
    let modified_data: GMData = parse_data_file(modified_data_raw)
        .map_err(|e| format!("Error while parsing modified data file: {e}"))?;
    
    info!("Extracting changes and exporting mod to file \"{}\"", mod_data_path.display());
    export_mod(&original_data, &modified_data, mod_data_path)
        .map_err(|e| format!("Error while exporting mod: {e}"))?;

    Ok(())
}


fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    
    if let Err(e) = main_open_and_close() {
        error!("{e}");
        exit(1);
    }

    info!("Done");
}

