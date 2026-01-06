//! **A data parsing and building library for GameMaker data files (`data.win`).**
//!
//! This library provides structs and functions to handle GameMaker game assets
//! in a meaningful way.
//!
//! For more information on the GameMaker specifics, check out the [`gamemaker`] module.
//!
//! ## Disclaimer
//! I, BioTomateDE, wrote this project myself. It is effectively my first Rust project ever.
//! While I have been working on this project for almost a year now (😭), I'm still
//! not the best Rust programmer.
//!
//! This library is still in testing stages
//! ([SemVer](https://semver.org/) major 0)
//! and may have issues.
//! Please report them to the attached GitHub repository.
//!
//! If you have any questions or concerns about my code
//! or documentation, please contact me via either:
//! - Discord DM: `@farming.simulator`
//! - [GitHub Issue](https://github.com/BioTomateDE/LibGM/issues/new)
//! - [Email](mailto:latuskati+cratesio@gmail.com?subject=%5BLibGM%5D%20Your%20code%20is%20fucking%20stupid%2C%20explain%20ts%20plz)
//!
//! ## Panicking
//! This library *should* **never panic**.
//! All malformed data files are caught into `LibGM`'s custom error type.
//! However, since this library is not mature yet, there might still be a few bugs.
//! For GUI applications, I would definitely recommend to **set a
//! [panic catcher](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)**
//! before calling any data parsing/building functions, just to be safe.
//!
//! ## Missing features
//! The following features are not yet supported by `LibGM`:
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
//!
//!

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
#![warn(clippy::correctness)]
#![warn(clippy::suspicious)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::style)]
#![warn(clippy::pedantic)]
//
// I sometimes need more than 3 bools in a struct???
// This lint is only relevant for people
// who have never heard of an enum.
#![allow(clippy::struct_excessive_bools)]
//
// Reading data can always fail (e.g. trying to read out of bounds).
// Almost all parser related functions have to read
// data at some point down the call hierarchy.
// Putting the same `Errors` header everywhere is meaningless.
#![allow(clippy::missing_errors_doc)]
//
// I store `Option<T>`, so passing `Option<&T>` instead of `&Option<T>`
// would require me to use `.as_ref()` every single time.
#![allow(clippy::ref_option)]
//
// Reinterpreting the bits when using `as`-casts
// is the intended behavior.
// When trying to "safely convert" between signed and
// unsigned integers, use `try_from` instead.
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
//
// "Cast from usize to u32 can truncate".
// This is irrelevant because all `usize`s are
// related to element count or data length.
// If either of those is out of `u32` bounds,
// the data reading/parsing will fail anyway
// since data files are only allowed to be
// smaller than 2 GB (`i32` limit).
#![allow(clippy::cast_possible_truncation)]
//
// YoYoGames may add a float field to some element in the future.
// This would break existing `Eq` structs.
#![allow(clippy::derive_partial_eq_without_eq)]
//
// This is a stylistic preference of mine.
// I may change this in the future.
#![allow(clippy::useless_let_if_seq)]
#![allow(clippy::needless_late_init)]
//
// I currently don't care about long functions.
// Usually, they are `deserialize` or `serialize` functions,
// which would be unclean to split up.
#![allow(clippy::too_many_lines)]

// Const assertion for soundness
const _: () = assert!(
    size_of::<usize>() >= size_of::<u32>(),
    "Cannot safely convert from u32 to usize on this platform. \
    Since GameMaker data files are 32-bit, this library will not function properly."
);

// Private modules
mod util;

// Public modules
pub mod actions;
pub mod error;
pub mod gamemaker;
pub mod gml;
pub mod prelude;

// Convenience re-exports
pub use error::{Error, Result};
pub use gamemaker::{
    deserialize::{parse_bytes, parse_file},
    serialize::{build_bytes, build_file},
};
