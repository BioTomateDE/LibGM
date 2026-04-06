use std::path::PathBuf;

use clap::Parser;

use crate::actions::Action;
use crate::tests::Test;

/// A simple CLI for operating and debugging LibGM
#[derive(Parser, Debug)]
pub struct Args {
    /// The GameMaker data file(s) to load
    ///
    /// Default: `./data.win`
    pub files: Vec<PathBuf>,

    /// The path of the output data file to build.
    ///
    /// Leaving this empty will skip rebuilding.
    #[arg(short, long)]
    pub out: Option<PathBuf>,

    /// The tests to execute.
    #[arg(short, long, value_delimiter = ',')]
    pub tests: Vec<Test>,

    /// Actions to perform on the data file.
    #[arg(short, long, value_delimiter = ',')]
    pub actions: Vec<Action>,

    /// Code entries to disassemble.
    #[arg(short, long, value_delimiter = ',')]
    pub codes: Vec<String>,
}

#[must_use]
pub fn parse() -> Args {
    Args::parse()
}
