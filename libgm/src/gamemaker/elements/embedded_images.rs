use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement, texture_page_items::GMTexturePageItem},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::assert::assert_int,
};

/// The embedded images of the data file. This is used to store built-in particle sprites,
/// every time you use `part_sprite` functions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMEmbeddedImages {
    pub embedded_images: Vec<GMEmbeddedImage>,
    pub exists: bool,
}

impl Deref for GMEmbeddedImages {
    type Target = Vec<GMEmbeddedImage>;
    fn deref(&self) -> &Self::Target {
        &self.embedded_images
    }
}

impl DerefMut for GMEmbeddedImages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.embedded_images
    }
}

impl GMChunkElement for GMEmbeddedImages {
    const NAME: &'static str = "EMBI";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMEmbeddedImages {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        assert_int("EMBI Version", 1, reader.read_u32()?)?;
        let embedded_images: Vec<GMEmbeddedImage> = reader.read_simple_list()?;
        Ok(Self { embedded_images, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(1); // EMBI version
        builder.write_simple_list(&self.embedded_images)?;
        Ok(())
    }
}

/// An embedded image entry in a GameMaker data file. This is GMS2 only.<br/>
/// Not to be confused with the other "embedded" resources, this is a bit different.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMEmbeddedImage {
    pub name: String,
    pub texture_entry: GMRef<GMTexturePageItem>,
}

impl GMElement for GMEmbeddedImage {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let texture_entry: GMRef<GMTexturePageItem> = reader.read_gm_texture()?;
        Ok(Self { name, texture_entry })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_gm_texture(self.texture_entry)?;
        Ok(())
    }
}
