use clap::ValueEnum;
use libgm::prelude::*;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EnableDebug,
    DisableDebug,
    DeserializeTextures,
}

impl Action {
    pub fn perform(self, data: &mut GMData) -> Result<()> {
        match self {
            Self::EnableDebug => data.enable_debug().context("enabling debug mode"),
            Self::DisableDebug => data.disable_debug().context("disabling debug mode"),
            Self::DeserializeTextures => data.deserialize_textures(),
        }
    }
}
