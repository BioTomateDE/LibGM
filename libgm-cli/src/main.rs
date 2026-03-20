#![allow(clippy::unnecessary_debug_formatting)]

mod actions;
mod cli;
mod dir;
mod logging;
mod tests;

use std::path::PathBuf;

use libgm::{
    gml::assembly::disassemble_code,
    prelude::*,
    wad::{
        data::GMData, deserialize::parse_file, elements::texture_page::Format,
        serialize::build_file,
    },
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

        // TODO REMOVE DEBUG
        fn serialize_textures(data: &mut GMData, format: Format) -> Result<()> {
            for texture_page in &mut data.texture_pages {
                let Some(image) = &mut texture_page.image else {
                    continue;
                };
                image.change_format(format)?;
            }
            Ok(())
        }
        fn dump_textures(data: &GMData, the: &str) -> Result<()> {
            for (i, texture_page) in data.texture_pages.iter().enumerate() {
                let Some(image) = &texture_page.image else {
                    continue;
                };
                let _ = std::fs::remove_dir("/tmp/gmtextures/");
                let path = format!("/tmp/gmtextures/{the}");
                std::fs::create_dir_all(&path).unwrap();
                image
                    .to_dynamic_image()?
                    .save(format!("{path}/{i}.png"))
                    .unwrap();
            }
            Ok(())
        }
        data.deserialize_textures()?;
        dump_textures(&data, "a")?;
        serialize_textures(&mut data, Format::Qoi)?;
        data.deserialize_textures()?;
        dump_textures(&data, "b")?;
        serialize_textures(&mut data, Format::Qoi)?;

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

// TODO: Overhaul the CLI: Allow for viewing of relevant data, exporting assembly and more
//       Maybe move the CLI to a different repo / publish it?
