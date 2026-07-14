// SPDX-License-Identifier: GPL-3.0-only
//! Everything related to parsing and building of GameMaker data files.

mod memory;
mod reference;
mod version_detection;

pub mod build;
pub mod chunk;
pub mod data;
pub mod elem;
pub mod parse;
pub mod version;

use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;

pub use self::build::build_bytes;
pub use self::build::build_file;
pub use self::data::GMData;
pub use self::parse::parse_bytes;
pub use self::parse::parse_file;
pub use self::reference::GMRef;
pub use self::version::GMVersion;
use crate::util::fmt::typename;

/// A wrapper struct that holds a vector or array.
///
/// This allows for Debug derives but does not print out the entire data as numbers.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Blob<T: BlobLike>(pub T);

impl<T: BlobLike> fmt::Debug for Blob<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blob<{}%{}>", typename::<T>(), self.0.len())
    }
}

impl<T: BlobLike> Deref for Blob<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BlobLike> DerefMut for Blob<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait BlobLike: private::Sealed {
    #[must_use]
    fn len(&self) -> usize;
}

impl<T: Copy> private::Sealed for Vec<T> {}
impl<T: Copy> BlobLike for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T: Copy> private::Sealed for &[T] {}
impl<T: Copy> BlobLike for &[T] {
    fn len(&self) -> usize {
        (*self).len()
    }
}

impl<T: Copy, const N: usize> private::Sealed for [T; N] {}
impl<T: Copy, const N: usize> BlobLike for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

mod private {
    pub trait Sealed {}
}
