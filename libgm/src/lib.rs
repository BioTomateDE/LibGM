//! todo: actual docstring

// These `must_use`s are usually critical and indicate a serious logical flaw.
// If you know that these are unused and are just using them for debugging purposes,
// you can temporarily comment this out.
#![deny(unused_must_use)]
//
// Same thing here.
#![deny(unreachable_patterns)]
//
//
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
//
// Reading data can always fail.
// No need to put the same `Errors` header everywhere.
#![allow(clippy::missing_errors_doc)]
//
// Reinterpreting the bits is the intended behavior.
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
//
// I sometimes need more than 3 bools in a struct???
// This lint is only relevant for people
// who have never heard of an enum.
#![allow(clippy::struct_excessive_bools)]
//
// "Cast from usize to u32 can truncate".
// This is irrelevant because all `usize`s are
// related to element count or data length.
// If either of those is out of `u32` bounds,
// the data reading/parsing will fail anyway
// since data files are only allowed to be
// smaller than 2 GB (`i32` limit).
#![allow(clippy::cast_possible_truncation)]

mod error;
mod util;

pub mod gamemaker;
pub mod gml;

pub mod prelude;
