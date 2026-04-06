//! A data parsing and building library for GameMaker data files (`data.win`).
//!
//! This library provides structs and functions to handle GameMaker game assets
//! in a meaningful way.
//!
//! ## Usage
//! For most purposes, using the [`parse_file`] and [`build_file`] functions is enough.
//!
//! ```no_run
//! use libgm::wad::GMData;
//! use libgm::wad::elements::game_object::GMGameObject;
//! use libgm::wad::elements::GMNamedListChunk;
//!
//! # fn main() -> libgm::Result<()> {
//!
//! // Load data file
//! let mut data: GMData = libgm::wad::parse_file("./data.win")?;
//! println!("Loaded {}", data.general_info.display_name);
//!
//! // Modify data file
//! let obj: &mut GMGameObject = data.game_objects.by_name_mut("obj_time")?;
//! obj.visible = true;
//!
//! // Write data file
//! libgm::wad::build_file(&data, "./modified_data.win")?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! If you need more control, you can use [`parse_bytes`], [`build_bytes`] or [`ParsingOptions`].
//!
//! For more information on the GameMaker specifics, check out the [`wad`] module.
//!
//! ## Disclaimer
//! This library is still in testing stages
//! ([SemVer](https://semver.org/) major 0)
//! and may have issues.
//! Please report them to the attached GitHub repository.
//!
//! If you have any questions or concerns about my code
//! or documentation, please contact me via either:
//! - [GitHub Issue](https://github.com/BioTomateDE/LibGM/issues/new)
//! - [Email](mailto:latuskati+cratesio@gmail.com?subject=%5BLibGM%5D%20Your%20code%20is%20fucking%20stupid%2C%20explain%20ts%20plz)
//! - Discord DM: `@biotomate.de`
//!
//! ## Panicking
//! This library *should* **never panic**.
//! All malformed data files are caught into LibGM's custom error type.
//! However, since this library is not mature yet, there might still be a few bugs.
//! For GUI applications, I would definitely recommend to enable the `catch-panic`
//! crate feature (which is enabled by default anyway).
//!
//! ## Missing features
//! The following features are not yet supported by LibGM:
//! - **Null pointers**.
//!   These typically occur in newer games compiled with `GMAC` (GameMaker Asset Compiler),
//!   which may null out pointers to unused elements.
//!   See [Issue#2](https://github.com/BioTomateDE/LibGM/issues/2) for more information.
//! - Special Vector Sprites
//! - Only partial pre WAD version 15 support
//! - Only partial/untested big endian support
//!
//! ## Breaking changes
//! Some things in this library are **not** considered "breaking changes" and may be
//! modified in `SemVer` patch updates. These could bring unwanted change of behavior
//! to your program if you don't have a `Cargo.lock` set/commited.
//! Some of these things include:
//! - All log messages (using the [`log`](https://crates.io/crates/log) crate), including:
//!   - Timing
//!   - Code Origin/Location
//!   - Message string
//! - All error contents:
//!   - Error message string
//!   - Context chain
//! - All structs and enums marked with `#[non_exhaustive]`
//! - Implementing traits like `GMElement` (These traits are only meant for writing generic code,
//!   not for implementing it for your own types)
//!
//! There might be some other struct fields or type names
//! with docstrings saying "this may change in the future".
//! These changes will still require a SemVer minor increment, though.
//! In other words, they are definitely version-safe to use,
//! but they might be renamed or reworked soon.
//!
//! ## Credits
//! Please note that this project is effectively a Rust port of
//! [UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool)
//! (UndertaleModLib, to be exact).
//! Most of the GameMaker elements' docstrings and struct field (names) are taken from there.

// Activate all lint groups.
// Pedantic is really strong, so many lints will be whitelisted (with a reason).
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic)]
//
#![deny(
    // These `must_use`s and unreachable patterns are usually
    // critical and indicate a serious logical flaw.
    //
    // If you know that these are unused and are just using them
    // for debugging purposes, you can temporarily comment this out.
    unused_must_use,
    unreachable_patterns,

    // This usually happens when renaming crate features.
    // This should be fixed immediately.
    unexpected_cfgs,
)]
//
#![allow(
    // Reading data can always fail (e.g. trying to read out of bounds).
    // Almost all parser related functions have to read
    // data at some point down the call hierarchy.
    // Putting the same `# Errors` header everywhere is meaningless.
    clippy::missing_errors_doc,

    // I sometimes need more than 3 bools in a struct???
    // This lint is only relevant for people
    // who have never heard of an enum.
    clippy::struct_excessive_bools,

    // Reinterpreting the bits when using `as`-casts is the intended behavior.
    // When trying to "safely convert" between signed and
    // unsigned integers, use `try_from` instead.
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,

    // "Cast from usize to u32 can truncate".
    // This is irrelevant because all `usize`s are related to element count or data length.
    // If either of those is out of `u32` bounds, the data reading/parsing will fail anyway
    // since data files are only allowed to be smaller than 2 GB (`i32` limit).
    clippy::cast_possible_truncation,

    // Unwraps are only used in a controlled manner where no panics can ever actually occur.
    clippy::missing_panics_doc,

    // YoYoGames may add a float field to some element in the future.
    // This would break existing `Eq` structs.
    clippy::derive_partial_eq_without_eq,

    // Applying this lint often makes the code less readable.
    clippy::useless_let_if_seq,

    // This is a style choice.
    clippy::match_same_arms,
)]

// Const assertion for soundness
const _: () = assert!(
    size_of::<usize>() >= size_of::<u32>(),
    "Cannot safely convert from u32 to usize on this platform. \
    Since GameMaker data files are 32-bit, this library will not function properly."
);

#[cfg(doc)]
use wad::{
    deserialize::{ParsingOptions, parse_bytes, parse_file},
    serialize::{build_bytes, build_file},
};

// Private modules
mod actions;
mod util;

// Public modules
pub mod error;
pub mod gml;
pub mod prelude;
pub mod wad;

// Convenience re-exports
pub use error::{Error, Result};

// === Some TODOs for the entire library ===
//
// When Rust finally drops Macros 2.0:
// * Migrate all `macro_rules!` to `macro`s and remove exporting from crate root.
// Reference: https://github.com/rust-lang/rust/issues/39412
//
// When most traits (`Into`, `TryInto`, `Iterator`, `PartialEq`) are const-stable:
// * Clean up all `const-hack` TODOs
