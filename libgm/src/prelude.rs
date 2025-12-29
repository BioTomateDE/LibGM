pub(crate) use crate::{error::bail, util::hint::*};
pub use crate::{
    error::{Context, Error, Result},
    gamemaker::{
        data::GMData,
        elements::{GMChunk, GMListChunk, GMNamedElement, GMNamedListChunk},
        reference::GMRef,
    },
};
