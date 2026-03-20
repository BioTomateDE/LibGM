use clap::ValueEnum;
use libgm::{prelude::*, wad::elements::embedded_texture::Format};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EnableDebug,
    DisableDebug,
    DeserializeTextures,
    SerializeTexturesPng,
    SerializeTexturesQoi,
    SerializeTexturesBz2Qoi,
    OptimizeMemory,
}

impl Action {
    pub fn perform(self, data: &mut GMData) -> Result<()> {
        match self {
            Self::EnableDebug => data.enable_debug(),
            Self::DisableDebug => data.disable_debug(),
            Self::DeserializeTextures => data.deserialize_textures(),
            Self::SerializeTexturesPng => serialize_textures(data, Format::Png),
            Self::SerializeTexturesQoi => serialize_textures(data, Format::Qoi),
            Self::SerializeTexturesBz2Qoi => serialize_textures(data, Format::Bz2Qoi),
            Self::OptimizeMemory => {
                data.optimize_memory();
                Ok(())
            },
        }
    }
}

fn serialize_textures(data: &mut GMData, format: Format) -> Result<()> {
    for texture_page in &mut data.embedded_textures {
        let Some(image) = &mut texture_page.image else {
            continue;
        };
        image.change_format(format)?;
    }
    Ok(())
}
