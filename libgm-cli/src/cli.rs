// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use clap::Parser;

use crate::actions::Action;
use crate::tests::Test;

/// A simple CLI for operating and debugging LibGM
#[derive(Parser, Debug)]
pub struct Args {
    /// The GameMaker data file(s) to load.
    ///
    /// Default: `./data.win`
    pub files: Vec<PathBuf>,

    /// The path of the output data file to build.
    ///
    /// Leaving this empty will skip rebuilding.
    #[arg(short, long)]
    pub out: Option<PathBuf>,

    /// If true, will serialize to the same file again.
    #[arg(long)]
    pub inplace: bool,

    /// Whether to allow invalid constants / alignment while parsing.
    ///
    /// Default: false
    #[arg(long)]
    pub lenient: bool,

    /// Display info about the data file.
    ///
    /// Default: false
    #[arg(short, long)]
    pub info: bool,

    /// The tests to execute.
    #[arg(short, long, value_delimiter = ',')]
    pub tests: Vec<Test>,

    /// Actions to perform on the data file.
    #[arg(short, long, value_delimiter = ',')]
    pub actions: Vec<Action>,

    /// Code entries to disassemble.
    #[arg(short, long, value_delimiter = ',')]
    pub codes: Vec<String>,

    /// Data files to compare to.
    #[arg(short, long, value_delimiter = ',')]
    pub diffs: Vec<PathBuf>,

    /// Whether to dump all sprites.
    #[arg(long)]
    pub dump_sprites: bool,

    /// Whether to dump all texture pages.
    #[arg(long)]
    pub dump_texture_pages: bool,
}

#[must_use]
pub fn parse() -> Args {
    Args::parse()
}
