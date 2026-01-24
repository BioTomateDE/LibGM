use std::fmt::{Display, UpperHex};

use crate::prelude::*;

pub fn int<I: Copy + Eq + Display + UpperHex>(
    actual: I,
    expected: I,
    description: &'static str,
) -> Result<()> {
    if cfg!(not(feature = "integrity-checks")) {
        return Ok(());
    }

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

// More stuff available in `DataReader::assert_x`
