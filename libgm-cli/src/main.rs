mod actions;
mod logging;
mod tests;

use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

use clap::Parser;
use libgm::{
    gamemaker::{data::GMData, deserialize::parse_file, serialize::build_file},
    prelude::*,
};

use crate::{actions::Action, tests::Test};

/// A simple CLI for operating and debugging LibGM
#[derive(Parser, Debug)]
struct Args {
    /// The GameMaker data file(s) to load (comma separated)
    /// | Default: ./data.win
    #[arg(value_delimiter = ',')]
    files: Vec<PathBuf>,

    /// The path of the output data file to build.
    /// Leaving this empty will skip rebuilding.
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// The tests to execute
    #[arg(short, long, value_delimiter = ',')]
    tests: Vec<Test>,

    /// Actions to perform on the data file (if outfile set)
    #[arg(short, long, value_delimiter = ',')]
    actions: Vec<Action>,
}

fn listdir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut data_file_paths: Vec<PathBuf> = Vec::new();
    let dir: ReadDir = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())
        .context("reading data file folder")?;

    for entry in dir {
        let entry = entry
            .map_err(|e| e.to_string())
            .context("reading directory entry metadata")?;

        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension() else {
            continue;
        };

        let ext = ext
            .to_str()
            .ok_or("Invalid File extension UTF-8 String {ext:?}")?;

        if matches!(ext, "win" | "unx" | "ios" | "droid") {
            data_file_paths.push(path);
        }
    }

    Ok(data_file_paths)
}

fn get_data_files(input_files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for path in input_files {
        let metadata = std::fs::metadata(path)
            .map_err(|e| e.to_string())
            .with_context(|| format!("reading metadata of {path:?}"))?;

        if metadata.is_dir() {
            let dir_files =
                listdir(path).with_context(|| format!("reading entries of dir {path:?}"))?;
            files.extend(dir_files);
        } else {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn run(mut args: Args) -> Result<()> {
    if args.files.is_empty() {
        args.files = vec![PathBuf::from("data.win")];
    }

    let tests: Vec<Test> = tests::deduplicate(args.tests);

    let files: Vec<PathBuf> = get_data_files(&args.files)?;

    for data_file in files {
        log::info!("Parsing data file {}", data_file.display());
        let mut data: GMData = parse_file(data_file)?;

        tests::perform(&data, &tests)?;

        for action in &args.actions {
            action.perform(&mut data)?;
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

    let args = Args::parse();
    if let Err(error) = run(args) {
        let chain_fn = if cfg!(target_os = "windows") {
            // Windows is ass and usually can't display these arrows correctly
            Error::chain
        } else {
            Error::chain_pretty
        };
        log::error!("{}", chain_fn(&error));
        std::process::exit(1);
    }

    log::info!("Done");
}
