use std::usize;

use crate::gamemaker::elements::GMElement;
use crate::prelude::*;
use crate::util::fmt::typename;

/// GMRef has (fake) generic types to make it clearer which type it belongs to (`name: GMRef` vs `name: String`).
/// It can be resolved to the data it references using the `.resolve()` method, which needs the list the elements are stored in.
/// This means that removing or inserting elements in the middle of the list will shift all their `GMRef`s; breaking them.
#[derive(Hash, PartialEq, Eq)]
pub struct GMRef<T> {
    /// The `GameMaker` ID / Index of this resource in the corresponding element vector.
    pub index: u32,

    /// Marker needs to be here to ignore "unused generic T" error; doesn't store any data
    _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for GMRef<T> {}

impl<T> Clone for GMRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> From<u32> for GMRef<T> {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl<T> From<usize> for GMRef<T> {
    fn from(value: usize) -> Self {
        Self::new(value as u32)
    }
}

impl<T> Into<u32> for GMRef<T> {
    fn into(self) -> u32 {
        self.index
    }
}

impl<T> Into<usize> for GMRef<T> {
    fn into(self) -> usize {
        self.index as usize
    }
}

impl<T> std::fmt::Debug for GMRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMRef<{}#{}>", typename::<T>(), self.index)
    }
}

impl<T> GMRef<T> {
    /// Creates a new `GameMaker` reference with the specified index.
    /// The fake generic type can often be omitted (if the compiler can infer it).
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self { index, _marker: std::marker::PhantomData }
    }

    /// Attempts to resolve this reference to an element in the given list by its index.
    ///
    /// Returns a reference to the element if the index is valid, or an error string if out of bounds.
    ///
    /// # Parameters
    /// - `elements_by_index`: A vector of elements indexed by `self.index`.
    ///
    /// # Errors
    /// Returns an error if `self.index` is out of bounds for the provided vector.
    pub fn resolve<'a>(&self, elements_by_index: &'a Vec<T>) -> Result<&'a T> {
        let element = elements_by_index.get(self.index as usize).ok_or_else(|| {
            format!(
                "Could not resolve {} reference with index {} in list with length {}",
                typename::<T>(),
                self.index,
                elements_by_index.len(),
            )
        })?;
        Ok(element)
    }
}
