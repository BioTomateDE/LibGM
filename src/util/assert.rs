use crate::prelude::*;
use std::fmt::{Display, UpperHex};

#[macro_export]
macro_rules! integrity_check {
    {$($tt:tt)*} => {
        #[cfg(not(feature = "no-integrity-checks"))] {
            $($tt)*
        }
    }
}

#[macro_export]
macro_rules! integrity_assert {
    {$condition:expr, $($arg:tt)*} => {
        #[cfg(not(feature = "no-integrity-checks"))] {
            if !$condition {
                return Err($crate::Error::new(format!($($arg)*)))
            }
        }
    }
}

pub fn assert_int<I: UpperHex + PartialEq + Display>(description: &'static str, expected: I, actual: I) -> Result<()> {
    integrity_assert! {
        expected == actual,
        "Expected {:?} to be {} but it is actually {} (0x{:0width$X})",
        description,
        expected,
        actual,
        actual,
        width = size_of::<I>() * 2,
    }
    Ok(())
}

pub fn assert_bool(description: &'static str, expected: bool, actual: bool) -> Result<()> {
    integrity_assert! {
        expected == actual,
        "Expected {:?} to be {} but it is actually {}",
        description,
        expected,
        actual,
    }
    Ok(())
}
