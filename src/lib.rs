#![deny(unused_must_use)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

// Core error handling
mod error;

// Internal utilities
pub mod test;
pub mod util;

// Main modules
pub mod gamemaker;
pub mod gml;

// Prelude for glob imports
pub mod prelude;
