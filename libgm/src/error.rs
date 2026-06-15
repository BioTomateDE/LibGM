// SPDX-License-Identifier: GPL-3.0-only
//! LibGM's custom error type is contained here,
//! as well as a convenience type alias for `Result`.
//!
//! Usually in Rust, you will see the most outer error cause first.
//! For example, in `anyhow`, you might see "Failed to read configuration"
//! and only the context chain (via `Option<Box<dyn ...>>`) reveals more
//! information about what actually caused this error.
//!
//! LibGM uses a different approach for its error system.
//! The most outer / broadest error would otherwise just always be "Failed to
//! parse/build data file" which conveys no relevant information. Instead, it displays
//! the root cause first. This can be trying to read data out of chunk bounds,
//! an assertion failing, an enum being an invalid value, etc.
//!
//! The specified context chain stores additional information in descending
//! order of importance, travelling down the call stack. The last element of the
//! context chain will be something very generic such as "Failed to parse data"
//! or similar.
//! ___________________
//!
//! When you write LibGM code, it is good practice to use the [`Context`] trait
//! frequently to make error traces better for end users (and maintainers trying to debug).
//! The string in your `.ctx()` calls should:
//! * start with a lowercase verb in the "-ing" form
//! * contain no punctuation at the end
//!
//! Examples: `.ctx("reading config file")` or
//! `.ctx(|| format!("acquiring metadata of {file:?}"))`.
//!
//! Your root error messages (such as when using `bail!`) should:
//! * start with an uppercase letter
//! * be a complete sentence
//! * contain no punctuation at the end
//!
//! Examples: `bail!("Expected x, got {y}")` or `.ok_or("Code parent not set")?`.

use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;

/// A LibGM error.
///
/// Contains an error message and a context chain.
///
/// For more information, see the [module level documentation][self].
#[derive(Debug)]
#[non_exhaustive]
pub struct Error {
    message: String,
    context: Vec<String>,
}

impl Error {
    /// Creates a new [`Error`] with the given message.
    ///
    /// The context chain will be empty.
    #[must_use]
    pub const fn new(message: String) -> Self {
        Self { message, context: Vec::new() }
    }

    /// Pushes context to the end of the context chain, in-place.
    pub fn push_context(&mut self, context: impl StringLike) {
        self.context.push(context.into_string());
    }

    /// Pushes context to the end of the context chain.
    ///
    /// This function consumes `self` and returns a modified version of it.
    #[must_use = "returns a new error with additional context"]
    pub fn with_context(mut self, context: impl StringLike) -> Self {
        self.push_context(context);
        self
    }

    /// Print out the chain with the specified arrow character.
    ///
    /// This will span multiple lines: one for the required message
    /// and one more for each string in the context chain.
    ///
    /// The context chain will be printed in forward order and the word
    /// "while" will be inserted before each context string.
    ///
    /// NOTE: Printing the chain is preferred over using the [`Display`] trait
    /// directly. Otherwise, the context chain is lost.
    #[must_use]
    pub fn chain_with(&self, arrow: &str) -> String {
        let mut output = self.message.clone();
        for context in &self.context {
            let _ = write!(output, "\n{arrow} while {context}");
        }
        output
    }

    /// Prints out the error chain with `>` as an arrow character.
    ///
    /// For more information about printing chains, see [`Self::chain_with`].
    #[must_use]
    pub fn chain(&self) -> String {
        self.chain_with(">")
    }

    /// Prints out the error chain with the Unicode character `↳` as an arrow.
    ///
    /// For more information about printing chains, see [`Self::chain_with`].
    #[must_use]
    pub fn chain_pretty(&self) -> String {
        self.chain_with("↳")
    }
}

/// NOTE: The `Display` implementation only prints out the root message.
/// To use the context chain, consider printing using [`Error::chain`] instead.
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::new(message.to_string())
    }
}

/// Types that can easily be converted to [`String`]s, including closures.
///
/// Any `FnOnce` that returns something that is [`StringLike`] will also implement this trait.
/// In its definition, it will call the closure to get the string-like value.
/// This is useful because it can combine eager and lazy evalutation into one trait (and therefore one context adding method).
pub trait StringLike {
    fn into_string(self) -> String;
}

impl StringLike for &str {
    fn into_string(self) -> String {
        self.to_owned()
    }
}

impl StringLike for String {
    fn into_string(self) -> String {
        self
    }
}

impl StringLike for Cow<'_, str> {
    fn into_string(self) -> String {
        self.into_owned()
    }
}

impl<S: StringLike, F: FnOnce() -> S> StringLike for F {
    fn into_string(self) -> String {
        self().into_string()
    }
}

/// Convenience type alias of [`std::result::Result`] with error type [`Error`].
///
/// This type alias is also re-exported in `libgm::prelude`.
pub type Result<T> = std::result::Result<T, Error>;

/// Trait for adding context to the context chain of a [`Result`].
/// This should be used frequently to create a better stack trace.
///
/// This trait is also re-exported in `libgm::prelude`.
pub trait Context<T> {
    /// Adds context to this [`Result`].
    ///
    /// This pushes a string to the end of the error context chain
    /// in case of [`Err`] and is a no-op in case of [`Ok`].
    ///
    /// **Avoid allocating `String`s before the error actually.**
    /// This usually happens when you use `format!`.
    /// Just wrap it in a closure: `|| format!(...)`, now it's lazily evaluated.
    fn ctx(self, context: impl StringLike) -> Result<T>;
}

impl<T> Context<T> for Result<T> {
    fn ctx(self, context: impl StringLike) -> Self {
        self.map_err(|err| err.with_context(context.into_string()))
    }
}

// useful for `.ok_or` and `.ok_or_else` chains -->
impl<T> Context<T> for std::result::Result<T, &str> {
    fn ctx(self, context: impl StringLike) -> Result<T> {
        self.map_err(|e| Error::new(e.to_owned()).with_context(context))
    }
}

impl<T> Context<T> for std::result::Result<T, String> {
    fn ctx(self, context: impl StringLike) -> Result<T> {
        self.map_err(|e| Error::new(e).with_context(context))
    }
}
// <--

/// Trait for adding context to the context chain of a [`Result`]
/// and a [`std::error::Error`] source error.
///
/// This trait is meant to be used on `Result<T, E>` where `E` is not LibGM's
/// [`Error`] (e.g. when performing IO operations or calling functions from other crates).
///
/// It works on any `Result` (as long as `E` implements `std::error::Error`).
pub trait ContextAny<T> {
    /// Adds context to this [`Result`] from a generic result with a printable error type.
    ///
    /// The error message will be capitalized.
    ///
    /// For more information, see [`Context::ctx`].
    fn ctx_any(self, context: impl StringLike) -> Result<T>;
}

impl<T, E: std::error::Error> ContextAny<T> for std::result::Result<T, E> {
    fn ctx_any(self, context: impl StringLike) -> Result<T> {
        self.map_err(|e| Error::new(ascii_capitalize(e.to_string())).with_context(context))
    }
}

#[must_use]
fn ascii_capitalize(mut string: String) -> String {
    if let Some(first) = string.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    string
}

/// Creates a new [LibGM Error](Error) using the specified format string.
///
/// This is a simple alias for `Error::new(format!(...)`.
///
/// This macro is currently exported at crate root but this may change in the
/// future when [Macros 2.0](https://github.com/rust-lang/rust/issues/39412) are stabilized.
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {
        $crate::error::Error::new(format!($($arg)*))
    };
}

/// Performs an early return with the specified formatted message.
///
/// This is a simple alias for `return Err(Error::new(format!(...));`.
///
/// This macro is currently exported at crate root but this may change in the
/// future when [Macros 2.0](https://github.com/rust-lang/rust/issues/39412) are stabilized.
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::Error::new(format!($($arg)*)))
    };
}

// Re-export macros in libgm::error
pub use bail;
pub use err;
