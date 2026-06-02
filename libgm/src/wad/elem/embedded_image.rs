// SPDX-License-Identifier: GPL-3.0-only

use super::string::GMStrings;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::GMNamedListChunk;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::GMNamedElement;
use crate::wad::elem::texture_page_item::GMTexturePageItem;
use crate::wad::elem::validate_identifier;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

/// The embedded images of the data file.
/// This is used to store built-in particle sprites,
/// every time you use `part_sprite` functions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMEmbeddedImages {
    pub embedded_images: Vec<GMEmbeddedImage>,
    pub exists: bool,
}

gm_list_chunk!(
    EMBI,
    GMEmbeddedImages,
    GMEmbeddedImage,
    embedded_images,
    direct
);

impl GMNamedListChunk for GMEmbeddedImages {
    fn ref_by_name(&self, name: &str, gm_strings: &GMStrings) -> Result<GMRef<GMEmbeddedImage>> {
        for (gm_ref, elem) in self.element_refs() {
            let elem_name: &String = elem.name.resolve(&gm_strings.strings)?;
            if name == elem_name {
                return Ok(gm_ref);
            }
        }
        Err(err!("Could not find Embedded Image with name {name:?}"))
    }
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
/// Not to be confused with the other "embedded" resources, this is a bit
/// different.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMEmbeddedImage {
    pub name: GMRef<String>,
    pub texture_entry: GMRef<GMTexturePageItem>,
}

impl GMNamedElement for GMEmbeddedImage {
    fn name_ref(&self) -> GMRef<String> {
        self.name
    }

    fn validate_name(&self, gm_strings: &GMStrings) -> Result<()> {
        let name = self.name(gm_strings)?;
        let ident = name.strip_suffix(".png");
        let Some(ident) = ident else {
            bail!("Embedded Image name {name:?} does not end in \".png\"");
        };
        validate_identifier(ident)
    }
}

impl GMElement for GMEmbeddedImage {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let texture_entry: GMRef<GMTexturePageItem> = reader.read_gm_texture()?;
        Ok(Self { name, texture_entry })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_gm_texture(self.texture_entry)?;
        Ok(())
    }
}
