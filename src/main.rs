#![allow(unused)]

use libgm::gamemaker::deserialize::resources::GMRef;
use libgm::prelude::*;
use libgm::util::bench::Stopwatch;
use std::path::Path;

fn read_data_file(data_file_path: &Path) -> Result<Vec<u8>> {
    let stopwatch = Stopwatch::start();
    let data: Vec<u8> = std::fs::read(data_file_path)
        .with_context(|| format!("reading data file with path {:?}", data_file_path.display()))?;
    log::trace!("Reading data file took {stopwatch}");
    Ok(data)
}

fn write_data_file(data: Vec<u8>, data_file_path: &Path) -> Result<()> {
    let stopwatch = Stopwatch::start();
    std::fs::write(data_file_path, data)
        .with_context(|| format!("writing data file with path {:?}", data_file_path.display()))?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

fn path_from_arg<'a>(arg: Option<&'a String>, default: &'a str) -> &'a Path {
    Path::new(arg.map_or(default, |s| s))
}

fn main_open_and_close() -> Result<()> {
    use libgm::gamemaker::data::GMData;
    use libgm::gamemaker::deserialize::parse_data_file;
    use libgm::gamemaker::serialize::build_data_file;

    let args: Vec<String> = std::env::args().collect();
    let input_path: &Path = path_from_arg(args.get(1), "data.win");
    let output_path: &Path = path_from_arg(args.get(2), "data_out.win");

    // Read data file
    log::info!("Loading data file {:?}", input_path.display());
    let raw_data: Vec<u8> = read_data_file(input_path).context("reading data file")?;

    log::info!("Parsing data file");
    let gm_data: GMData = parse_data_file(raw_data).context("parsing data file")?;

    // Decompile a specific code
    let code_count = gm_data.codes.codes.len();
    for i in 0..code_count {
        let code_ref = GMRef::new(i as u32);
        log::debug!(
            "({i}/{code_count}) Decompiling {:?}",
            code_ref.resolve(&gm_data.codes.codes)?.name.display(&gm_data.strings)
        );
        libgm::gml::decompiler::decompile_to_ast(&gm_data, code_ref)?;
    }

    // // Build data file
    // log::info!("Building data file");
    // let raw_data: Vec<u8> = build_data_file(&gm_data).context("\nwhile building data file")?;
    // drop(gm_data);
    //
    // log::info!("Writing data file {:?}", output_path.display());
    // write_data_file(raw_data, output_path).context("writing data file")?;

    Ok(())
}

fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    log::debug!("============= LibGM v{} =============", env!("CARGO_PKG_VERSION"));

    if let Err(error) = main_open_and_close() {
        log::error!("{}", error.chain_with("↳"));
        std::process::exit(1);
    }

    log::info!("Done");
}
