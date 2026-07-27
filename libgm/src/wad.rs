// SPDX-License-Identifier: GPL-3.0-only
//! Everything related to parsing and building of GameMaker data files.

mod memory;
mod reference;
mod version_detection;

pub mod chunk;
pub mod data;
pub mod elem;
pub mod load;
pub mod save;
pub mod version;

use std::any::type_name;
use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;

pub use self::data::GMData;
pub use self::load::load_bytes;
pub use self::load::load_file;
pub use self::reference::GMRef;
pub use self::save::save_bytes;
pub use self::save::save_file;
pub use self::version::GMVersion;

// use-aliases don't show deprecated warnings for some reason
#[deprecated(since = "0.8.0", note = "renamed to load_bytes")]
pub fn parse_bytes(raw_data: impl AsRef<[u8]>) -> crate::Result<GMData> {
    load_bytes(raw_data)
}

#[deprecated(since = "0.8.0", note = "renamed to load_file")]
pub fn parse_file(data_file_path: impl AsRef<std::path::Path>) -> crate::Result<GMData> {
    load_file(data_file_path)
}

#[deprecated(since = "0.8.0", note = "renamed to save_bytes")]
pub fn build_bytes(gm_data: &GMData) -> crate::Result<Vec<u8>> {
    save_bytes(gm_data)
}

#[deprecated(since = "0.8.0", note = "renamed to save_bytes")]
pub fn build_file(gm_data: &GMData, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
    save_file(gm_data, path)
}

/// A wrapper struct that holds a vector or array.
///
/// This allows for Debug derives but does not print out the entire data as numbers.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Blob<T: BlobLike>(pub T);

impl<T: BlobLike> fmt::Debug for Blob<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debugfmt(f)
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
    fn debugfmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<T: Copy> private::Sealed for Vec<T> {}
impl<T: Copy> BlobLike for Vec<T> {
    fn debugfmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Blob<Vec<{}>#{}>", type_name::<T>(), self.len())
    }
}

impl<T: Copy> private::Sealed for &[T] {}
impl<T: Copy> BlobLike for &[T] {
    fn debugfmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Blob<[{}]#{}>", type_name::<T>(), self.len())
    }
}

impl<T: Copy, const N: usize> private::Sealed for [T; N] {}
impl<T: Copy, const N: usize> BlobLike for [T; N] {
    fn debugfmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Blob<[{}; {}]>", type_name::<T>(), self.len())
    }
}

mod private {
    pub trait Sealed {}
}
