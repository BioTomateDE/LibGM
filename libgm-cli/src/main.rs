mod actions;
mod cli;
mod dir;
mod logging;
mod tests;

use std::path::PathBuf;

use libgm::{
    gamemaker::{data::GMData, deserialize::parse_file, serialize::build_file},
    gml::assembly::disassemble_code,
    prelude::*,
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
