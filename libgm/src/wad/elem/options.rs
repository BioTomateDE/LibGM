// SPDX-License-Identifier: GPL-3.0-only
mod constant;
mod flags;
mod new;
mod old;

pub use constant::Constant;
pub use flags::Flags;

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::texture_page_item::GMTexturePageItem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

/// Most (if not all) of these options are probably unused and remnant from GM8.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMOptions {
    is_new_format: bool,
    pub flags: Flags,
    pub window_scale: i32,
    pub window_color: u32,
    pub color_depth: u32,
    pub resolution: u32,
    pub frequency: u32,
    pub vertex_sync: i32,
    pub priority: i32,
    pub back_image: GMRef<GMTexturePageItem>,
    pub front_image: GMRef<GMTexturePageItem>,
    pub load_image: GMRef<GMTexturePageItem>,
    pub load_alpha: u32,
    pub constants: Vec<Constant>,
    pub exists: bool,
}

gm_chunk!(OPTN, GMOptions);

impl GMElement for GMOptions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let is_new_format: bool = reader.read_u32()? == 0x8000_0000;
        reader.cur_pos -= 4;
        if is_new_format {
            new::parse(reader)
        } else {
            old::parse(reader)
        }
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if self.is_new_format {
            new::build(builder, self)
        } else {
            old::build(builder, self)
        }
    }
}
