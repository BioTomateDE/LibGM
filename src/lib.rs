mod modding;
mod utility;
mod csharp_rng;

pub mod gamemaker;
pub mod gml;

pub use gamemaker::data::GMData;
pub use gamemaker::deserialize::parse_data_file;
pub use gamemaker::serialize::build_data_file;
use crate::utility::filename_to_str;

/// This function should only be used within the `tests` or `benches` directory in LibGM.
/// Do not use this if you are using LibGM as a dependency.
#[doc(hidden)]
pub fn __test_data_files(test_fn: impl Fn(GMData) -> Result<(), String>) -> Result<(), String> {
    unsafe { std::env::set_var("BIO_LOG", "debug"); }
    biologischer_log::init(env!("CARGO_CRATE_NAME"));

    let mut data_file_paths = Vec::new();
    for file in std::fs::read_dir("tests/data_files").unwrap() {
        let path = file.unwrap().path();
        let Some(ext) = path.extension() else {continue};
        let ext: &str = ext.to_str().unwrap();
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
        let raw_data: Vec<u8> = std::fs::read(&data_file_path).unwrap();
        log::info!("Parsing data file {name}");
        let gm_data: GMData = parse_data_file(&raw_data, false)?;
        drop(raw_data);
        log::info!("Testing data file {name}");
        test_fn(gm_data).map_err(|e| format!("{e}\n↳ while testing data file {}", filename_to_str(&data_file_path)))?;
    }

    log::info!("All data files passed.");
    Ok(())
}

