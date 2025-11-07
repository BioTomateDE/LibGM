use libgm::gamemaker::data::GMData;
use libgm::gamemaker::deserialize::parse_data_file;
use libgm::gamemaker::serialize::write_data_file;
use libgm::prelude::*;
use std::path::Path;

fn path_from_arg<'a>(arg: Option<&'a String>, default: &'static str) -> &'a Path {
    Path::new(arg.map_or(default, |s| s))
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let input_path: &Path = path_from_arg(args.get(1), "data.win");
    let output_path: &Path = path_from_arg(args.get(2), "data_out.win");

    // Read data file
    log::info!("Loading data file {input_path:?}");
    let gm_data: GMData = parse_data_file(input_path)?;

    // Build data file
    log::info!("Building data file {output_path:?}");
    write_data_file(&gm_data, output_path)?;

    Ok(())
}

fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    log::debug!("============= LibGM v{} =============", env!("CARGO_PKG_VERSION"));

    if let Err(error) = run() {
        log::error!("{}", error.chain_with("↳"));
        std::process::exit(1);
    }

    log::info!("Done");
}
