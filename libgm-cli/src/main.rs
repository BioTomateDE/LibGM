mod tests;

use crate::tests::assembler::test_assembler;
use crate::tests::decompiler::test_decompiler;
use clap::{Parser, ValueEnum};
use libgm::gamemaker::data::GMData;
use libgm::gamemaker::deserialize::parse_data_file;
use libgm::gamemaker::serialize::{build_data_file, write_data_file};
use libgm::prelude::*;
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

/// A simple CLI for operating and debugging LibGM
#[derive(Parser, Debug)]
struct Args {
    /// The `GameMaker` data file(s) to load (comma separated)
    /// Default: ./data.win
    #[arg(short, long, value_delimiter = ',')]
    files: Vec<PathBuf>,

    /// The path of the output data file to build.
    /// Leaving this empty will skip rebuilding.
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// The tests to execute.
    #[arg(short, long, value_delimiter = ',')]
    tests: Vec<Test>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Test {
    Builder,
    Assembler,
    Decompiler,
}

fn listdir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut data_file_paths: Vec<PathBuf> = Vec::new();
    let dir: ReadDir = std::fs::read_dir(dir).context("reading data file folder")?;

    for entry in dir {
        let path = entry.context("reading directory entry metadata")?.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension() else {
            continue;
        };
        let ext = ext.to_str().context("converting file extension to UTF-8")?;
        if matches!(ext, "win" | "unx" | "ios" | "droid") {
            data_file_paths.push(path);
        }
    }

    Ok(data_file_paths)
}

fn run(mut args: Args) -> Result<()> {
    if args.files.is_empty() {
        args.files.push(PathBuf::from("data.win"));
    }

    let mut files = Vec::new();
    for path in args.files {
        let metadata = std::fs::metadata(&path).with_context(|| format!("reading metadata of {path:?}"))?;
        if metadata.is_dir() {
            let dir_files = listdir(&path).with_context(|| format!("reading entries of dir {path:?}"))?;
            files.extend(dir_files);
        } else {
            files.push(path);
        }
    }

    for data_file in files {
        log::info!("Parsing data file {data_file:?}");
        let mut data: GMData = parse_data_file(data_file)?;

        if let Some(out_file) = &args.out {
            log::info!("Building data file {out_file:?}");
            write_data_file(&data, out_file)?;
        }

        for test in &args.tests {
            match test {
                Test::Builder => {
                    build_data_file(&data)?;
                }
                Test::Assembler => {
                    test_assembler(&mut data)?;
                }
                Test::Decompiler => {
                    test_decompiler(&data)?;
                }
            }
        }
    }

    Ok(())
}

fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "trace");
    }
    pretty_env_logger::init();

    let args = Args::parse();
    if let Err(error) = run(args) {
        log::error!("{}", error.chain_with("↳"));
        std::process::exit(1);
    }

    log::info!("Done");
}
