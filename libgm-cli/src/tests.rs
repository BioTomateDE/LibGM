pub mod assembler;

use libgm::gamemaker::data::GMData;
use libgm::gamemaker::deserialize::read_data_bytes;
use libgm::gamemaker::serialize::build_data_file;
use libgm::prelude::*;

use crate::Test;

pub fn perform(data: &GMData, tests: &[Test]) -> Result<()> {
    println!();

    for test in tests {
        match test {
            Test::All => {
                // Perform all (other) tests.
                let all_tests = vec![Test::Reparse, Test::Assembler];
                perform(data, &all_tests)?;
            }
            Test::Builder => {
                log::info!("Performing Builder Test");
                build_data_file(data)?;
            }
            Test::Reparse => {
                log::info!("Performing Reparse Test");
                let raw: Vec<u8> = build_data_file(data)?;
                read_data_bytes(raw)?;
            }
            Test::Assembler => {
                log::info!("Performing Assembler Test");
                assembler::test_assembler(data)?;
            }
        }
    }
    Ok(())
}
