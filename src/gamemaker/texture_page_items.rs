use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::embedded_textures::GMEmbeddedTexture;
use crate::gm_serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMTexturePageItems {
    pub texture_page_items: Vec<GMTexturePageItem>,
    pub exists: bool,
}
impl GMChunkElement for GMTexturePageItems {
    fn empty() -> Self {
        Self { texture_page_items: vec![], exists: false }
    }
}
impl GMElement for GMTexturePageItems {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let texture_page_items: Vec<GMTexturePageItem> = reader.read_texture_page_items_with_occurrences()?;
        Ok(Self { texture_page_items, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.texture_page_items)?;
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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let source_x: u16 = reader.read_u16()?;
        let source_y: u16 = reader.read_u16()?;
        let source_width: u16 = reader.read_u16()?;
        let source_height: u16 = reader.read_u16()?;
        let target_x: u16 = reader.read_u16()?;
        let target_y: u16 = reader.read_u16()?;
        let target_width: u16 = reader.read_u16()?;
        let target_height: u16 = reader.read_u16()?;
        let bounding_width: u16 = reader.read_u16()?;
        let bounding_height: u16 = reader.read_u16()?;
        let texture_page_id: u16 = reader.read_u16()?;
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
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
        builder.write_resource_id(&self.texture_page);
        Ok(())
    }
}

