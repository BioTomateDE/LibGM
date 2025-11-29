use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
    context: Vec<String>,
}

impl Error {
    #[cold]
    pub const fn new(message: String) -> Self {
        Self { message, context: Vec::new() }
    }

    pub fn chain_with(&self, arrow: &str) -> String {
        let mut output = format!("{}", self.message);
        for context in &self.context {
            output += &format!("\n{arrow} while {context}");
        }
        output
    }

    pub fn chain(&self) -> String {
        self.chain_with(">")
    }

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

impl FromStr for Error {
    type Err = ();
    #[cold]
    fn from_str(message: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::new(message.to_string()))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T>;
}

impl<T> Context<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let mut error = e.clone();
            error.context.push(context.into());
            error
        })
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|e| {
            let mut error = e.clone();
            error.context.push(f().into());
            error
        })
    }
}

impl<T, S: Into<String>> Context<T> for std::result::Result<T, S> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|string| {
            let mut error = Error::new(string.into());
            error.context.push(context.into());
            error
        })
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|string| {
            let mut error = Error::new(string.into());
            error.context.push(f().into());
            error
        })
    }
}

macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::Error::new(format!($($arg)*)))
    };
}

pub(crate) use bail;
