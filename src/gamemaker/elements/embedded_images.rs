use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMEmbeddedImages {
    pub embedded_images: Vec<GMEmbeddedImage>,
    pub exists: bool,
}
impl GMChunkElement for GMEmbeddedImages {

    fn stub() -> Self {
        Self { embedded_images: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMEmbeddedImages {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let version: i32 = reader.read_i32()?;
        if version != 1 {
            return Err(format!("Expected EMBI version 1 but got {version}"))
        }
        let embedded_images: Vec<GMEmbeddedImage> = reader.read_simple_list()?;
        Ok(Self { embedded_images, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_i32(1);   // EMBI version
        builder.write_simple_list(&self.embedded_images)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMEmbeddedImage {
    pub name: GMRef<String>,
    pub texture_entry: GMRef<GMTexturePageItem>,
}
impl GMElement for GMEmbeddedImage {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let texture_entry: GMRef<GMTexturePageItem> = reader.read_gm_texture()?;
        Ok(Self { name, texture_entry })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_texture(&self.texture_entry)?;
        Ok(())
    }
}

