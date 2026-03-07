use std::fmt::{Display, Formatter};

use crate::{prelude::*, util::fmt::hexdump};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ChunkName {
    bytes: [u8; 4],
}

impl ChunkName {
    /// This function panics on invalid chunk names.
    /// It is only meant to be used in `const` contexts
    /// where panics result in a compile error.
    pub const fn new(name: &'static str) -> Self {
        assert!(name.len() == 4, "Expected string of length 4");

        let bytes = name.as_bytes();

        // TODO(const-hack): Iterators are not const stable.
        let mut i = 0;
        while i < 4 {
            assert!(
                validate_char(bytes[i]),
                "Expected chunk name to only consist of uppercase ASCII letters and digits",
            );
            i += 1;
        }

        // TODO(const-hack): `try_into` is not const stable.
        let bytes: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
        Self { bytes }
    }

    #[inline]
    pub fn from_bytes(bytes: [u8; 4]) -> Result<Self> {
        let valid: bool = bytes.iter().all(|&byte| validate_char(byte));
        if !valid {
            let hexdump = hexdump(&bytes);
            bail!(
                "Expected chunk name [{hexdump}] to only \
                consist of uppercase ASCII letters and digits"
            );
        }
        Ok(Self { bytes })
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(self) -> [u8; 4] {
        self.bytes
    }

    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        // Safe because we validated UTF-8 in constructor
        unsafe { std::str::from_utf8_unchecked(&self.bytes) }
    }
}

impl Display for ChunkName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for ChunkName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{self}'")
    }
}

#[inline]
const fn validate_char(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'0'..=b'9')
}
