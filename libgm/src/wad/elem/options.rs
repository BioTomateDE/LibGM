// SPDX-License-Identifier: GPL-3.0-only
mod constant;
mod flags;
mod new;
mod old;

pub use constant::Constant;
pub use flags::OptionFlags;

use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::chunk::ChunkName;
use crate::wad::elem::GMElement;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

/// Most (if not all) of these options are probably unused and remnant from GM8.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub flags: OptionFlags,
    pub window_scale: i32,
    pub window_color: u32,
    pub color_depth: u32,
    pub resolution: u32,
    pub frequency: u32,
    pub vertex_sync: i32,
    pub priority: i32,
    pub back_image: GMRef<TexturePageItem>,
    pub front_image: GMRef<TexturePageItem>,
    pub load_image: GMRef<TexturePageItem>,
    pub load_alpha: u32,
    pub constants: Vec<Constant>,
}

impl GMChunk for Options {
    const NAME: ChunkName = ChunkName::OPTN;
}

impl GMElement for Options {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.version >= GMVersion::Wad14 {
            new::deserialize(reader)
        } else {
            old::deserialize(reader)
        }
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        if writer.version() >= GMVersion::Wad14 {
            new::serialize(writer, self)
        } else {
            old::serialize(writer, self)
        }
    }
}
