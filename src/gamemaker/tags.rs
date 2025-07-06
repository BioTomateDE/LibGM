use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gm_serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMTags {
    pub tags: Vec<GMRef<String>>,
    pub asset_tags: Vec<GMAssetTags>,
    // TODO: change this to a hashmap (before doing export logic)
    pub exists: bool,
}
impl GMChunkElement for GMTags {
    fn empty() -> Self {
        Self { tags: vec![], asset_tags: vec![], exists: false }
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
        let asset_tags: Vec<GMAssetTags> = reader.read_pointer_list()?;
        Ok(Self { tags, asset_tags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.align(4);
        builder.write_i32(1);   // TAGS version
        builder.write_simple_list_of_strings(&self.tags)?;
        builder.write_pointer_list(&self.asset_tags)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMAssetTags {
    pub id: i32,
    pub tags: Vec<GMRef<String>>,
}
impl GMElement for GMAssetTags {
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

