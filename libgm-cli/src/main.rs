mod actions;
mod tests;

use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

use clap::Parser;
use libgm::{
    gamemaker::{data::GMData, deserialize::read_data_file, serialize::write_data_file},
    prelude::*,
};

use crate::{actions::Action, tests::Test};

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
        let path = entry
            .map_err(|e| e.to_string())
            .context("reading directory entry metadata")?
            .path();
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

fn run(mut args: Args) -> Result<()> {
    if args.files.is_empty() {
        args.files.push(PathBuf::from("data.win"));
    }

    // Very clunky test deduplication
    args.tests.dedup();
    if args.tests.contains(&Test::All) {
        args.tests = vec![Test::All];
    }

    let mut files = Vec::new();
    for path in args.files {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| e.to_string())
            .with_context(|| format!("reading metadata of {path:?}"))?;
        if metadata.is_dir() {
            let dir_files =
                listdir(&path).with_context(|| format!("reading entries of dir {path:?}"))?;
            files.extend(dir_files);
        } else {
            files.push(path);
        }
    }

    for data_file in files {
        log::info!("Parsing data file {data_file:?}");
        let mut data: GMData = read_data_file(data_file)?;

        tests::perform(&data, &args.tests)?;

        if let Some(out_file) = &args.out {
            for action in &args.actions {
                action.perform(&mut data)?;
            }

            log::info!("Building data file {out_file:?}");
            write_data_file(&data, out_file)?;
        }
    }

    Ok(())
}

fn diff() -> Result<()> {
    let a = Path::new("data.win");
    let a = read_data_file(a)?;
    let b = Path::new("game.unx");
    let b = read_data_file(b)?;

    dbg!(size_of::<GMData>());
    dbg!(a.animation_curves == b.animation_curves);
    dbg!(a.audio_groups == b.audio_groups);
    dbg!(a.audios == b.audios);
    dbg!(a.backgrounds == b.backgrounds);
    dbg!(a.codes == b.codes);
    dbg!(a.embedded_images == b.embedded_images);
    dbg!(a.extensions == b.extensions);
    dbg!(a.feature_flags == b.feature_flags);
    dbg!(a.filter_effects == b.filter_effects);
    dbg!(a.fonts == b.fonts);
    dbg!(a.functions == b.functions);
    dbg!(a.game_end_scripts == b.game_end_scripts);
    dbg!(a.game_objects == b.game_objects);
    dbg!(a.general_info == b.general_info);
    dbg!(a.global_init_scripts == b.global_init_scripts);
    dbg!(a.language_info == b.language_info);
    dbg!(a.options == b.options);
    dbg!(a.particle_emitters == b.particle_emitters);
    dbg!(a.particle_systems == b.particle_systems);
    dbg!(a.paths == b.paths);
    dbg!(a.rooms == b.rooms);
    dbg!(a.root_ui_nodes == b.root_ui_nodes);
    dbg!(a.scripts == b.scripts);
    dbg!(a.sequences == b.sequences);
    dbg!(a.shaders == b.shaders);
    dbg!(a.sounds == b.sounds);
    dbg!(a.sprites == b.sprites);
    dbg!(a.tags == b.tags);
    dbg!(a.texture_group_infos == b.texture_group_infos);
    dbg!(a.texture_page_items == b.texture_page_items);
    dbg!(a.timelines == b.timelines);
    dbg!(a.embedded_textures == b.embedded_textures);
    dbg!(a.variables == b.variables);
    Ok(())
}

fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "trace");
    }
    pretty_env_logger::init();

    let args = Args::parse();
    if let Err(error) = run(args) {
        log::error!("{}", error.chain_pretty());
        std::process::exit(1);
    }

    log::info!("Done");
}
