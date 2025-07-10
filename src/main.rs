#![deny(unused_must_use)]
#![deny(unreachable_patterns)]
#![deny(unused_assignments)]
#![deny(unused_imports)]
#![deny(unused_mut)]
#![deny(unused_macros)]
#![deny(clippy::all)]

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

mod gamemaker;
mod export_mod;
mod utility;
mod csharp_rng;

use std::path::Path;
use std::process::exit;
use log::{error, info};

use gamemaker::deserialize::GMData;
// use crate::export_mod::export::export_mod;
use gamemaker::deserialize::parse_data_file;
use gamemaker::serialize::build_data_file;
use crate::utility::Stopwatch;


fn read_data_file(data_file_path: &Path) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let data: Vec<u8> = std::fs::read(data_file_path)
        .map_err(|e| format!("Could not read data file with path \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Reading data file took {stopwatch}");
    Ok(data)
}

fn write_data_file(data: Vec<u8>, data_file_path: &Path) -> Result<(), String> {
    let stopwatch = Stopwatch::start();
    std::fs::write(data_file_path, data)
        .map_err(|e| format!("Could not write data file with path \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

fn path_from_arg<'a>(arg: Option<&'a String>, default: &'a str) -> &'a Path {
    Path::new(arg.map_or(default, |s| s))
}


fn main_open_and_close() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let original_data_file_path: &Path = path_from_arg(args.get(1), "data.win");
    let modified_data_file_path: &Path = path_from_arg(args.get(2), "data_out.win");

    info!("Loading data file \"{}\"", original_data_file_path.display());
    let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
        .map_err(|e| format!("{e}\n↳ while reading data file"))?;

    info!("Parsing data file");
    let original_data: GMData = parse_data_file(&original_data_raw, false)
        .map_err(|e| format!("\n{e}\n↳ while parsing data file"))?;

    info!("Building data file");
    let modified_data_raw: Vec<u8> = build_data_file(&original_data)
        .map_err(|e| format!("\n{e}\n↳ while building data file"))?;

    info!("Writing data file \"{}\"", modified_data_file_path.display());
    write_data_file(modified_data_raw, modified_data_file_path)
        .map_err(|e| format!("{e}\n↳ while writing data file"))?;

    Ok(())
}


// fn main_export_mod() -> Result<(), String> {
//     let args: Vec<String> = std::env::args().collect();
//     let original_data_file_path = path_from_arg(args.get(1), "data_original.win");
//     let modified_data_file_path = path_from_arg(args.get(2), "data_modified.win");
//     let mod_data_path = path_from_arg(args.get(3), "acornmod.tar.zst");
//
//     info!("Loading original data file \"{}\"", original_data_file_path.display());
//     let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
//         .map_err(|e| format!("{e}\n↳ while reading original data file"))?;
//
//     info!("Parsing original data file");
//     let original_data: GMData = parse_data_file(original_data_raw)
//         .map_err(|e| format!("{e}\n↳ while parsing original data file"))?;
//
//     info!("Loading modified data file \"{}\"", modified_data_file_path.display());
//     let modified_data_raw: Vec<u8> = read_data_file(modified_data_file_path)
//         .map_err(|e| format!("{e}\n↳ while reading modified data file"))?;
//
//     info!("Parsing modified data file");
//     let modified_data: GMData = parse_data_file(modified_data_raw)
//         .map_err(|e| format!("{e}\n↳ while parsing modified data file"))?;
//
//     info!("Extracting changes and exporting mod to file \"{}\"", mod_data_path.display());
//     export_mod(&original_data, &modified_data, mod_data_path)
//         .map_err(|e| format!("{e}\n↳ while exporting AcornGM mod"))?;
//
//     Ok(())
// }


fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    log::debug!("============= LibGM v{} =============", env!("CARGO_PKG_VERSION"));
    
    if let Err(e) = main_open_and_close() {
        error!("{e}");
        exit(1);
    }

    info!("Done");
}

