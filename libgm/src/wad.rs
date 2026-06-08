// SPDX-License-Identifier: GPL-3.0-only
//! Everything related to parsing and building of GameMaker data files.

mod memory;
mod reference;
mod version_detection;

pub mod build;
pub mod chunk;
pub mod data;
pub mod elem;
pub mod parse;
pub mod version;

pub use self::build::build_bytes;
pub use self::build::build_file;
pub use self::data::GMData;
pub use self::parse::parse_bytes;
pub use self::parse::parse_file;
pub use self::reference::GMRef;
pub use self::version::GMVersion;
