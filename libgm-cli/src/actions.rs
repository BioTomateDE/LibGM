use clap::ValueEnum;
use libgm::{prelude::*, wad::elements::texture_page::Format};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EnableDebug,
    DisableDebug,
    DeserializeTextures,
    SerializeTexturesPng,
    SerializeTexturesQoi,
    SerializeTexturesBz2Qoi,
    OptimizeMemory,
    CollapseEvents,
    Most,
}

impl Action {
    pub fn perform(self, data: &mut GMData) -> Result<()> {
        match self {
            Self::EnableDebug => data.enable_debug(),
            Self::DisableDebug => data.disable_debug(),
            Self::DeserializeTextures => data.deserialize_all_textures(),
            Self::SerializeTexturesPng => serialize_textures(data, Format::Png),
            Self::SerializeTexturesQoi => serialize_textures(data, Format::Qoi),
            Self::SerializeTexturesBz2Qoi => serialize_textures(data, Format::Bz2Qoi),
            Self::OptimizeMemory => {
                data.optimize_memory();
                Ok(())
            },
            Self::CollapseEvents => {
                data.collapse_all_events();
                Ok(())
            },
            Self::Most => data.post_deserialize(),
        }
    }
}

fn serialize_textures(data: &mut GMData, format: Format) -> Result<()> {
    for texture_page in &mut data.texture_pages {
        let Some(image) = &mut texture_page.image else {
            continue;
        };
        image.change_format(format)?;
    }
    Ok(())
}
