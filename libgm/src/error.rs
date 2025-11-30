use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    #[must_use]
    pub fn chain_with(&self, arrow: &str) -> String {
        use std::fmt::Write;
        let mut output = self.message.clone();
        for context in &self.context {
            write!(output, "\n{arrow} while {context}").unwrap();
        }
        output
    }

    #[must_use]
    pub fn chain(&self) -> String {
        self.chain_with(">")
    }

    #[must_use]
    pub fn chain_pretty(&self) -> String {
        self.chain_with("↳")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// Convenience type alias for [`std::fmt::Result`] with [`crate::error::Error`] as the error type.
/// This type is also exported in `libgm::prelude`.
pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T>;
}

impl<T> Context<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Self {
        self.map_err(|mut err| {
            err.context.push(context.into());
            err
        })
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Self {
        self.map_err(|mut err| {
            err.context.push(f());
            err
        })
    }
}

impl<T, S: Into<String>> Context<T> for std::result::Result<T, S> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|string| {
            let mut err = Error::new(string.into());
            err.context.push(context.into());
            err
        })
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|string| {
            let mut err = Error::new(string.into());
            err.context.push(f());
            err
        })
    }
}

macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::Error::new(format!($($arg)*)))
    };
}

pub(crate) use bail;
