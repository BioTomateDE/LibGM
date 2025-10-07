pub use crate::error::{Error, Result, Context};
pub use crate::{bail, err};


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

