use crate::{
    gamemaker::{
        chunk::ChunkName, deserialize::reader::DataReader, elements::GMElement,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMTags {
    pub tags: Vec<String>,
    pub asset_tags: Vec<AssetTags>,
    pub exists: bool,
}

impl GMChunk for GMTags {
    const NAME: ChunkName = ChunkName::new("TAGS");

    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("TAGS Version")?;
        let tags: Vec<String> = reader.read_simple_list()?;
        let asset_tags: Vec<AssetTags> = reader.read_pointer_list()?;

        Ok(Self { tags, asset_tags, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1); // TAGS version
        builder.write_simple_list(&self.tags)?;
        builder.write_pointer_list(&self.asset_tags)?;
        Ok(())
    }
}

impl GMTags {
    /// Attempts to get asset tags by the given id.
    pub fn by_id(&self, id: i32) -> Option<&Vec<String>> {
        self.asset_tags
            .iter()
            .find(|at| at.id == id)
            .map(|at| &at.tags)
    }

    /// Attempts to get asset tags by the given id.
    /// but mutably ykyk
    pub fn by_id_mut(&mut self, id: i32) -> Option<&mut Vec<String>> {
        self.asset_tags
            .iter_mut()
            .find(|at| at.id == id)
            .map(|at| &mut at.tags)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetTags {
    pub id: i32,
    pub tags: Vec<String>,
}

impl GMElement for AssetTags {
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
