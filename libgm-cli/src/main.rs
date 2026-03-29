#![allow(clippy::unnecessary_debug_formatting)]

mod actions;
mod cli;
mod dir;
mod logging;
mod tests;

use std::path::PathBuf;

use libgm::{
    gml::{Instruction, assembly::disassemble_code, insert_instructions, instruction::PushValue},
    prelude::*,
    wad::{data::GMData, deserialize::parse_file, serialize::build_file},
};

use crate::tests::Test;

fn run(mut args: cli::Args) -> Result<()> {
    // If no file was specified, try to load `data.win`.
    // This is very useful for standard IDEs which run the binary with no arguments.
    if args.files.is_empty() {
        args.files = vec![PathBuf::from("data.win")];
    }

    let tests: Vec<Test> = tests::deduplicate(args.tests);
    let files: Vec<PathBuf> = dir::get_data_files(&args.files)?;

    for data_file in files {
        log::info!("Parsing data file {}", data_file.display());
        let mut data: GMData = parse_file(data_file)?;

        tests::perform(&data, &tests)?;

        for action in &args.actions {
            action.perform(&mut data)?;
        }

        for code_name in &args.codes {
            let code = data.codes.by_name(code_name)?;
            let assembly = disassemble_code(code, &data)?;
            println!("===== {code_name} =====");
            println!("{assembly}");
            println!();
        }

        // // TODO REMOVE DEBUG
        // let gref = GMRef::new(0);
        // println!("{}\n\n", disassemble_code(&data.codes[0], &data)?);
        // let c = data.codes.by_ref_mut(gref)?;
        // insert_instructions(
        //     &mut c.instructions,
        //     3,
        //     &[
        //         Instruction::Push { value: PushValue::Double(f64::INFINITY) },
        //         Instruction::PopWithContextExit,
        //     ],
        // )?;
        // println!("{}\n\n", disassemble_code(&data.codes[0], &data)?);
        //
        if let Some(out_file) = &args.out {
            log::info!("Building data file {}", out_file.display());
            build_file(&data, out_file)?;
        }
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

// TODO: Overhaul the CLI: Allow for viewing of relevant data, exporting assembly and more
//       Maybe move the CLI to a different repo / publish it?
