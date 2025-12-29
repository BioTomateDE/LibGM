pub use crate::{
    error::{Context, Error, Result},
    gamemaker::{
        data::GMData,
        elements::{GMChunk, GMListChunk, GMNamedElement, GMNamedListChunk},
        reference::GMRef,
    },
};
pub(crate) use crate::{
    error::{bail, err},
    util::hint::*,
};

