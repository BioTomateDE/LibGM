// SPDX-License-Identifier: GPL-3.0-only
#![allow(clippy::unnecessary_debug_formatting)]

mod actions;
mod cli;
mod diff;
mod dir;
mod dump;
mod info;
mod logging;
mod tests;

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use libgm::prelude::*;
use libgm::wad::data::GMData;
use libgm::wad::load::LoadingOptions;
use libgm::wad::save::save_file;

use self::cli::Args;
use self::diff::print_diffs;

fn handle_file(path: &Path, args: &Args) -> Result<Duration> {
    log::info!("Loading data file {}", path.display());
    let raw_data: Vec<u8> = std::fs::read(path).ctx_any("reading data file")?;
    let loader = if args.lenient { LoadingOptions::LENIENT } else { LoadingOptions::STRICT };
    let begin = Instant::now();
    let mut data: GMData = loader.load_bytes(&raw_data)?;
    let time = begin.elapsed();
    if args.bench {
        log::info!("Loading data took {time:.2?}");
    }

    if args.info {
        info::print_info(&data);
    }

    for action in &args.actions {
        action.perform(&mut data)?;
    }

    tests::perform(&mut data, &args.tests, &raw_data)?;

    for data_file2 in &args.diffs {
        log::info!("Diffing with data file {}", data_file2.display());
        let data2: GMData = loader.load_file(data_file2)?;
        print_diffs(&data, &data2);
    }

    for code_name in &args.codes {
        let code = data.codes.by_name(code_name, &data.strings)?;
        let assembly = libgm::gml::assembly::disassemble_code(code, &data)?;
        println!("===== {code_name} =====");
        println!("{assembly}");
        println!();
    }

    if args.dump_sprites {
        dump::dump_sprites(&data)?;
    }
    if args.dump_texture_pages {
        dump::dump_texture_pages(&data)?;
    }

    // dbg!(&data);

    // std::fs::create_dir_all("asm").unwrap();
    // for c in &data.codes.elems {
    //     let n = data.strings.by_ref(c.name)?;
    //     let asm = disassemble_code(c, &data)?;
    //     std::fs::write(format!("asm/{n}.txt"), asm).unwrap();
    // }

    // std::fs::create_dir_all("sounds").unwrap();
    // for snd in data.sounds.elements() {
    //     let name = data.strings.by_ref(snd.name)?;
    //     if snd.audio_file.is_none() {
    //         continue;
    //     }
    //     let audio = data.audios.by_ref(snd.audio_file)?;
    //     std::fs::write(format!("sounds/{name}.wav"), &audio.audio_data).unwrap();
    // }

    if let Some(out_file) = &args.out {
        log::info!("Encoding data file {}", out_file.display());
        save_file(&data, out_file)?;
    } else if args.inplace {
        log::info!("Encoding data file to same location");
        save_file(&data, path)?;
    }

    println!();
    Ok(time)
}

fn main() {
    logging::init();
    let mut args = cli::parse();
    args.tests = tests::deduplicate(args.tests);

    // If no file was specified, try to load `data.win`.
    // This is very useful for standard IDEs which run the binary with no arguments.
    if args.files.is_empty() {
        args.files = vec![PathBuf::from("data.win")];
    }

    let files: Vec<PathBuf> = match dir::get_data_files(&args.files) {
        Ok(f) => f,
        Err(e) => {
            log::error!("{}", e.chain());
            std::process::exit(1);
        }
    };

    let mut benches: Vec<Duration> = Vec::new();

    for data_file_path in files {
        match handle_file(&data_file_path, &args) {
            Ok(time) => benches.push(time),
            Err(error) => {
                log::error!("{}", error.chain());
                if args.fatal {
                    std::process::exit(2);
                }
            }
        }
    }

    if args.bench && benches.len() > 1 {
        let total_time: Duration = benches.iter().sum();
        let count = benches.len(); // only successfully parsed data files
        log::info!("Loading all {count} data files took {total_time:.4?}");
    } else {
        log::info!("Done");
    }
}

// TODO: Overhaul the CLI: Allow for viewing of relevant data, exporting
// assembly and more       Maybe move the CLI to a different repo / publish it?
