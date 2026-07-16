// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::texture_page::TexturePage;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TexturePageItems {
    pub elems: Vec<TexturePageItem>,
}

gm_list_chunk!(TPAG, TexturePageItems, TexturePageItem, direct);

impl GMElement for TexturePageItems {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let mut elems: Vec<TexturePageItem> = Vec::with_capacity(pointers.len());

        for (i, pointer) in pointers.into_iter().enumerate() {
            reader.cur_pos = pointer;
            reader
                .texture_page_item_occurrences
                .insert(pointer, GMRef::from(i));
            elems.push(TexturePageItem::deserialize(reader)?);
        }

        reader.align(4)?;
        Ok(Self { elems })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.elems)?;
        builder.align(4);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TexturePageItem {
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
    pub texture_page: GMRef<TexturePage>,
}

impl GMElement for TexturePageItem {
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
        let texture_page: GMRef<TexturePage> = GMRef::new(texture_page_id as i32);

        Ok(Self {
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
