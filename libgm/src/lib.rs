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
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
//
// Reading data can always fail (e.g. trying to read out of bounds).
// Almost all parser related functions have to read
// data at some point down the call hierarchy.
// Putting the same `Errors` header everywhere is meaningless.
#![allow(clippy::missing_errors_doc)]
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
// Reinterpreting the bits when using `as`-casts
// is the intended behavior.
// When trying to "safely convert" between signed and
// unsigned integers, use `try_from` instead.
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
//
// I sometimes need more than 3 bools in a struct???
// This lint is only relevant for people
// who have never heard of an enum.
#![allow(clippy::struct_excessive_bools)]
//
// This is a stylistic preference of mine.
// I may change this in the future.
#![allow(clippy::useless_let_if_seq)]

mod error;
mod util;

pub mod gamemaker;
pub mod gml;
pub mod prelude;
