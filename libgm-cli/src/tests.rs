mod assembler;

use clap::ValueEnum;
use libgm::prelude::*;
use libgm::wad::data::GMData;
use libgm::wad::deserialize::parse_bytes;
use libgm::wad::serialize::build_bytes;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Test {
    All,
    Builder,
    Reparse,
    Assembler,
    NameValidation,
}

const ALL_TESTS: &[Test] = &[Test::Reparse, Test::Assembler, Test::NameValidation];

#[must_use]
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
    if tests.is_empty() {
        return Ok(());
    }

    for test in tests {
        match test {
            Test::All => {
                unreachable!("Test::All is replaced by deduplication beforehand")
            }
            Test::Builder => {
                log::info!("Performing Builder Test");
                build_bytes(data)?;
            }
            Test::Reparse => {
                log::info!("Performing Reparse Test");
                let raw: Vec<u8> = build_bytes(data)?;
                let _new_data = parse_bytes(raw)?;
                // log_differences(data, &new_data);
            }
            Test::Assembler => {
                log::info!("Performing Assembler Test");
                assembler::test(data)?;
            }
            Test::NameValidation => {
                log::info!("Performing Name Validation Test");
                data.validate_names()?;
            }
        }
    }
    Ok(())
}

#[allow(unused)]
fn log_differences(old: &GMData, new: &GMData) {
    macro_rules! diffs {
        ($($field:ident)*) => {{ $(
            if old.$field != new.$field {
                log::warn!(concat!("Difference for ", stringify!($field), ":"));
                $crate::diff::print_diff(&old.$field, &new.$field);
                // let old_str = format!("{:#?}", old.$field);
                // let new_str = format!("{:#?}", new.$field);
                // let diff = ::similar::TextDiff::from_lines(&old_str, &new_str);
                // println!("{}", diff.unified_diff());
            }
        )* }};
    }

    diffs!(
         animation_curves
         audio_groups
         audios
         backgrounds
         codes
         embedded_images
         extensions
         feature_flags
         filter_effects
         fonts
         functions
         game_end_scripts
         game_objects
         general_info
         global_init_scripts
         language_info
         options
         particle_emitters
         particle_systems
         paths
         rooms
         ui_nodes
         scripts
         sequences
         shaders
         sounds
         sprites
         tags
         texture_group_infos
         texture_page_items
         texture_pages
         timelines
         variables
    );
}
