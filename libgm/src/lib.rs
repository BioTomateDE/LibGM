// SPDX-License-Identifier: GPL-3.0-only
//! A data parsing and building library for GameMaker data files (`data.win` / `game.unx`).
//!
//! This library provides structs and functions to
//! handle GameMaker game assets in a meaningful way.
//!
//! It provides a powerful API to the data file, which yields you full control of
//! GameMaker's internals while still providing you with abstractions to handle game assets conveniently.
//!
//! LibGM does not free you from all redundancies found in the data file format.
//! It does not aim to restore a data file into a classic GameMaker project structure.
//! This tradeoff makes parsing and building faster, allows you to view and edit slightly malformed
//! data files, and stays as close to the original data file layout when serializing.
//! Perhaps there can be a high-level crate in the future that abstracts on this library.
//!
//! ## Usage
//! For most purposes, using the [`parse_file`] and [`build_file`] functions is enough.
//! If you need more control, you can use [`parse_bytes`], [`build_bytes`] or [`ParsingOptions`].
//!
//! It is recommended to import the prelude so important types and traits are available to use:
//! ```
//! use libgm::prelude::*;
//! ```
//!
//! This is an example of how to open a data file stored on disk
//! and extract some basic information:
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use libgm::prelude::*;
//!
//! let path = "C:/Program Files (x86)/Steam/steamapps/common/Undertale/data.win";
//! let data: GMData = libgm::wad::parse_file(path)?;
//!
//! let name: &str = data.strings.by_ref(data.general_info.display_name)?;
//! let creation: chrono::DateTime<chrono::Utc> = data.general_info.creation_timestamp;
//! println!("Opened {name}!");
//! println!("Game was created at {creation:?}.");
//! println!("This data file has {} sprites.", data.sprites.len());
//! # Ok(()) }
//! ```
//!
//! Modifiying parts of the game:
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use libgm::prelude::*;
//! use libgm::wad::elem::game_object::GMGameObject;
//! use libgm::wad::elem::sprite::GMSprite;
//!
//! let mut data: GMData = libgm::wad::parse_file("./game.unx")?;
//! let object: &mut GMGameObject = data
//!     .game_objects
//!     .by_name_mut("obj_mysteryman", &data.strings)?;
//! object.solid = true;
//! object.depth = 66666;
//!
//! let sprite: &GMSprite = data.sprites.by_ref(object.sprite)?;
//! let w: u32 = sprite.width;
//! let h: u32 = sprite.width;
//! println!("Mysteryman's sprite dimensions are {w}x{h} pixels");
//!
//! libgm::wad::build_file(&data, "./game_modded.unx")?;
//! # Ok(()) }
//! ```
//!
//! More complicated parsing for data files stored in memory:
//! ```no_run
//! # fn some_sophisticated_source() -> &'static [u8] { &[] }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use libgm::prelude::*;
//! use libgm::wad::elem::audio::GMAudio;
//! use libgm::wad::elem::sound::Flags;
//! use libgm::wad::parse::ParsingOptions;
//! let raw_data: &[u8] = some_sophisticated_source();
//! let parser = ParsingOptions::new()
//!     .verify_constants(false)
//!     .verify_alignment(true);
//! let gm_data: GMData = parser.parse_bytes(raw_data)?;
//! std::fs::create_dir_all("exported_sounds")?;
//!
//! for sound in gm_data.sounds.elements() {
//!     if sound.flags.contains(Flags::EMBEDDED) {
//!         let name = gm_data.strings.by_ref(sound.name)?;
//!         let audio: &GMAudio = gm_data.audios.by_ref(sound.audio)?;
//!         let path = format!("exported_sounds/{name}.wav");
//!         std::fs::write(path, &audio.data)?;
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! For more information on the GameMaker specifics, check out the [`wad`] module.
//!
//! ## Disclaimer
//! This library is mainly tested against different Undertale and Deltarune versions.
//! Other games may encounter some issues.
//! Please report them to the attached Codeberg repository.
//!
//! LibGM is designed to roundtrip: Parsing a data file and then building it
//! without any modifications should produce the same exact output file.
//! If this assertion fails for some game, you can report this issue as well.
//!
//! If you have any questions or concerns about my code
//! or documentation, please contact me via either:
//! - Discord DM: `@biotomate.de`
//! - [Codeberg Issue](https://codeberg.org/BioTomateDE/LibGM/issues/new)
//! - [Email](mailto:biotomatede@proton.me?Subject=LibGM%20Question)
//!
//! ## Panicking
//! This library *should* never panic.
//! All malformed data files are caught into LibGM's custom error type.
//! However, since this library is not entirely mature yet, there might still be a few
//! bugs. For GUI applications, I would definitely recommend to enable the
//! `catch-panic` crate feature (which is enabled by default anyway).
//!
//! ## Missing features
//! The following features are not yet supported by LibGM:
//! - Special Vector Sprites
//! - Only partial pre WAD version 15 support (pre 2016)
//! - Only partial/untested big endian support
//!
//! ## Breaking changes
//! Some things in this library are **not** considered "breaking changes" and
//! may be modified in SemVer patch updates. These could bring unwanted change
//! of behavior to your program if you don't have a `Cargo.lock` set/commited.
//! Some of these things include:
//! - All log messages (using the [`log`](https://crates.io/crates/log) crate), including:
//!   - Timing
//!   - Code Origin/Location
//!   - Message string
//! - All error contents:
//!   - Error message string
//!   - Context chain
//! - All structs and enums marked with `#[non_exhaustive]`
//! - Implementing traits like `GMChunk` (These traits are only meant for
//!   writing generic code, not for implementing it for your own types)
//!
//! There might be some other struct fields or type names
//! with docstrings saying "this may change in the future".
//! These changes will still require a SemVer minor increment, though.
//! In other words, they are definitely version-safe to use,
//! but they might be renamed or reworked soon.
//!
//! ## Credits
//! This project is effectively a Rust port of
//! [UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool).
//! Version detection code, some element docstrings and other parts are taken from there.
//! Huge shoutout to the Underminers Team!

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

    // Only used in small local scope.
    clippy::enum_glob_use,

    // These are style choices.
    clippy::match_same_arms,
    clippy::cast_lossless,
    clippy::manual_range_contains,
)]

// Const assertion for soundness
const _: () = assert!(
    size_of::<usize>() >= size_of::<u32>(),
    "Cannot safely convert from u32 to usize on this platform. Since GameMaker data files are \
     32-bit, this library will not function properly."
);

#[rustfmt::skip]
#[cfg(doc)]
use wad::{
    parse::{ParsingOptions, parse_bytes, parse_file},
    build::{build_bytes, build_file},
};

// Private modules
mod actions;
mod util;

// Public modules
#[cfg(doc)]
pub mod _spec;
pub mod error;
pub mod gm_enum;
pub mod gml;
pub mod prelude;
pub mod wad;

// Convenience re-exports
pub use self::error::Error;
pub use self::error::Result;

// === Some TODOs for the entire library ===
//
// When Rust finally drops Macros 2.0:
// * Migrate all `macro_rules!` to `macro`s and remove exporting from crate root.
// Reference: https://github.com/rust-lang/rust/issues/39412
//
// When most traits (`Into`, `TryInto`, `Iterator`, `PartialEq`) are const-stable:
// * Clean up all `const-hack`s
