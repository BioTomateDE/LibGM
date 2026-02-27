//! The prelude contains commonly used items when working with LibGM.
//! This includes error types, a result alias, traits and more.
//! You can import it by using `use libgm::prelude::*;`.

pub use crate::{
    error::{Context, Error, Result, bail, err},
    gamemaker::{
        data::GMData,
        elements::{GMChunk, GMListChunk, GMNamedElement, GMNamedListChunk},
        reference::GMRef,
    },
};
