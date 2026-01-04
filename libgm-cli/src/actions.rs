mod enable_debug;

use clap::ValueEnum;
use libgm::{gamemaker::data::GMData, prelude::*};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EnableDebug,
    DeserializeTextures,
}

impl Action {
    pub fn perform(self, data: &mut GMData) -> Result<()> {
        match self {
            Self::EnableDebug => enable_debug::perform(data).context("enabling debug mode"),
            Self::DeserializeTextures => data.deserialize_textures(),
        }
    }
}
