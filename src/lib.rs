// Core error handling
mod error;
pub use error::*;

// Internal utilities
mod csharp_rng;
pub mod utility;

// Main modules
pub mod gamemaker;
pub mod gml;
mod modding;

// Convenience re-exports
pub use gamemaker::data::GMData;
pub use gamemaker::deserialize::parse_data_file;
pub use gamemaker::serialize::build_data_file;

// Prelude for glob imports
pub mod prelude;


/// This function should only be used within the `tests` or `benches` directory in LibGM.
/// Do not use this if you are using LibGM as a dependency.
#[doc(hidden)]
pub fn __test_data_files(test_fn: impl Fn(GMData) -> Result<()>) -> Result<()> {
    use crate::utility::filename_to_str;
    unsafe { std::env::set_var("BIO_LOG", "debug"); }
    biologischer_log::init(env!("CARGO_CRATE_NAME"));

    let mut data_file_paths = Vec::new();
    for file in std::fs::read_dir("tests/data_files").context("reading data file folder")? {
        let path = file.context("reading file metadata")?.path();
        let Some(ext) = path.extension() else {continue};
        let ext: &str = ext.to_str().context("converting file extension to UTF-8")?;
        if matches!(ext, "win" | "unx" | "ios" | "droid") {
            data_file_paths.push(path);
        }
    }

    log::info!("Testing data files [{}]",
        data_file_paths.iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
    );

    for data_file_path in data_file_paths {
        let name = format!("\"{}\"", data_file_path.display());
        log::info!("Reading data file {name}");
        let raw_data: Vec<u8> = std::fs::read(&data_file_path).context("reading data file")?;
        log::info!("Parsing data file {name}");
        let gm_data: GMData = parse_data_file(&raw_data)?;
        drop(raw_data);
        log::info!("Testing data file {name}");
        test_fn(gm_data).with_context(|| format!("testing data file {}", filename_to_str(&data_file_path)))?;
    }

    log::info!("All data files passed.");
    Ok(())
}

