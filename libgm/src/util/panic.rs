use std::{any::Any, panic::UnwindSafe};

use crate::error::{Error, Result};

#[allow(clippy::option_if_let_else)]
fn extract_panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
    if let Some(string) = payload.downcast_ref::<&str>() {
        string.to_string()
    } else if let Ok(string) = payload.downcast::<String>() {
        *string
    } else {
        "Unknown panic value".to_string()
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
