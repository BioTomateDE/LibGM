// SPDX-License-Identifier: GPL-3.0-only
#![allow(clippy::unnecessary_debug_formatting)]

mod actions;
mod cli;
mod dir;
mod logging;
mod tests;

use std::path::PathBuf;

use libgm::gml::assembly::disassemble_code;
use libgm::prelude::*;
use libgm::wad::build::build_file;
use libgm::wad::data::GMData;
use libgm::wad::parse::ParsingOptions;
use libgm_cli::diff::print_diffs;
use tests::Test;

fn run(mut args: cli::Args) -> Result<()> {
    // If no file was specified, try to load `data.win`.
    // This is very useful for standard IDEs which run the binary with no arguments.
    if args.files.is_empty() {
        args.files = vec![PathBuf::from("data.win")];
    }

    let tests: Vec<Test> = tests::deduplicate(args.tests);
    let files: Vec<PathBuf> = dir::get_data_files(&args.files)?;

    let parser = if args.lenient {
        ParsingOptions::LENIENT
    } else {
        ParsingOptions::STRICT
    };

    for data_file in files {
        log::info!("Parsing data file {}", data_file.display());
        if args.gen8_only {
            let raw_data = std::fs::read(data_file).context_src("reading data file")?;
            let gen8 = parser.parse_general_info(raw_data)?;
            println!("General Info: {gen8:#?}");
            continue;
        }

        let raw_data: Vec<u8> = std::fs::read(&data_file).context_src("reading data file")?;
        let mut data: GMData = parser.parse_bytes(&raw_data)?;

        tests::perform(&mut data, &tests, &raw_data)?;

        for data_file2 in &args.diffs {
            log::info!("Diffing with data file {}", data_file2.display());
            let data2: GMData = parser.parse_file(data_file2)?;
            print_diffs(&data, &data2);
        }

        for action in &args.actions {
            action.perform(&mut data)?;
        }

        for code_name in &args.codes {
            let code = data.codes.by_name(code_name, &data.strings)?;
            let assembly = disassemble_code(code, &data)?;
            println!("===== {code_name} =====");
            println!("{assembly}");
            println!();
        }

        // std::fs::create_dir_all("all").unwrap();
        // for c in &data.codes.codes {
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
            log::info!("Building data file {}", out_file.display());
            build_file(&data, out_file)?;
        } else if args.inplace {
            log::info!("Building data file to same location");
            build_file(&data, data_file)?;
        }

        println!();
    }

    Ok(())
}

fn main() {
    logging::init();
    let args = cli::parse();

    if let Err(error) = run(args) {
        let chain_fn = if cfg!(target_os = "windows") {
            // Windows usually can't display these arrows correctly
            Error::chain
        } else {
            Error::chain_pretty
        };
        log::error!("{}", chain_fn(&error));
        std::process::exit(1);
    }

    log::info!("Done");
}

// TODO: Overhaul the CLI: Allow for viewing of relevant data, exporting
// assembly and more       Maybe move the CLI to a different repo / publish it?
