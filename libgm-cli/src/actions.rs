use clap::ValueEnum;
use libgm::{actions::toggle_debug, gamemaker::data::GMData, prelude::*};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EnableDebug,
    DisableDebug,
    DeserializeTextures,
}

impl Action {
    pub fn perform(self, data: &mut GMData) -> Result<()> {
        match self {
            Self::EnableDebug => toggle_debug(data, true).context("enabling debug mode"),
            Self::DisableDebug => toggle_debug(data, false).context("disabling debug mode"),
            Self::DeserializeTextures => data.deserialize_textures(),
        }
    }
}
