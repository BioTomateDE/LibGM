mod constant;
mod flags;
mod new;
mod old;

pub use constant::Constant;
pub use flags::Flags;

use crate::{
    gamemaker::{
        chunk::ChunkName,
        deserialize::reader::DataReader,
        elements::{GMChunk, GMElement, texture_page_item::GMTexturePageItem},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMOptions {
    is_new_format: bool,
    pub unknown1: u32,
    pub unknown2: u32,
    pub flags: Flags,
    pub window_scale: i32,
    pub window_color: u32,
    pub color_depth: u32,
    pub resolution: u32,
    pub frequency: u32,
    pub vertex_sync: i32,
    pub priority: i32,
    pub back_image: Option<GMRef<GMTexturePageItem>>,
    pub front_image: Option<GMRef<GMTexturePageItem>>,
    pub load_image: Option<GMRef<GMTexturePageItem>>,
    pub load_alpha: u32,
    pub constants: Vec<Constant>,
    pub exists: bool,
}

impl GMChunk for GMOptions {
    const NAME: ChunkName = ChunkName::new("OPTN");
    fn exists(&self) -> bool {
        self.exists
    }
}

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
            new::build(builder, self)?;
        } else {
            old::build(builder, self);
        }
        Ok(())
    }
}
