// SPDX-License-Identifier: GPL-3.0-only
//! Contains the `GMRef` type which is used to refer to other GameMaker elements.

use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::hint::cold_path;
use std::marker::PhantomData;

use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::elem::string::Strings;

/// An optional reference to another GameMaker element.
///
/// This is typically a Reference by ID in the data file format,
/// but is also used for texture page items (which are pointers).
///
/// [`GMRef`] has (fake) generic types to make it clearer which type it belongs
/// to.
/// * Example without: `pub texture_mask: GMRef`
/// * Example with: `pub texture_mask: GMRef<Sprite>`
///
/// It can be resolved to the data it references using the `.resolve()` method,
/// which needs the vector the elements are stored in.
/// This means that removing or inserting elements in the middle of
/// the list will shift all their `GMRef`s; breaking them.
#[repr(transparent)]
pub struct GMRef<T> {
    /// The GameMaker ID / Index of this resource in the corresponding element vector.
    ///
    /// This will be negative if the reference is null.
    pub(crate) index: i32,

    /// Marker needs to be here to ignore "unused generic T" error;
    /// doesn't store any data.
    _marker: PhantomData<T>,
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

impl<T> Default for GMRef<T> {
    fn default() -> Self {
        Self::none()
    }
}

impl<T> From<u32> for GMRef<T> {
    fn from(index: u32) -> Self {
        debug_assert!(index < i32::MAX as u32, "Invalid GMRef index {index}");
        Self::new(index as i32)
    }
}

impl<T> From<usize> for GMRef<T> {
    fn from(index: usize) -> Self {
        debug_assert!(index < i32::MAX as usize, "Invalid GMRef index {index}");
        Self::new(index as i32)
    }
}

// impl<T> From<Option<u32>> for GMRef<T> {
//     fn from(index: Option<u32>) -> Self {
//         match index {
//             Some(idx) => Self::new(idx),
//             None => Self::null(),
//         }
//     }
// }

// impl<T> From<Option<usize>> for GMRef<T> {
//     fn from(index: Option<usize>) -> Self {
//         match index {
//             Some(idx) => Self::from(idx),
//             None => Self::null(),
//         }
//     }
// }

// impl<T> From<GMRef<T>> for u32 {
//     fn from(gm_ref: GMRef<T>) -> Self {
//         gm_ref.index
//     }
// }

// impl<T> From<GMRef<T>> for usize {
//     fn from(gm_ref: GMRef<T>) -> Self {
//         gm_ref.index as Self
//     }
// }

impl<T> fmt::Debug for GMRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_some() {
            write!(f, "GMRef<{}#{}>", typename::<T>(), self.index)
        } else {
            write!(f, "GMRef<{}#null>", typename::<T>())
        }
    }
}

impl<T> GMRef<T> {
    /// Creates a new GameMaker reference with the specified index.
    ///
    /// If the specified index is negative, then this reference is
    /// counted as null and will not point to anything.
    #[must_use]
    pub const fn new(index: i32) -> Self {
        Self { index, _marker: PhantomData }
    }

    /// Creates a null GameMaker reference which does not point to anything.
    #[must_use]
    pub const fn none() -> Self {
        Self { index: -1, _marker: PhantomData }
    }

    #[must_use]
    pub const fn is_some(self) -> bool {
        self.index >= 0
    }

    #[must_use]
    pub const fn is_none(self) -> bool {
        self.index < 0
    }

    #[must_use]
    pub const fn index(self) -> Option<usize> {
        if self.is_some() {
            Some(self.index as usize)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        self.index
    }

    /// Attempts to resolve this reference to an element in the given slice by its index.
    ///
    /// # Parameters
    /// - `elements`: A vector of elements indexed by `self.index`.
    ///
    /// # Errors
    /// Returns an error if `self.index` is out of bounds for the provided vector.
    pub fn resolve(self, elements: &[T]) -> Result<&T> {
        let Some(idx) = self.index() else {
            cold_path();
            bail!("The reference {self:?} is none and therefore does not point to anything");
        };
        let len = elements.len();
        let elem = elements.get(idx).ok_or_else(|| {
            cold_path();
            format!("The reference {self:?} is out of bounds for elements vector with length {len}")
        })?;
        Ok(elem)
    }

    /// Attempts to resolve this reference to an element in the given slice by its index.
    ///
    /// # Parameters
    /// - `elements`: A vector of elements indexed by `self.index`.
    ///
    /// # Errors
    /// Returns an error if `self.index` is out of bounds for the provided vector.
    pub fn resolve_mut(self, elements: &mut [T]) -> Result<&mut T> {
        let Some(idx) = self.index() else {
            cold_path();
            bail!("The reference {self:?} is none and therefore does not point to anything");
        };
        let len = elements.len();
        let elem: &mut T = elements.get_mut(idx).ok_or_else(|| {
            cold_path();
            format!("The reference {self:?} is out of bounds for elements vector with length {len}")
        })?;
        Ok(elem)
    }

    /// Attempts to resolve this reference to an element in the given slice by its index.
    ///
    /// # Parameters
    /// - `elements`: A vector of nullable elements indexed by `self.index`.
    ///
    /// # Errors
    /// Returns an error if `self.index` is out of bounds for the provided vector
    /// or if the specified element is [`None`].
    pub fn opt_resolve(self, elements: &[Option<T>]) -> Result<&T> {
        let Some(idx) = self.index() else {
            cold_path();
            bail!("The reference {self:?} is none and therefore does not point to anything");
        };
        let len = elements.len();
        let element: &Option<T> = elements.get(idx).ok_or_else(|| {
            cold_path();
            format!("The reference {self:?} is out of bounds for elements vector with length {len}")
        })?;
        element.as_ref().ok_or_else(|| {
            cold_path();
            err!("The reference {self:?} points to a null element (removed by asset compiler)")
        })
    }

    /// Attempts to resolve this reference to an element in the given slice by its index.
    ///
    /// # Parameters
    /// - `elements`: A vector of nullable elements indexed by `self.index`.
    ///
    /// # Errors
    /// Returns an error if `self.index` is out of bounds for the provided vector
    /// or if the specified element is [`None`].
    pub fn opt_resolve_mut(self, elements: &mut [Option<T>]) -> Result<&mut T> {
        let Some(idx) = self.index() else {
            cold_path();
            bail!("The reference {self:?} is none and therefore does not point to anything");
        };
        let len = elements.len();
        let element: &mut Option<T> = elements.get_mut(idx).ok_or_else(|| {
            cold_path();
            format!("The reference {self:?} is out of bounds for elements vector with length {len}")
        })?;
        element.as_mut().ok_or_else(|| {
            cold_path();
            err!("The reference {self:?} points to a null element (removed by asset compiler)")
        })
    }
}

impl GMRef<String> {
    #[must_use]
    pub fn display(self, gm_strings: &Strings) -> &str {
        self.resolve(&gm_strings.elems)
            .map_or("<invalid string ref>", String::as_str)
    }
}
