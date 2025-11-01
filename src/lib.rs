#![deny(unused_must_use)]
#![deny(unreachable_patterns)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

// Core error handling
mod error;
pub use error::*;

// Internal utilities
pub mod test;
pub mod util;

// Main modules
pub mod gamemaker;
pub mod gml;

// Convenience re-exports
pub use gamemaker::data::GMData;
pub use gamemaker::deserialize::parse_data_file;
pub use gamemaker::serialize::build_data_file;

// Prelude for glob imports
pub mod prelude;
