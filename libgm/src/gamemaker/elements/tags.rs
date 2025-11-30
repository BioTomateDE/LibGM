use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::assert::assert_int,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMTags {
    pub tags: Vec<String>,
    pub asset_tags: HashMap<i32, Vec<String>>,
    pub exists: bool,
}

impl Deref for GMTags {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.tags
    }
}

impl DerefMut for GMTags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tags
    }
}

impl GMChunkElement for GMTags {
    const NAME: &'static str = "TAGS";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        assert_int("TAGS Version", 1, reader.read_u32()?)?;
        let tags: Vec<String> = reader.read_simple_list_of_strings()?;
        let temp_asset_tags: Vec<TempAssetTags> = reader.read_pointer_list()?;

        let mut asset_tags: HashMap<i32, Vec<String>> = HashMap::new();
        for temp_asset_tag in temp_asset_tags {
            if asset_tags
                .insert(temp_asset_tag.id, temp_asset_tag.tags)
                .is_some()
            {
                bail!(
                    "Duplicate Asset ID {} while parsing Tags",
                    temp_asset_tag.id
                );
            }
        }
        Ok(Self { tags, asset_tags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1); // TAGS version
        builder.write_simple_list_of_strings(&self.tags)?;
        let temp_asset_tags: Vec<TempAssetTags> = self
            .asset_tags
            .clone()
            .into_iter()
            .map(|(id, tags)| TempAssetTags { id, tags })
            .collect();
        builder.write_pointer_list(&temp_asset_tags)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TempAssetTags {
    id: i32,
    tags: Vec<String>,
}

impl GMElement for TempAssetTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let id = reader.read_i32()?;
        let tags: Vec<String> = reader.read_simple_list_of_strings()?;
        Ok(Self { id, tags })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.id);
        builder.write_simple_list_of_strings(&self.tags)?;
        Ok(())
    }
}
