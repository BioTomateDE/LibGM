use std::path::PathBuf;

use clap::Parser;

use crate::{actions::Action, tests::Test};

#[derive(Parser, Debug)]
/// A simple CLI for operating and debugging LibGM
pub struct Args {
    /// The GameMaker data file(s) to load
    ///
    /// Default: `./data.win`
    pub files: Vec<PathBuf>,

    #[arg(short, long)]
    /// The path of the output data file to build.
    ///
    /// Leaving this empty will skip rebuilding.
    pub out: Option<PathBuf>,

    #[arg(short, long)]
    /// The tests to execute.
    pub tests: Vec<Test>,

    #[arg(short, long)]
    /// Actions to perform on the data file.
    pub actions: Vec<Action>,

    #[arg(short, long)]
    /// Code entries to disassemble.
    pub codes: Vec<String>,
}

#[must_use]
pub fn parse() -> Args {
    Args::parse()
}
