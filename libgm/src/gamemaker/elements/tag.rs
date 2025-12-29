use std::collections::HashMap;

use macros::list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[list_chunk("TAGS")]
pub struct GMTags {
    pub tags: Vec<String>,
    pub asset_tags: HashMap<i32, Vec<String>>,
    pub exists: bool,
}

impl GMElement for GMTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("TAGS Version")?;
        let tags: Vec<String> = reader.read_simple_list()?;
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
        builder.write_simple_list(&self.tags)?;
        let temp_asset_tags: Vec<TempAssetTags> = self
            .asset_tags
            .iter()
            .map(|(&id, tags)| TempAssetTags { id, tags: tags.clone() })
            .collect();
        builder.write_pointer_list(&temp_asset_tags)?;
        Ok(())
    }
}

struct TempAssetTags {
    id: i32,
    tags: Vec<String>,
}

impl GMElement for TempAssetTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let id = reader.read_i32()?;
        let tags: Vec<String> = reader.read_simple_list()?;
        Ok(Self { id, tags })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.id);
        builder.write_simple_list(&self.tags)?;
        Ok(())
    }
}
