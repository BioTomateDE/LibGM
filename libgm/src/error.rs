use std::fmt::{Display, Formatter, Write};

/// A LibGM error.
/// Contains an error message as well as a context chain.
///
#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    context: Vec<String>,
}

impl Error {
    #[cold]
    #[must_use]
    pub const fn new(message: String) -> Self {
        Self { message, context: Vec::new() }
    }

    /// Add context in-place.
    pub fn add_context(&mut self, context: impl Into<String>) {
        self.context.push(context.into());
    }

    /// Add context and return itself.
    #[must_use = "returns a new error with additional context"]
    pub fn push_context(mut self, context: impl Into<String>) -> Self {
        self.add_context(context);
        self
    }

    /// Print out the chain with the specified arrow character.
    ///
    /// Printing the chain is preferred over using the `Display` trait directly.
    /// Otherwise, the context chain is lost.
    #[must_use]
    pub fn chain_with(&self, arrow: &str) -> String {
        let mut output = self.message.clone();
        for context in &self.context {
            let _ = write!(output, "\n{arrow} while {context}");
        }
        output
    }

    /// Print out the error chain with `>` as an arrow character.
    ///
    /// For more information about printing chains, see [`Self::chain_with`].
    #[must_use]
    pub fn chain(&self) -> String {
        self.chain_with(">")
    }

    /// Print out the error chain with the unicode character `↳` as an arrow.
    ///
    /// For more information about printing chains, see [`Self::chain_with`].
    #[must_use]
    pub fn chain_pretty(&self) -> String {
        self.chain_with("↳")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for Error {
    #[cold]
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for Error {
    #[cold]
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
    fn context(self, context: impl Into<String>) -> Result<T>;
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T>;
}

impl<T> Context<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Self {
        self.map_err(|err| err.push_context(context))
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Self {
        self.map_err(|err| err.push_context(f()))
    }
}

impl<T, S: Into<String>> Context<T> for std::result::Result<T, S> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|string| Error::new(string.into()).push_context(context))
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|string| Error::new(string.into()).push_context(f()))
    }
}

/// Perform an early return with the specified formatted message.
/// This is a simple alias for `return Err(Error::new(format!(...));`.
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::Error::new(format!($($arg)*)))
    };
}

pub(crate) use bail;
