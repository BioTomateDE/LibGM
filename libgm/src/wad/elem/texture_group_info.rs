// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::GMNamedElement;
use crate::wad::elem::background::Background;
use crate::wad::elem::font::Font;
use crate::wad::elem::sprite::Sprite;
use crate::wad::elem::string::Strings;
use crate::wad::elem::texture_page::TexturePage;
use crate::wad::elem::validate_identifier;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::version::LTSBranch;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextureGroupInfos {
    pub elems: Vec<TextureGroupInfo>,
    pub exists: bool,
}

// not sure if direct
gm_list_chunk!(TGIN, TextureGroupInfos, TextureGroupInfo, direct);

impl GMNamedElement for TextureGroupInfo {
    fn name_ref(&self) -> GMRef<String> {
        self.name
    }

    fn validate_name(&self, gm_strings: &Strings) -> Result<()> {
        // Allow ".png" inside the identifier
        let name = self.name(gm_strings)?;
        for part in name.split_terminator(".png") {
            validate_identifier(part)?;
        }
        Ok(())
    }
}

impl GMNamedListChunk for TextureGroupInfos {
    fn ref_by_name(&self, name: &str, gm_strings: &Strings) -> Result<GMRef<Self::Element>> {
        for (gm_ref, elem) in self.element_refs() {
            let elem_name: &String = elem.name.resolve(&gm_strings.elems)?;
            if name == elem_name {
                return Ok(gm_ref);
            }
        }
        Err(err!("Could not find Texture Group Info with name {name:?}"))
    }
}

impl GMElement for TextureGroupInfos {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_gms2_chunk_version("TGIN Version")?;
        let elems: Vec<TextureGroupInfo> = reader.read_pointer_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(1); // TGIN version
        builder.write_pointer_list(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureGroupInfo {
    pub name: GMRef<String>,
    pub texture_pages: Vec<GMRef<TexturePage>>,
    pub sprites: Vec<GMRef<Sprite>>,
    pub spine_sprites: Vec<GMRef<Sprite>>,
    pub fonts: Vec<GMRef<Font>>,
    pub tilesets: Vec<GMRef<Background>>,
    pub data_2022_9: Option<Data2022_9>,
}

impl GMElement for TextureGroupInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let data_2022_9: Option<Data2022_9> = reader.deserialize_if_gm_version((2022, 9))?;
        let texture_pages_ptr = reader.read_u32()?;
        let sprites_ptr = reader.read_u32()?;
        let spine_sprites_ptr = if reader.general_info.version < (2023, 1, LTSBranch::PostLTS) {
            reader.read_u32()?
        } else {
            0
        };
        let fonts_ptr = reader.read_u32()?;
        let tilesets_ptr = reader.read_u32()?;

        reader.assert_pos(texture_pages_ptr, "Texture Pages")?;
        let texture_pages: Vec<GMRef<TexturePage>> = reader.read_simple_list()?;

        reader.assert_pos(sprites_ptr, "Sprites")?;
        let sprites: Vec<GMRef<Sprite>> = reader.read_simple_list()?;

        let spine_sprites: Vec<GMRef<Sprite>> =
            if reader.general_info.version < (2023, 1, LTSBranch::PostLTS) {
                reader.assert_pos(spine_sprites_ptr, "Spine Sprites")?;
                reader.read_simple_list()?
            } else {
                Vec::new()
            };

        reader.assert_pos(fonts_ptr, "Fonts")?;
        let fonts: Vec<GMRef<Font>> = reader.read_simple_list()?;

        reader.assert_pos(tilesets_ptr, "Tilesets")?;
        let tilesets: Vec<GMRef<Background>> = reader.read_simple_list()?;

        Ok(Self {
            name,
            texture_pages,
            sprites,
            spine_sprites,
            fonts,
            tilesets,
            data_2022_9,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_if_ver(
            &self.data_2022_9,
            "Directory, Extension, LoadType",
            (2022, 9),
        )?;
        builder.write_pointer(&self.texture_pages);
        builder.write_pointer(&self.sprites);
        if builder.version() < (2023, 1, LTSBranch::PostLTS) {
            builder.write_pointer(&self.spine_sprites);
        }
        builder.write_pointer(&self.fonts);
        builder.write_pointer(&self.tilesets);

        builder.resolve_pointer(&self.texture_pages)?;
        builder.write_simple_list(&self.texture_pages)?;

        builder.resolve_pointer(&self.sprites)?;
        builder.write_simple_list(&self.sprites)?;

        if builder.version() < (2023, 1, LTSBranch::PostLTS) {
            builder.resolve_pointer(&self.spine_sprites)?;
            builder.write_simple_list(&self.spine_sprites)?;
        }

        builder.resolve_pointer(&self.fonts)?;
        builder.write_simple_list(&self.fonts)?;

        builder.resolve_pointer(&self.tilesets)?;
        builder.write_simple_list(&self.tilesets)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data2022_9 {
    pub directory: GMRef<String>,
    pub extension: GMRef<String>,
    pub load_type: LoadType,
}

impl GMElement for Data2022_9 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let directory: GMRef<String> = reader.read_gm_string()?;
        let extension: GMRef<String> = reader.read_gm_string()?;
        let load_type: LoadType = reader.read_enum()?;
        Ok(Self { directory, extension, load_type })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.directory)?;
        builder.write_gm_string(self.extension)?;
        builder.write_enum(self.load_type);
        Ok(())
    }
}

gm_enum!(LoadType {
    /// The texture data is located inside this file.
    InFile = 0,

    /// The textures of the group this belongs to are located externally
    /// May mean more specifically that textures for one texture group are all
    /// in one file.
    SeparateGroup = 1,

    /// The textures of the group this belongs to are located externally.
    /// May mean more specifically that textures are separated into different
    /// files, within the group.
    SeparateTextures = 2,
});
