//! todo: actual docstring

// These `must_use`s and unreachable patterns are usually
// critical and indicate a serious logical flaw.
//
// If you know that these are unused and are just
// using them for debugging purposes,
// you can temporarily comment this out.
#![deny(unused_must_use)]
#![deny(unreachable_patterns)]
//
//
// Lint exceptions (allowlist) are defined in `clippy.toml`.
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod error;
mod util;

pub mod gamemaker;
pub mod gml;
pub mod prelude;
