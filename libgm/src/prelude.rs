// SPDX-License-Identifier: GPL-3.0-only
//! The prelude contains commonly used items when working with LibGM.
//! This includes error types, a result alias, traits and more.
//! You can import it by using `use libgm::prelude::*;`.

pub use crate::error::Context;
pub use crate::error::ContextAny;
pub use crate::error::Error;
pub use crate::error::Result;
pub use crate::error::bail;
pub use crate::error::err;
pub use crate::wad::GMRef;
pub use crate::wad::chunk::GMChunk;
pub use crate::wad::chunk::GMDirectListChunk;
pub use crate::wad::chunk::GMListChunk;
pub use crate::wad::chunk::GMNamedListChunk;
pub use crate::wad::chunk::GMNullableListChunk;
pub use crate::wad::data::GMData;
pub use crate::wad::elem::GMNamedElement;
