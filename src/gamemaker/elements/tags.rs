use std::collections::HashMap;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMTags {
    pub tags: Vec<GMRef<String>>,
    pub asset_tags: HashMap<i32, Vec<GMRef<String>>>,
    pub exists: bool,
}
impl GMChunkElement for GMTags {
    fn empty() -> Self {
        Self { tags: vec![], asset_tags: HashMap::new(), exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.align(4)?;
        let version: i32 = reader.read_i32()?;
        if version != 1 {
            return Err(format!("Expected TAGS version 1 but got {version}"))
        }
        let tags: Vec<GMRef<String>> = reader.read_simple_list_of_strings()?;
        let temp_asset_tags: Vec<TempAssetTags> = reader.read_pointer_list()?;

        let mut asset_tags: HashMap<i32, Vec<GMRef<String>>> = HashMap::new();
        for temp_asset_tag in temp_asset_tags {
            if asset_tags.insert(temp_asset_tag.id, temp_asset_tag.tags).is_some() {
                return Err(format!("Duplicate Asset ID {} while parsing Tags", temp_asset_tag.id))
            }
        }
        Ok(Self { tags, asset_tags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.align(4);
        builder.write_i32(1);   // TAGS version
        builder.write_simple_list_of_strings(&self.tags)?;
        let temp_asset_tags: Vec<TempAssetTags> = self.asset_tags
            .clone()
            .into_iter()
            .map(|(id, tags)| TempAssetTags {id, tags})
            .collect();
        builder.write_pointer_list(&temp_asset_tags)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
struct TempAssetTags {
    id: i32,
    tags: Vec<GMRef<String>>,
}
impl GMElement for TempAssetTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let id: i32 = reader.read_i32()?;
        let tags: Vec<GMRef<String>> = reader.read_simple_list_of_strings()?;
        Ok(Self { id, tags })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.id);
        builder.write_simple_list_of_strings(&self.tags)?;
        Ok(())
    }
}

