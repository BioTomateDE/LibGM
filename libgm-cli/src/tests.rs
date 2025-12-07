mod assembler;

use clap::ValueEnum;
use libgm::{
    gamemaker::{data::GMData, deserialize::read_data_bytes, serialize::build_data_file},
    prelude::*,
};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Test {
    All,
    Builder,
    Reparse,
    Assembler,
    NameValidation,
}

const ALL_TESTS: &[Test] = &[Test::Reparse, Test::Assembler, Test::NameValidation];

pub fn deduplicate(mut tests: Vec<Test>) -> Vec<Test> {
    if tests.contains(&Test::All) {
        return ALL_TESTS.to_vec();
    }

    tests.dedup();

    if tests.contains(&Test::Reparse) {
        let builder_index = tests.iter().position(|&t| t == Test::Builder);
        if let Some(index) = builder_index {
            tests.remove(index);
        }
    }

    tests
}

pub fn perform(data: &GMData, tests: &[Test]) -> Result<()> {
    println!();

    for test in tests {
        match test {
            Test::All => {
                // Perform all (other) tests.
                let all_tests = vec![Test::NameValidation, Test::Reparse, Test::Assembler];
                perform(data, &all_tests)?;
            },
            Test::Builder => {
                log::info!("Performing Builder Test");
                build_data_file(data)?;
            },
            Test::Reparse => {
                log::info!("Performing Reparse Test");
                let raw: Vec<u8> = build_data_file(data)?;
                read_data_bytes(raw)?;
            },
            Test::Assembler => {
                log::info!("Performing Assembler Test");
                assembler::test(data)?;
            },
            Test::NameValidation => {
                log::info!("Performing Name Validation Test");
                data.validate_names()?;
            },
        }
    }
    Ok(())
}
