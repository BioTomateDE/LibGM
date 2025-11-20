use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::backgrounds::GMBackground;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTexture;
use crate::gamemaker::elements::fonts::GMFont;
use crate::gamemaker::elements::sprites::GMSprite;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::LTSBranch;
use crate::gamemaker::reference::GMRef;
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::assert::assert_int;
use crate::util::init::num_enum_from;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
pub struct GMTextureGroupInfos {
    pub texture_group_infos: Vec<GMTextureGroupInfo>,
    pub exists: bool,
}

impl Deref for GMTextureGroupInfos {
    type Target = Vec<GMTextureGroupInfo>;
    fn deref(&self) -> &Self::Target {
        &self.texture_group_infos
    }
}

impl DerefMut for GMTextureGroupInfos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.texture_group_infos
    }
}

impl GMChunkElement for GMTextureGroupInfos {
    const NAME: &'static str = "TGIN";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMTextureGroupInfos {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        assert_int("TGIN Version", 1, reader.read_u32()?)?;
        let texture_group_infos: Vec<GMTextureGroupInfo> = reader.read_pointer_list()?;
        Ok(Self { texture_group_infos, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(1); // TGIN version
        builder.write_pointer_list(&self.texture_group_infos)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GMTextureGroupInfo {
    pub name: String,
    pub texture_pages: Vec<GMRef<GMEmbeddedTexture>>,
    pub sprites: Vec<GMRef<GMSprite>>,
    pub spine_sprites: Vec<GMRef<GMSprite>>,
    pub fonts: Vec<GMRef<GMFont>>,
    pub tilesets: Vec<GMRef<GMBackground>>,
    pub data_2022_9: Option<GMTextureGroupInfo2022_9>,
}

impl GMElement for GMTextureGroupInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let data_2022_9: Option<GMTextureGroupInfo2022_9> = reader.deserialize_if_gm_version((2022, 9))?;
        let texture_pages_ptr = reader.read_u32()?;
        let sprites_ptr = reader.read_u32()?;
        let spine_sprites_ptr = if !reader.general_info.is_version_at_least((2023, 1, LTSBranch::PostLTS)) {
            reader.read_u32()?
        } else {
            0
        };
        let fonts_ptr = reader.read_u32()?;
        let tilesets_ptr = reader.read_u32()?;

        reader.assert_pos(texture_pages_ptr, "Texture Pages")?;
        let texture_pages: Vec<GMRef<GMEmbeddedTexture>> = reader.read_simple_list_of_resource_ids()?;

        reader.assert_pos(sprites_ptr, "Sprites")?;
        let sprites: Vec<GMRef<GMSprite>> = reader.read_simple_list_of_resource_ids()?;

        let spine_sprites: Vec<GMRef<GMSprite>> =
            if !reader.general_info.is_version_at_least((2023, 1, LTSBranch::PostLTS)) {
                reader.assert_pos(spine_sprites_ptr, "Spine Sprites")?;
                reader.read_simple_list_of_resource_ids()?
            } else {
                Vec::new()
            };

        reader.assert_pos(fonts_ptr, "Fonts")?;
        let fonts: Vec<GMRef<GMFont>> = reader.read_simple_list_of_resource_ids()?;

        reader.assert_pos(tilesets_ptr, "Tilesets")?;
        let tilesets: Vec<GMRef<GMBackground>> = reader.read_simple_list_of_resource_ids()?;

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
        builder.write_gm_string(&self.name);
        self.data_2022_9
            .serialize_if_gm_ver(builder, "Directory, Extension, LoadType", (2022, 9))?;
        builder.write_pointer(&self.texture_pages)?;
        builder.write_pointer(&self.sprites)?;
        if !builder.is_gm_version_at_least((2023, 1, LTSBranch::PostLTS)) {
            builder.write_pointer(&self.spine_sprites)?;
        }
        builder.write_pointer(&self.fonts)?;
        builder.write_pointer(&self.tilesets)?;

        builder.resolve_pointer(&self.texture_pages)?;
        builder.write_simple_list_of_resource_ids(&self.texture_pages)?;

        builder.resolve_pointer(&self.sprites)?;
        builder.write_simple_list_of_resource_ids(&self.sprites)?;

        if !builder.is_gm_version_at_least((2023, 1, LTSBranch::PostLTS)) {
            builder.resolve_pointer(&self.spine_sprites)?;
            builder.write_simple_list_of_resource_ids(&self.spine_sprites)?;
        }

        builder.resolve_pointer(&self.fonts)?;
        builder.write_simple_list_of_resource_ids(&self.fonts)?;

        builder.resolve_pointer(&self.tilesets)?;
        builder.write_simple_list_of_resource_ids(&self.tilesets)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GMTextureGroupInfo2022_9 {
    pub directory: String,
    pub extension: String,
    pub load_type: GMTextureGroupInfoLoadType,
}

impl GMElement for GMTextureGroupInfo2022_9 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let directory: String = reader.read_gm_string()?;
        let extension: String = reader.read_gm_string()?;
        let load_type: GMTextureGroupInfoLoadType = num_enum_from(reader.read_i32()?)?;
        Ok(Self { directory, extension, load_type })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.directory);
        builder.write_gm_string(&self.extension);
        builder.write_i32(self.load_type.into());
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMTextureGroupInfoLoadType {
    /// The texture data is located inside this file.
    InFile = 0,

    /// The textures of the group this belongs to are located externally
    /// May mean more specifically that textures for one texture group are all in one file.
    SeparateGroup = 1,

    /// The textures of the group this belongs to are located externally.
    /// May mean more specifically that textures are separated into different files, within the group.
    SeparateTextures = 2,
}
