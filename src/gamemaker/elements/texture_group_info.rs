use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::elements::backgrounds::GMBackground;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTexture;
use crate::gamemaker::elements::fonts::GMFont;
use crate::gamemaker::gm_version::LTSBranch;
use crate::gamemaker::elements::sprites::GMSprite;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::utility::num_enum_from;

#[derive(Debug, Clone)]
pub struct GMTextureGroupInfos {
    pub texture_group_infos: Vec<GMTextureGroupInfo>,
    pub exists: bool,
}
impl GMChunkElement for GMTextureGroupInfos {
    fn stub() -> Self {
        Self { texture_group_infos: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMTextureGroupInfos {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let version: i32 = reader.read_i32()?;
        if version != 1 {
            return Err(format!("Expected TGIN version 1 but got {version}"))
        }
        let texture_group_infos: Vec<GMTextureGroupInfo> = reader.read_pointer_list()?;
        Ok(Self { texture_group_infos, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_i32(1);   // TGIN version
        builder.write_pointer_list(&self.texture_group_infos)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMTextureGroupInfo {
    pub name: GMRef<String>,
    pub texture_pages: Vec<GMRef<GMEmbeddedTexture>>,
    pub sprites: Vec<GMRef<GMSprite>>,
    pub spine_sprites: Vec<GMRef<GMSprite>>,
    pub fonts: Vec<GMRef<GMFont>>,
    pub tilesets: Vec<GMRef<GMBackground>>,
    pub data_2022_9: Option<GMTextureGroupInfo2022_9>,
}
impl GMElement for GMTextureGroupInfo {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let data_2022_9: Option<GMTextureGroupInfo2022_9> = reader.deserialize_if_gm_version((2022, 9))?;
        let texture_pages_ptr: usize = reader.read_usize()?;
        let sprites_ptr: usize = reader.read_usize()?;
        let spine_sprites_ptr: usize = if !reader.general_info.is_version_at_least((2023, 1, LTSBranch::PostLTS)) {
            reader.read_usize()?
        } else { 0 };
        let fonts_ptr: usize = reader.read_usize()?;
        let tilesets_ptr: usize = reader.read_usize()?;

        reader.assert_pos(texture_pages_ptr, "Texture Pages")?;
        let texture_pages: Vec<GMRef<GMEmbeddedTexture>> = reader.read_simple_list_of_resource_ids()?;

        reader.assert_pos(sprites_ptr, "Sprites")?;
        let sprites: Vec<GMRef<GMSprite>> = reader.read_simple_list_of_resource_ids()?;

        let spine_sprites: Vec<GMRef<GMSprite>> = if !reader.general_info.is_version_at_least((2023, 1, LTSBranch::PostLTS)) {
            reader.assert_pos(spine_sprites_ptr, "Spine Sprites")?;
            reader.read_simple_list_of_resource_ids()?
        } else { Vec::new() };

        reader.assert_pos(fonts_ptr, "Fonts")?;
        let fonts: Vec<GMRef<GMFont>> = reader.read_simple_list_of_resource_ids()?;

        reader.assert_pos(tilesets_ptr, "Tilesets")?;
        let tilesets: Vec<GMRef<GMBackground>> = reader.read_simple_list_of_resource_ids()?;

        Ok(Self { name, texture_pages, sprites, spine_sprites, fonts, tilesets, data_2022_9 })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        self.data_2022_9.serialize_if_gm_ver(builder, "Directory, Extension, LoadType", (2022, 9))?;
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
    pub directory: GMRef<String>,
    pub extension: GMRef<String>,
    pub load_type: GMTextureGroupInfoLoadType,
}
impl GMElement for GMTextureGroupInfo2022_9 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let directory: GMRef<String> = reader.read_gm_string()?;
        let extension: GMRef<String> = reader.read_gm_string()?;
        let load_type: GMTextureGroupInfoLoadType = num_enum_from(reader.read_i32()?)?;
        Ok(Self { directory, extension, load_type, })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.directory)?;
        builder.write_gm_string(&self.extension)?;
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

