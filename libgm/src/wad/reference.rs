//! Contains the `GMRef` type which is used to refer to other GameMaker elements.

use std::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
};

use crate::{prelude::*, util::fmt::typename};

/// A reference to another GameMaker element.
///
/// This is typically a Reference by ID in the data file format,
/// but is also used for texture page items (which are pointers).
///
/// [`GMRef`] has (fake) generic types to make it clearer which type it belongs to.
/// * Example without: `pub texture_mask: GMRef`
/// * Example with: `pub texture_mask: GMRef<GMSprite>`
///
/// It can be resolved to the data it references using the `.resolve()` method,
/// which needs the vector the elements are stored in.
/// This means that removing or inserting elements in the middle of
/// the list will shift all their `GMRef`s; breaking them.
pub struct GMRef<T> {
    /// The GameMaker ID / Index of this resource in the corresponding element vector.
    pub(crate) index: u32,

    /// Marker needs to be here to ignore "unused generic T" error; doesn't store any data.
    _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for GMRef<T> {}
impl<T> Clone for GMRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for GMRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for GMRef<T> {}

impl<T> Hash for GMRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> From<u32> for GMRef<T> {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

impl<T> From<usize> for GMRef<T> {
    fn from(index: usize) -> Self {
        Self::new(index as u32)
    }
}

impl<T> From<GMRef<T>> for u32 {
    fn from(gm_ref: GMRef<T>) -> Self {
        gm_ref.index
    }
}

impl<T> From<GMRef<T>> for usize {
    fn from(gm_ref: GMRef<T>) -> Self {
        gm_ref.index as Self
    }
}

impl<T> Debug for GMRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMRef<{}#{}>", typename::<T>(), self.index)
    }
}

impl<T> GMRef<T> {
    /// Creates a new GameMaker reference with the specified index.
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
    pub fn resolve(self, elements_by_index: &[T]) -> Result<&T> {
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

    /// Attempts to resolve this reference to an element in the given list by its index.
    ///
    /// Returns a reference to the element if the index is valid, or an error string if out of bounds.
    ///
    /// # Parameters
    /// - `elements_by_index`: A vector of elements indexed by `self.index`.
    ///
    /// # Errors
    /// Returns an error if `self.index` is out of bounds for the provided vector.
    pub fn resolve_mut(self, elements_by_index: &mut [T]) -> Result<&mut T> {
        let length = elements_by_index.len();
        let element = elements_by_index
            .get_mut(self.index as usize)
            .ok_or_else(|| {
                format!(
                    "Could not resolve {} reference with index {} in list with length {}",
                    typename::<T>(),
                    self.index,
                    length,
                )
            })?;
        Ok(element)
    }
}
