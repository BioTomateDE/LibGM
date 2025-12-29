use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{
            GMElement, GMNamedElement, texture_page_item::GMTexturePageItem, validate_identifier,
        },
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

/// The embedded images of the data file.
/// This is used to store built-in particle sprites,
/// every time you use `part_sprite` functions.
#[named_list_chunk("EMBI", name_exception)]
pub struct GMEmbeddedImages {
    pub embedded_images: Vec<GMEmbeddedImage>,
    pub exists: bool,
}

impl GMElement for GMEmbeddedImages {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_gms2_chunk_version("EMBI Version")?;
        let embedded_images: Vec<GMEmbeddedImage> = reader.read_simple_list()?;
        Ok(Self { embedded_images, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(1); // EMBI version
        builder.write_simple_list(&self.embedded_images)?;
        Ok(())
    }
}

/// An embedded image entry in a GameMaker data file. This is GMS2 only.
/// Not to be confused with the other "embedded" resources, this is a bit different.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMEmbeddedImage {
    pub name: String,
    pub texture_entry: GMRef<GMTexturePageItem>,
}

impl GMNamedElement for GMEmbeddedImage {
    fn name(&self) -> &String {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn validate_name(&self) -> Result<()> {
        let name = &self.name;
        let ident = name.strip_suffix(".png");
        let Some(ident) = ident else {
            bail!("Embedded Image name {name:?} does not end in \".png\"");
        };
        validate_identifier(ident)
    }
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
