use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use crate::util::assert::assert_int;

/// The embedded images of the data file. This is used to store built-in particle sprites,
/// every time you use `part_sprite` functions.
#[derive(Debug, Clone, Default)]
pub struct GMEmbeddedImages {
    pub embedded_images: Vec<GMEmbeddedImage>,
    pub exists: bool,
}

impl GMChunkElement for GMEmbeddedImages {
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
#[derive(Debug, Clone)]
pub struct GMEmbeddedImage {
    pub name: GMRef<String>,
    pub texture_entry: GMRef<GMTexturePageItem>,
}

impl GMElement for GMEmbeddedImage {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let texture_entry: GMRef<GMTexturePageItem> = reader.read_gm_texture()?;
        Ok(Self { name, texture_entry })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_texture(&self.texture_entry)?;
        Ok(())
    }
}
