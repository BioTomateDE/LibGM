//! LibGM's custom error type is contained here, as well as a convenience type
//! alias for `Result`.
//!
//! Usually, you will see the most outer error cause first.
//! For example, in `anyhow`, you might see "Failed to read configuration"
//! and only the context chain (via `Option<Box<dyn ...>>`) reveals more
//! information about what actually caused this error.
//!
//! LibGM uses a different approach for its error system.
//! The most outer / broadest error would otherwise just always be "Failed to
//! parse data file" which conveys no relevant information. Instead, it displays
//! the root cause first. This can be trying to read data out of chunk bounds,
//! an assertion failing, an enum being an invalid value, etc.
//!
//! The specified context chain stores additional information in descending
//! order of importance, travelling down the call stack. The last element of the
//! context chain will be something very generic such as "Failed to parse data"
//! or similar.
//!
//! Additionally, LibGM Errors also store their potential error source in an
//! `Option<Box<dyn std::error::Error>>` for better integration with traditional
//! error systems. This source will only be set when failable functions from
//! other crates (or std) are called. The majority of LibGM errors are
//! assertions that only consist of text; no dynamic error source.
//!
//! ___
//!
//! When you write LibGM code, it is good practice to use the [`Context`] trait
//! frequently to make error traces better for end users (and maintainers trying
//! to debug). The string in your `.context()` calls should:
//! * start with a lowercase verb in the "-ing" form
//! * contain no punctuation at the end
//!
//! Examples: `.context("reading config file")` or `.with_context(||
//! format!("acquiring metadata of {file:?}"))`.
//!
//! Your root error messages (such as when using `bail!`) should:
//! * start with an uppercase letter
//! * be a complete sentence
//! * contain no punctuation at the end
//!
//! Examples: `bail!("Expected x, got {y}")` or `.ok_or("Code parent not
//! set")?`.

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;

/// A LibGM error.
///
/// Contains an error message, a context chain as well as a potential source
/// error.
///
/// For more information, see the [module level documentation][self].
#[derive(Debug)]
#[non_exhaustive]
pub struct Error {
    message: String,
    context: Vec<String>,
    source: Option<Box<dyn std::error::Error + 'static>>,
}

impl Error {
    /// Creates a new [`Error`] with the given message.
    ///
    /// The context chain and source will be empty.
    #[must_use]
    pub const fn new(message: String) -> Self {
        Self {
            message,
            context: Vec::new(),
            source: None,
        }
    }

    /// Adds an error source to this error.
    ///
    /// This function consumes `self` and returns a modified version of it.
    ///
    /// To retrieve the source, use the [`std::error::Error::source`] method.
    #[must_use = "returns a new error with the specified source"]
    pub fn with_source(self, source: Box<dyn std::error::Error>) -> Self {
        Self { source: Some(source), ..self }
    }

    /// Pushes context to the end of the context chain, in-place.
    pub fn push_context(&mut self, context: impl Into<String>) {
        self.context.push(context.into());
    }

    /// Pushes context to the end of the context chain.
    ///
    /// This function consumes `self` and returns a modified version of it.
    #[must_use = "returns a new error with additional context"]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref()
    }
}

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
    /// This pushes a string to the end of the error context chain
    /// in case of [`Err`] and is a no-op in case of [`Ok`].
    ///
    /// **Avoid allocating `String`s before the error actually occurred.**
    /// In that case, use [`Context::with_context`] for lazy evaluation instead.
    fn context(self, context: &str) -> Result<T>;

    /// Adds context to this [`Result`] using the given closure that returns a
    /// [`String`].
    ///
    /// The context to be appended is lazily evaluated,
    /// meaning the closure only executes if an error actually occurred.
    ///
    /// This makes it more suited for `format!` calls since it avoids a heap
    /// allocation in the common [`Ok`] case.
    ///
    /// For more information, see [`Context::context`].
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T>;
}

impl<T> Context<T> for Result<T> {
    fn context(self, context: &str) -> Self {
        self.map_err(|err| err.with_context(context))
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Self {
        self.map_err(|err| err.with_context(f()))
    }
}

impl<T> Context<T> for std::result::Result<T, &str> {
    fn context(self, context: &str) -> Result<T> {
        self.map_err(|error| Error::new(error.to_owned()).with_context(context))
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|error| Error::new(error.to_owned()).with_context(f()))
    }
}

impl<T> Context<T> for std::result::Result<T, String> {
    fn context(self, context: &str) -> Result<T> {
        self.map_err(|error| Error::new(error).with_context(context))
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|error| Error::new(error).with_context(f()))
    }
}

/// Trait for adding context to the context chain of a [`Result`]
/// and a [`std::error::Error`] source error.
///
/// This trait is meant to be used on `Result<T, E>` where `E` is not LibGM's
/// [`Error`] (e.g. when performing IO operations or calling functions from
/// other crates).
///
/// It works on any `Result` (as long as `E` implements `std::error::Error`) and
/// automatically adds the boxed source error as further context.
///
/// For more information, see [`Context`].
pub trait ContextSrc<T> {
    /// Adds context to this [`Result`] and sets the error source.
    ///
    /// For more information, see [`Context::context`].
    fn context_src(self, context: &str) -> Result<T>;

    /// Adds context to this [`Result`] using the given closure that returns a
    /// [`String`] and sets the error source.
    ///
    /// For more information, see [`Context::with_context`].
    fn with_context_src(self, f: impl FnOnce() -> String) -> Result<T>;
}

impl<T, E: std::error::Error + 'static> ContextSrc<T> for std::result::Result<T, E> {
    fn context_src(self, context: &str) -> Result<T> {
        self.map_err(|error| {
            Error::new(ascii_capitalize(error.to_string()))
                .with_context(context)
                .with_source(Box::new(error))
        })
    }

    fn with_context_src(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|error| {
            Error::new(ascii_capitalize(error.to_string()))
                .with_context(f())
                .with_source(Box::new(error))
        })
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
