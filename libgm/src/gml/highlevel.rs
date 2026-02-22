pub mod compile;
pub mod token;

/// A location in source code.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Location {
    /// The zero-indexed line index in the source code string.
    line: u32,

    /// The zero-indexed character index in that line.
    char: u32,

    /// The byte index in the source code string.
    byte: u32,
}
