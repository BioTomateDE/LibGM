use std::fmt;

use crate::gml::highlevel::Location;

#[derive(Debug, Clone)]
pub struct CompileError<'a> {
    pub(super) error: crate::Error,
    pub(super) source_code: &'a str,
    pub start_position: Location,
    pub end_position: Location,
}

impl<'a> CompileError<'a> {
    /// The part of the source code where the error originated.
    #[must_use]
    pub fn code_part(&self) -> &'a str {
        let start = self.start_position.byte as usize;
        let end = self.end_position.byte as usize;
        &self.source_code[start..end]
    }

    /// The one-indexed line number where this error originated.
    #[must_use]
    pub fn line(&self) -> u32 {
        self.start_position.line + 1
    }

    /// The one-indexed character number (on the corresponding line) where this error originated.
    #[must_use]
    pub fn char(&self) -> u32 {
        self.start_position.char + 1
    }

    /// The underlying [`libgm::Error`] this [`CompileError`] stores.
    ///
    /// This function consumes the error.
    /// If you need a reference, use [`Self::libgm_error_ref`] instead.
    ///
    /// [`libgm::Error`]: crate::Error
    #[must_use]
    pub fn libgm_error(self) -> crate::Error {
        self.error
    }

    /// A reference to the underlying [`libgm::Error`] this [`CompileError`] stores.
    ///
    /// If you need ownership of the LibGM error, use [`Self::libgm_error`] instead.
    ///
    /// [`libgm::Error`]: crate::Error
    #[must_use]
    pub fn libgm_error_ref(&self) -> &crate::Error {
        &self.error
    }
}

impl fmt::Display for CompileError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code: &str = self.code_part();
        let line = self.line();
        let char = self.char();
        let e = self.libgm_error_ref();
        write!(f, "Compilation error at line {line}:{char} ({code:?}): {e}")
    }
}

impl std::error::Error for CompileError<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
