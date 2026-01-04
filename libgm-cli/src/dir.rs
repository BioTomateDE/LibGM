use std::{
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

use libgm::prelude::*;

/// Fetch all contained GameMaker data files within a directory.
///
/// This operation is NOT recursive and will only search for files directly
/// within the directory (no sub-directories!).
///
/// Only files with a `.win`, `.unx`, `.ios` or `.droid` extension are considered
/// GameMaker data files. Other files will be skipped.
fn list_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut data_file_paths: Vec<PathBuf> = Vec::new();
    let dir: ReadDir = fs::read_dir(dir)
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

pub fn get_data_files(input_files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for path in input_files {
        let metadata = fs::metadata(path)
            .map_err(|e| e.to_string())
            .with_context(|| format!("reading metadata of {path:?}"))?;

        if metadata.is_dir() {
            let dir_files: Vec<PathBuf> =
                list_dir(path).with_context(|| format!("reading entries of directory {path:?}"))?;
            files.extend(dir_files);
        } else {
            files.push(path.clone());
        }
    }

    Ok(files)
}
