// SPDX-License-Identifier: GPL-3.0-only
use std::any::Any;
use std::panic::UnwindSafe;

use crate::error::Error;
use crate::error::Result;

fn extract_panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
    if let Some(&string) = payload.downcast_ref::<&str>() {
        format!("Caught panic message (report this!): {string}")
    } else if let Ok(string) = payload.downcast::<String>() {
        format!("Caught panic message (report this!): {string}")
    } else {
        // can only happen if an upstream crate decides to be funny
        "LibGM panicked with a non-string value. Please report this error!".to_owned()
    }
}

pub fn catch<T, F>(func: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + UnwindSafe,
{
    match std::panic::catch_unwind(func) {
        Ok(ret) => ret,
        Err(err) => Err(Error::new(extract_panic_message(err))),
    }
}
