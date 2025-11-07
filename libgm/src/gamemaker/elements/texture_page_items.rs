use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTexture;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
pub struct GMTexturePageItems {
    pub texture_page_items: Vec<GMTexturePageItem>,
    pub exists: bool,
}

impl Deref for GMTexturePageItems {
    type Target = Vec<GMTexturePageItem>;
    fn deref(&self) -> &Self::Target {
        &self.texture_page_items
    }
}

impl DerefMut for GMTexturePageItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.texture_page_items
    }
}

impl GMChunkElement for GMTexturePageItems {
    const NAME: &'static str = "TPAG";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMTexturePageItems {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let mut texture_page_items: Vec<GMTexturePageItem> = Vec::with_capacity(pointers.len());

        for (i, pointer) in pointers.into_iter().enumerate() {
            reader.cur_pos = pointer;
            reader
                .texture_page_item_occurrences
                .insert(pointer, GMRef::new(i as u32));
            texture_page_items.push(GMTexturePageItem::deserialize(reader)?);
        }

        reader.align(4)?;
        Ok(Self { texture_page_items, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.texture_page_items)?;
        builder.align(4);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMTexturePageItem {
    pub source_x: u16,
    pub source_y: u16,
    pub source_width: u16,
    pub source_height: u16,
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
    pub texture_page: GMRef<GMEmbeddedTexture>,
}

impl GMElement for GMTexturePageItem {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let source_x = reader.read_u16()?;
        let source_y = reader.read_u16()?;
        let source_width = reader.read_u16()?;
        let source_height = reader.read_u16()?;
        let target_x = reader.read_u16()?;
        let target_y = reader.read_u16()?;
        let target_width = reader.read_u16()?;
        let target_height = reader.read_u16()?;
        let bounding_width = reader.read_u16()?;
        let bounding_height = reader.read_u16()?;
        let texture_page_id = reader.read_u16()?;
        let texture_page: GMRef<GMEmbeddedTexture> = GMRef::new(texture_page_id.into());

        Ok(GMTexturePageItem {
            source_x,
            source_y,
            source_width,
            source_height,
            target_x,
            target_y,
            target_width,
            target_height,
            bounding_width,
            bounding_height,
            texture_page,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.resolve_pointer(self)?;
        builder.write_u16(self.source_x);
        builder.write_u16(self.source_y);
        builder.write_u16(self.source_width);
        builder.write_u16(self.source_height);
        builder.write_u16(self.target_x);
        builder.write_u16(self.target_y);
        builder.write_u16(self.target_width);
        builder.write_u16(self.target_height);
        builder.write_u16(self.bounding_width);
        builder.write_u16(self.bounding_height);
        builder.write_u16(self.texture_page.index as u16);
        Ok(())
    }
}
