use std::{any::Any, panic::UnwindSafe};

use crate::error::{Error, Result};

fn extract_panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
    if let Some(&string) = payload.downcast_ref::<&str>() {
        string.to_owned()
    } else if let Ok(string) = payload.downcast::<String>() {
        *string
    } else {
        "Unknown panic value".to_owned()
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
