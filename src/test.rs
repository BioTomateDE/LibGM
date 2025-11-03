use crate::error::Context;
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::parse_data_file;
use crate::prelude::*;
use crate::util::fmt::filename_to_str;
use std::fs::ReadDir;
use std::path::PathBuf;

/// Test all data files in `tests/data_files/` with the specified function.
/// See `tests/README.md` for more information.
pub fn test_data_files(test_fn: impl Fn(GMData) -> Result<()>) {
    // SAFETY: This program is single threaded.
    unsafe {
        std::env::set_var("BIO_LOG", "debug");
    }

    biologischer_log::init(env!("CARGO_CRATE_NAME"));

    if let Err(error) = run(test_fn) {
        println!();
        panic!("{}", error.chain());
    }
}

fn run(test_fn: impl Fn(GMData) -> Result<()>) -> Result<()> {
    let mut data_file_paths: Vec<PathBuf> = Vec::new();
    let dir: ReadDir = std::fs::read_dir("tests/data_files").context("reading data file folder")?;

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

    let data_filenames = data_file_paths
        .iter()
        .map(|p| filename_to_str(p))
        .collect::<Result<Vec<_>>>()?;

    log::debug!("Testing data files [{}]", data_filenames.join(", "));

    for data_file_path in data_file_paths {
        let filename = filename_to_str(&data_file_path)?;
        log::info!("Testing data file {filename}");

        let raw_data: Vec<u8> = std::fs::read(&data_file_path).context("reading data file")?;
        let gm_data: GMData = parse_data_file(raw_data).with_context(|| format!("parsing data file {filename}"))?;
        test_fn(gm_data).with_context(|| format!("testing data file {filename}"))?;
    }

    log::info!("All data files passed.");
    Ok(())
}
