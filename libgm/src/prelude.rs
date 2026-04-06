//! The prelude contains commonly used items when working with LibGM.
//! This includes error types, a result alias, traits and more.
//! You can import it by using `use libgm::prelude::*;`.

pub use crate::error::Context;
pub use crate::error::ContextSrc;
pub use crate::error::Error;
pub use crate::error::Result;
pub use crate::error::bail;
pub use crate::error::err;
pub use crate::wad::data::GMData;
pub use crate::wad::elements::GMChunk;
pub use crate::wad::elements::GMListChunk;
pub use crate::wad::elements::GMNamedElement;
pub use crate::wad::elements::GMNamedListChunk;
pub use crate::wad::reference::GMRef;
