// SPDX-License-Identifier: GPL-3.0-only

use super::string::Strings;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::GMNamedListChunk;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::GMNamedElement;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

/// The embedded images of the data file.
/// This is used to store built-in particle sprites,
/// every time you use `part_sprite` functions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EmbeddedImages {
    pub elems: Vec<EmbeddedImage>,
}

gm_list_chunk!(EMBI, EmbeddedImages, EmbeddedImage, direct);

impl GMNamedListChunk for EmbeddedImages {
    fn ref_by_name(&self, name: &str, gm_strings: &Strings) -> Result<GMRef<EmbeddedImage>> {
        for (gm_ref, elem) in self.element_refs() {
            let elems: &String = elem.name.resolve(&gm_strings.elems)?;
            if name == elems {
                return Ok(gm_ref);
            }
        }
        Err(err!("Could not find Embedded Image with name {name:?}"))
    }
}

impl GMElement for EmbeddedImages {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_gms2_chunk_version("EMBI Version")?;
        let embedded_images: Vec<EmbeddedImage> = reader.read_simple_list()?;
        Ok(Self { elems: embedded_images })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(1); // EMBI version
        builder.write_simple_list(&self.elems)?;
        Ok(())
    }
}

/// An embedded image entry in a GameMaker data file. This is GMS2 only.
/// Not to be confused with the other "embedded" resources, this is a bit
/// different.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedImage {
    pub name: GMRef<String>,
    pub texture_entry: GMRef<TexturePageItem>,
}

impl GMNamedElement for EmbeddedImage {
    fn name_ref(&self) -> GMRef<String> {
        self.name
    }

    fn validate_name(&self, _: &Strings) -> Result<()> {
        // this will be a filename or path probably
        Ok(())
    }
}

impl GMElement for EmbeddedImage {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let texture_entry: GMRef<TexturePageItem> = reader.read_gm_texture()?;
        Ok(Self { name, texture_entry })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_gm_texture(self.texture_entry)?;
        Ok(())
    }
}
