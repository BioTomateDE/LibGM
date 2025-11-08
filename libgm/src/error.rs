use std::fmt::{Display, Write};
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
#[error("{context}")]
pub struct Error {
    context: String,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    #[cold]
    pub const fn new(context: String) -> Self {
        Self { context, source: None }
    }

    pub fn chain_vec(&self) -> Vec<String> {
        let mut chain = vec![self.context.clone()];

        let mut source = self.source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error);
        while let Some(err) = source {
            chain.push(format!("{}", err));
            source = err.source();
        }

        chain.reverse();
        chain
    }

    pub fn chain_with(&self, arrow: &str) -> String {
        let mut chain = vec![&self.context as &dyn Display];
        let mut source = self.source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error);

        while let Some(err) = source {
            chain.push(err as &dyn Display);
            source = err.source();
        }
        chain.reverse();

        let mut output = format!("{}", chain[0]);
        for context in &chain[1..] {
            write!(&mut output, "\n{arrow} while {context}").unwrap();
        }
        output
    }

    pub fn chain(&self) -> String {
        self.chain_with(">")
    }
}

impl From<String> for Error {
    #[cold]
    fn from(context: String) -> Self {
        Self { context, source: None }
    }
}

impl From<&str> for Error {
    #[cold]
    fn from(context: &str) -> Self {
        Self { context: context.to_string(), source: None }
    }
}

impl FromStr for Error {
    type Err = ();
    #[cold]
    fn from_str(string: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self { context: string.to_string(), source: None })
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> Context<T> for std::result::Result<T, E> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| Error { context: context.into(), source: Some(Box::new(e)) })
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        self.map_err(|e| Error { context: f(), source: Some(Box::new(e)) })
    }
}

impl<T> Context<T> for Option<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| Error::new(context.into()))
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        self.ok_or_else(|| Error::new(f()))
    }
}

macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::Error::new(format!($($arg)*)))
    };
}

pub(crate) use bail;
