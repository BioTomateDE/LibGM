//! todo: actual docstring

#![deny(unused_must_use)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

mod error;
mod util;

pub mod gamemaker;
pub mod gml;

pub mod prelude;
