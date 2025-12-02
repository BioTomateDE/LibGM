use std::fmt::{Display, UpperHex};

use crate::{gml::instructions::GMDataType, prelude::*};

pub fn assert_int<I: Copy + Eq + Display + UpperHex>(
    description: &'static str,
    expected: I,
    actual: I,
) -> Result<()> {
    if expected == actual {
        return Ok(());
    }

    let width = size_of::<I>() * 2;
    bail!(
        "Expected {} to be {} but it is actually {} (0x{:0width$X})",
        description,
        expected,
        actual,
        actual,
    );
}

pub fn assert_bool(description: &'static str, expected: bool, actual: bool) -> Result<()> {
    if expected == actual {
        return Ok(());
    }

    bail!("Expected {description} to be {expected} but it is actually {actual}");
}

pub fn assert_data_type(
    description: &'static str,
    expected: GMDataType,
    actual: GMDataType,
) -> Result<()> {
    if expected == actual {
        return Ok(());
    }

    bail!("Expected {description} Data Type to be {expected:?} but it is actually {actual:?}");
}
