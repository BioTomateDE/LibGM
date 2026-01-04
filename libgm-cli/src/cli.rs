use std::path::PathBuf;

use clap::Parser;

use crate::{action::Action, tests::Test};

#[derive(Parser, Debug)]
/// A simple CLI for operating and debugging LibGM
pub struct Args {
    /// The GameMaker data file(s) to load (whitespace separated)
    ///
    /// Default: `./data.win`
    pub files: Vec<PathBuf>,

    #[arg(short, long)]
    /// The path of the output data file to build.
    ///
    /// Leaving this empty will skip rebuilding.
    pub out: Option<PathBuf>,

    #[arg(short, long, value_delimiter = ',')]
    /// The tests to execute (comma separated).
    pub tests: Vec<Test>,

    #[arg(short, long, value_delimiter = ',')]
    /// Actions to perform on the data file (comma separated).
    pub actions: Vec<Action>,
}

#[must_use]
pub fn parse() -> Args {
    Args::parse()
}
